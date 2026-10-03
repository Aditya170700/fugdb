use serde::{Deserialize, Serialize};
use tauri::State;
use crate::error::AppError;
use crate::models::query::QueryResult;
use crate::state::AppState;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExplainResult {
    pub raw_plan: String,
    pub json_plan: Option<serde_json::Value>,
    pub query_result: QueryResult,
    pub execution_time_ms: Option<f64>,
    pub planning_time_ms: Option<f64>,
    pub dialect: String,
    pub has_analyze: bool,
}

#[tauri::command]
pub async fn execute_query(
    connection_id: String,
    sql: String,
    page_size: Option<u64>,
    offset: Option<u64>,
    state: State<'_, AppState>,
) -> Result<QueryResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    adapter.execute_query(&sql, page_size, offset).await
}

#[tauri::command]
pub async fn explain_query(
    connection_id: String,
    sql: String,
    analyze: bool,
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<ExplainResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let driver_name = driver.unwrap_or_else(|| "postgres".to_string()).to_lowercase();
    let clean_sql = sql.trim().trim_end_matches(';');

    let (explain_sql, dialect) = match driver_name.as_str() {
        "sqlite" => (
            format!("EXPLAIN QUERY PLAN {}", clean_sql),
            "SQLite".to_string(),
        ),
        "mysql" => (
            if analyze {
                format!("EXPLAIN ANALYZE {}", clean_sql)
            } else {
                format!("EXPLAIN FORMAT=JSON {}", clean_sql)
            },
            "MySQL".to_string(),
        ),
        "mssql" | "sqlserver" => (
            format!("SET SHOWPLAN_ALL ON; {} ;", clean_sql),
            "Microsoft SQL Server".to_string(),
        ),
        _ => (
            if analyze {
                format!("EXPLAIN (ANALYZE, COSTS, VERBOSE, BUFFERS, FORMAT JSON) {}", clean_sql)
            } else {
                format!("EXPLAIN (COSTS, VERBOSE, FORMAT JSON) {}", clean_sql)
            },
            "PostgreSQL".to_string(),
        ),
    };

    // Execute the explain query
    let result = match adapter.execute_query(&explain_sql, None, None).await {
        Ok(res) => res,
        Err(err) => {
            // Fallback for Postgres if BUFFERS or ANALYZE fails on write operations
            if driver_name == "postgres" || driver_name == "postgresql" {
                let fallback_sql = format!("EXPLAIN (FORMAT JSON) {}", clean_sql);
                adapter.execute_query(&fallback_sql, None, None).await
                    .map_err(|_| err)?
            } else {
                return Err(err);
            }
        }
    };

    let mut raw_plan = String::new();
    let mut json_plan: Option<serde_json::Value> = None;
    let mut execution_time_ms: Option<f64> = None;
    let mut planning_time_ms: Option<f64> = None;

    if let Some(first_row) = result.rows.first() {
        if let Some(first_cell) = first_row.first() {
            if let Some(json_str) = first_cell.as_str() {
                raw_plan = json_str.to_string();
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(arr) = parsed.as_array() {
                        if let Some(first_item) = arr.first() {
                            planning_time_ms = first_item.get("Planning Time").and_then(|v| v.as_f64());
                            execution_time_ms = first_item.get("Execution Time").and_then(|v| v.as_f64());
                        }
                    }
                    json_plan = Some(parsed);
                }
            } else if first_cell.is_object() || first_cell.is_array() {
                json_plan = Some(first_cell.clone());
                raw_plan = serde_json::to_string_pretty(first_cell).unwrap_or_default();
            }
        }
    }

    if raw_plan.is_empty() {
        raw_plan = result.rows.iter().map(|r| {
            r.iter().map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => "".to_string(),
                _ => v.to_string(),
            }).collect::<Vec<_>>().join(" | ")
        }).collect::<Vec<_>>().join("\n");
    }

    Ok(ExplainResult {
        raw_plan,
        json_plan,
        query_result: result,
        execution_time_ms,
        planning_time_ms,
        dialect,
        has_analyze: analyze,
    })
}
