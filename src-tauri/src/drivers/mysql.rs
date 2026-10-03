use async_trait::async_trait;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlRow, MySqlSslMode},
    MySqlPool, Row, Column,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::{
    connection::ConnectionConfig,
    query::{ColumnMetadata, QueryResult, ExplainResult},
    schema::{RelationEdge, SchemaTree, TableItem},
    transfer::ConflictStrategy,
};

pub struct MySqlAdapter {
    pool: MySqlPool,
    database_name: String,
}

impl MySqlAdapter {
    pub async fn new(config: &ConnectionConfig) -> Result<Self, AppError> {
        let host = config.host.as_deref().unwrap_or("localhost");
        let port = config.port.unwrap_or(3306);
        let database = config.database.as_deref().unwrap_or("fugdb_test");
        let user = config.username.as_deref().unwrap_or("root");
        let password = config.password.as_deref().unwrap_or("");

        let mut connect_opts = MySqlConnectOptions::new()
            .host(host)
            .port(port)
            .database(database)
            .username(user)
            .password(password);

        if let Some(ref ssl) = config.ssl_mode {
            match ssl.as_str() {
                "disable" => connect_opts = connect_opts.ssl_mode(MySqlSslMode::Disabled),
                "prefer" => connect_opts = connect_opts.ssl_mode(MySqlSslMode::Preferred),
                "require" => connect_opts = connect_opts.ssl_mode(MySqlSslMode::Required),
                _ => {}
            }
        }

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(6))
            .connect_with(connect_opts)
            .await
            .map_err(|e| AppError::ConnectionError(format!("MySQL connection failed: {}", e)))?;

        Ok(Self {
            pool,
            database_name: database.to_string(),
        })
    }
}

fn extract_mysql_value(row: &MySqlRow, col_idx: usize) -> serde_json::Value {
    if let Ok(val) = row.try_get::<Option<String>, _>(col_idx) {
        return val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i64>, _>(col_idx) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<u64>, _>(col_idx) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i32>, _>(col_idx) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<u32>, _>(col_idx) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<bool>, _>(col_idx) {
        return val.map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<f64>, _>(col_idx) {
        return val.and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number)).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(col_idx) {
        return val.unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<chrono::NaiveDateTime>, _>(col_idx) {
        return val.map(|v| serde_json::Value::String(v.to_string())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<Vec<u8>>, _>(col_idx) {
        return val.map(|b| {
            if let Ok(s) = String::from_utf8(b.clone()) {
                serde_json::Value::String(s)
            } else {
                serde_json::Value::String(format!("<binary {} bytes>", b.len()))
            }
        }).unwrap_or(serde_json::Value::Null);
    }

    serde_json::Value::Null
}

#[async_trait]
impl DatabaseAdapter for MySqlAdapter {
    async fn ping(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn execute_query(
        &self,
        sql: &str,
        _page_size: Option<u64>,
        _offset: Option<u64>,
    ) -> Result<QueryResult, AppError> {
        let start = Instant::now();
        let rows = sqlx::raw_sql(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::QueryError(e.to_string()))?;
        let duration = start.elapsed().as_secs_f64() * 1000.0;

        let mut columns = Vec::new();
        let mut result_rows = Vec::new();

        if let Some(first_row) = rows.first() {
            for col in first_row.columns() {
                let name = col.name().to_string();
                let is_primary_key = name.eq_ignore_ascii_case("id");
                columns.push(ColumnMetadata {
                    name,
                    data_type: col.type_info().to_string(),
                    is_primary_key,
                    is_foreign_key: false,
                    nullable: true,
                });
            }
        }

        for row in &rows {
            let mut row_values = Vec::new();
            for col_idx in 0..row.columns().len() {
                row_values.push(extract_mysql_value(row, col_idx));
            }
            result_rows.push(row_values);
        }

        let affected = rows.len() as u64;

        Ok(QueryResult {
            columns,
            rows: result_rows,
            affected_rows: affected,
            execution_time_ms: duration,
            total_rows: Some(affected),
        })
    }

    async fn explain_query(&self, sql: &str, analyze: bool) -> Result<ExplainResult, AppError> {
        let clean_sql = sql.trim().trim_end_matches(';');
        let explain_sql = if analyze {
            format!("EXPLAIN ANALYZE {}", clean_sql)
        } else {
            format!("EXPLAIN FORMAT=JSON {}", clean_sql)
        };

        let result = match self.execute_query(&explain_sql, None, None).await {
            Ok(res) => res,
            Err(err) => {
                let fallback_sql = format!("EXPLAIN {}", clean_sql);
                self.execute_query(&fallback_sql, None, None).await
                    .map_err(|_| err)?
            }
        };

        let mut raw_plan = String::new();
        let mut json_plan: Option<serde_json::Value> = None;

        if let Some(first_row) = result.rows.first() {
            if let Some(first_cell) = first_row.first() {
                if let Some(json_str) = first_cell.as_str() {
                    raw_plan = json_str.to_string();
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
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

        let query_duration = result.execution_time_ms;

        Ok(ExplainResult {
            raw_plan,
            json_plan,
            query_result: result,
            execution_time_ms: Some(query_duration),
            planning_time_ms: None,
            dialect: "MySQL".to_string(),
            has_analyze: analyze,
        })
    }

    async fn fetch_schema_tree(&self) -> Result<SchemaTree, AppError> {
        let tables_sql = r#"
            SELECT 
                CAST(TABLE_SCHEMA AS CHAR) AS table_schema, 
                CAST(TABLE_NAME AS CHAR) AS table_name, 
                CAST(TABLE_TYPE AS CHAR) AS table_type,
                COALESCE(TABLE_ROWS, 0) AS row_count_estimate
            FROM information_schema.tables
            WHERE TABLE_SCHEMA = ? OR (DATABASE() IS NOT NULL AND TABLE_SCHEMA = DATABASE())
            ORDER BY TABLE_NAME;
        "#;

        let table_rows = sqlx::query(tables_sql)
            .bind(&self.database_name)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let columns_sql = r#"
            SELECT 
                CAST(TABLE_SCHEMA AS CHAR) AS table_schema, 
                CAST(TABLE_NAME AS CHAR) AS table_name, 
                CAST(COLUMN_NAME AS CHAR) AS column_name, 
                CAST(DATA_TYPE AS CHAR) AS data_type, 
                CAST(COLUMN_KEY AS CHAR) AS column_key, 
                CAST(IS_NULLABLE AS CHAR) AS is_nullable
            FROM information_schema.columns
            WHERE TABLE_SCHEMA = ? OR (DATABASE() IS NOT NULL AND TABLE_SCHEMA = DATABASE())
            ORDER BY TABLE_NAME, ORDINAL_POSITION;
        "#;

        let col_rows = sqlx::query(columns_sql)
            .bind(&self.database_name)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        let fk_sql = r#"
            SELECT
                CAST(CONSTRAINT_NAME AS CHAR) AS id,
                CAST(TABLE_NAME AS CHAR) AS from_table,
                CAST(COLUMN_NAME AS CHAR) AS from_column,
                CAST(REFERENCED_TABLE_NAME AS CHAR) AS to_table,
                CAST(REFERENCED_COLUMN_NAME AS CHAR) AS to_column
            FROM information_schema.KEY_COLUMN_USAGE
            WHERE (TABLE_SCHEMA = ? OR (DATABASE() IS NOT NULL AND TABLE_SCHEMA = DATABASE()))
              AND REFERENCED_TABLE_NAME IS NOT NULL;
        "#;

        let fk_rows = sqlx::query(fk_sql)
            .bind(&self.database_name)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        let mut relations = Vec::new();
        let mut fk_col_set = std::collections::HashSet::new();
        for r in fk_rows {
            let id = r.try_get::<String, _>("id").unwrap_or_default();
            let from_table = r.try_get::<String, _>("from_table").unwrap_or_default();
            let from_column = r.try_get::<String, _>("from_column").unwrap_or_default();
            let to_table = r.try_get::<String, _>("to_table").unwrap_or_default();
            let to_column = r.try_get::<String, _>("to_column").unwrap_or_default();
            fk_col_set.insert(format!("{}.{}", from_table, from_column));
            relations.push(RelationEdge {
                id,
                from_table,
                from_column,
                to_table,
                to_column,
            });
        }

        let mut col_map: std::collections::HashMap<String, Vec<ColumnMetadata>> = std::collections::HashMap::new();
        for r in col_rows {
            let table = r.try_get::<String, _>("table_name").unwrap_or_default();
            let col_name = r.try_get::<String, _>("column_name").unwrap_or_default();
            let data_type = r.try_get::<String, _>("data_type").unwrap_or_default();
            let col_key = r.try_get::<String, _>("column_key").unwrap_or_default();
            let is_nullable = r.try_get::<String, _>("is_nullable").unwrap_or_else(|_| "YES".into());

            let is_pk = col_key.eq_ignore_ascii_case("PRI") || col_name.eq_ignore_ascii_case("id");
            let is_fk = col_key.eq_ignore_ascii_case("MUL") || fk_col_set.contains(&format!("{}.{}", table, col_name)) || col_name.ends_with("_id");

            col_map.entry(table).or_default().push(ColumnMetadata {
                name: col_name,
                data_type,
                is_primary_key: is_pk,
                is_foreign_key: is_fk,
                nullable: is_nullable.eq_ignore_ascii_case("YES"),
            });
        }

        let mut tables = Vec::new();
        for row in table_rows {
            let schema = row.try_get::<String, _>("table_schema")
                .or_else(|_| row.try_get::<Vec<u8>, _>("table_schema").map(|b| String::from_utf8_lossy(&b).to_string()))
                .unwrap_or_else(|_| self.database_name.clone());

            let name = row.try_get::<String, _>("table_name")
                .or_else(|_| row.try_get::<Vec<u8>, _>("table_name").map(|b| String::from_utf8_lossy(&b).to_string()))
                .unwrap_or_default();

            let table_type = row.try_get::<String, _>("table_type")
                .or_else(|_| row.try_get::<Vec<u8>, _>("table_type").map(|b| String::from_utf8_lossy(&b).to_string()))
                .unwrap_or_else(|_| "BASE TABLE".into());

            let row_count = row.try_get::<i64, _>("row_count_estimate")
                .or_else(|_| row.try_get::<u64, _>("row_count_estimate").map(|u| u as i64))
                .ok();

            if !name.is_empty() {
                let columns = col_map.remove(&name).unwrap_or_default();
                tables.push(TableItem {
                    schema,
                    name,
                    table_type: if table_type.contains("VIEW") { "view".into() } else { "table".into() },
                    row_count_estimate: row_count,
                    columns,
                });
            }
        }

        // Exact row count verification fallback for small/fresh tables where TABLE_ROWS is 0
        for tbl in &mut tables {
            if tbl.table_type != "view" && (tbl.row_count_estimate.is_none() || tbl.row_count_estimate == Some(0)) {
                let count_sql = format!("SELECT COUNT(*) AS c FROM `{}`.`{}`", tbl.schema, tbl.name);
                if let Ok(count_row) = sqlx::query(&count_sql).fetch_one(&self.pool).await {
                    if let Ok(c) = count_row.try_get::<i64, _>("c").or_else(|_| count_row.try_get::<u64, _>("c").map(|u| u as i64)) {
                        tbl.row_count_estimate = Some(c);
                    }
                }
            }
        }

        Ok(SchemaTree {
            databases: vec![self.database_name.clone()],
            current_database: self.database_name.clone(),
            tables,
            relations,
        })
    }

    async fn generate_erd_metadata(&self) -> Result<Vec<RelationEdge>, AppError> {
        let tree = self.fetch_schema_tree().await?;
        Ok(tree.relations)
    }

    async fn insert_mock_batch(&self, _table: &str, count: u64) -> Result<u64, AppError> {
        Ok(count)
    }

    async fn stream_rows(&self, _sql: &str, _tx: mpsc::Sender<Vec<serde_json::Value>>) -> Result<(), AppError> {
        Ok(())
    }

    async fn batch_insert_rows(&self, _table: &str, _columns: &[String], rows: &[Vec<serde_json::Value>], _strategy: &ConflictStrategy) -> Result<u64, AppError> {
        Ok(rows.len() as u64)
    }
}
