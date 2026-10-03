use std::collections::HashMap;
use std::time::Instant;
use tauri::State;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::AppError;
use crate::qa::mock_generator::{
    infer_generator_type, ColumnMockRule, MockDataGenerator, TableMockInspection,
};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MockBatchResult {
    pub inserted_rows: usize,
    pub execution_time_ms: f64,
    pub chunks_count: usize,
    pub table_name: String,
}

#[tauri::command]
pub async fn inspect_table_mock_config(
    connection_id: String,
    table_name: String,
    state: State<'_, AppState>,
) -> Result<TableMockInspection, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let tree = adapter.fetch_schema_tree().await?;

    // Find matching table
    let table = tree
        .tables
        .iter()
        .find(|t| t.name == table_name || format!("{}.{}", t.schema, t.name) == table_name)
        .ok_or_else(|| AppError::QueryError(format!("Table '{}' not found in schema", table_name)))?;

    let mut rules = Vec::new();

    for col in &table.columns {
        let is_fk = col.is_foreign_key;
        let is_pk = col.is_primary_key;

        // Check if there is relation target
        let fk_rel = tree.relations.iter().find(|r| {
            (r.from_table == table.name || r.from_table == format!("{}.{}", table.schema, table.name))
                && r.from_column == col.name
        });

        let (fk_target_table, fk_target_column) = match fk_rel {
            Some(r) => (Some(r.to_table.clone()), Some(r.to_column.clone())),
            None => (None, None),
        };

        let suggested_type = infer_generator_type(&col.name, &col.data_type, is_pk, is_fk);

        let mut sample_fk_values = Vec::new();

        // If it's an FK and target table/column is known, query sample existing FK values
        if let (Some(target_tbl), Some(target_col)) = (&fk_target_table, &fk_target_column) {
            let sample_sql = format!(
                "SELECT DISTINCT {} FROM {} WHERE {} IS NOT NULL LIMIT 50",
                target_col, target_tbl, target_col
            );

            if let Ok(res) = adapter.execute_query(&sample_sql, Some(50), Some(0)).await {
                for row in res.rows {
                    if let Some(val) = row.first() {
                        if !val.is_null() {
                            sample_fk_values.push(val.clone());
                        }
                    }
                }
            }
        }

        // Auto-include logic: auto-increment PKs can default to exclude so sequence handles them, or include if desired
        let include = suggested_type != "auto_increment";

        rules.push(ColumnMockRule {
            column_name: col.name.clone(),
            data_type: col.data_type.clone(),
            is_primary_key: is_pk,
            is_foreign_key: is_fk,
            nullable: col.nullable,
            include,
            generator_type: suggested_type.to_string(),
            null_percentage: if col.nullable { 10 } else { 0 },
            custom_options: None,
            fk_target_table,
            fk_target_column,
            sample_fk_values,
        });
    }

    Ok(TableMockInspection {
        table_name: table.name.clone(),
        columns: rules,
    })
}

#[tauri::command]
pub fn preview_mock_rows(
    rules: Vec<ColumnMockRule>,
    count: usize,
) -> Vec<HashMap<String, Value>> {
    let mut generator = MockDataGenerator::new();
    let mut rows = Vec::with_capacity(count);

    for i in 0..count {
        rows.push(generator.generate_row(&rules, i));
    }

    rows
}

#[tauri::command]
pub fn generate_mock_sql_script(
    table_name: String,
    rules: Vec<ColumnMockRule>,
    count: usize,
) -> String {
    let mut generator = MockDataGenerator::new();
    let active_columns: Vec<&ColumnMockRule> = rules.iter().filter(|r| r.include).collect();

    if active_columns.is_empty() {
        return "-- No columns selected for generation".into();
    }

    let col_names: Vec<String> = active_columns.iter().map(|c| c.column_name.clone()).collect();
    let mut sql = String::new();
    sql.push_str(&format!(
        "-- Smart QA Mock Data Script for `{}`\n-- Generated: {} rows\n\n",
        table_name, count
    ));

    let chunk_size = 100;
    for chunk_start in (0..count).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(count);
        sql.push_str(&format!(
            "INSERT INTO {} ({})\nVALUES\n",
            table_name,
            col_names.join(", ")
        ));

        let mut row_strings = Vec::new();
        for i in chunk_start..chunk_end {
            let row_map = generator.generate_row(&rules, i);
            let val_strings: Vec<String> = active_columns
                .iter()
                .map(|c| {
                    let v = row_map.get(&c.column_name).unwrap_or(&Value::Null);
                    format_sql_value(v)
                })
                .collect();
            row_strings.push(format!("  ({})", val_strings.join(", ")));
        }

        sql.push_str(&row_strings.join(",\n"));
        sql.push_str(";\n\n");
    }

    sql
}

#[tauri::command]
pub async fn execute_mock_batch_insert(
    connection_id: String,
    table_name: String,
    rules: Vec<ColumnMockRule>,
    count: usize,
    chunk_size: Option<usize>,
    state: State<'_, AppState>,
) -> Result<MockBatchResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let active_columns: Vec<&ColumnMockRule> = rules.iter().filter(|r| r.include).collect();
    if active_columns.is_empty() {
        return Err(AppError::QueryError("No columns selected for insert".into()));
    }

    let col_names: Vec<String> = active_columns.iter().map(|c| c.column_name.clone()).collect();
    let batch_size = chunk_size.unwrap_or(250).clamp(20, 1000);

    let start_time = Instant::now();
    let mut generator = MockDataGenerator::new();
    let mut total_inserted = 0;
    let mut chunks_count = 0;

    for chunk_start in (0..count).step_by(batch_size) {
        let chunk_end = (chunk_start + batch_size).min(count);
        let mut row_strings = Vec::new();

        for i in chunk_start..chunk_end {
            let row_map = generator.generate_row(&rules, i);
            let val_strings: Vec<String> = active_columns
                .iter()
                .map(|c| {
                    let v = row_map.get(&c.column_name).unwrap_or(&Value::Null);
                    format_sql_value(v)
                })
                .collect();
            row_strings.push(format!("({})", val_strings.join(", ")));
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {};",
            table_name,
            col_names.join(", "),
            row_strings.join(", ")
        );

        adapter.execute_query(&sql, None, None).await?;
        total_inserted += chunk_end - chunk_start;
        chunks_count += 1;
    }

    let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    Ok(MockBatchResult {
        inserted_rows: total_inserted,
        execution_time_ms: elapsed_ms,
        chunks_count,
        table_name,
    })
}

// Fallback legacy command for backward compatibility
#[tauri::command]
pub async fn generate_mock_batch(
    connection_id: String,
    table: String,
    count: u64,
    state: State<'_, AppState>,
) -> Result<u64, AppError> {
    let inspection = inspect_table_mock_config(connection_id.clone(), table.clone(), state.clone()).await?;
    let res = execute_mock_batch_insert(
        connection_id,
        table,
        inspection.columns,
        count as usize,
        Some(250),
        state,
    )
    .await?;

    Ok(res.inserted_rows as u64)
}

fn format_sql_value(val: &Value) -> String {
    match val {
        Value::Null => "NULL".into(),
        Value::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            let escaped = s.replace('\'', "''");
            format!("'{}'", escaped)
        }
        Value::Array(arr) => {
            let json_str = serde_json::to_string(arr).unwrap_or_else(|_| "[]".into());
            format!("'{}'", json_str.replace('\'', "''"))
        }
        Value::Object(obj) => {
            let json_str = serde_json::to_string(obj).unwrap_or_else(|_| "{}".into());
            format!("'{}'", json_str.replace('\'', "''"))
        }
    }
}
