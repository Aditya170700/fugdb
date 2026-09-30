use async_trait::async_trait;
use sqlx::{
    sqlite::{SqliteColumn, SqlitePoolOptions, SqliteRow},
    SqlitePool, Row, Column,
};
use std::time::Instant;
use tokio::sync::mpsc;

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::{
    connection::ConnectionConfig,
    query::{ColumnMetadata, QueryResult},
    schema::{RelationEdge, SchemaTree, TableItem},
    transfer::ConflictStrategy,
};

pub struct SqliteAdapter {
    pool: SqlitePool,
    _db_path: String,
}

impl SqliteAdapter {
    pub async fn new(config: &ConnectionConfig) -> Result<Self, AppError> {
        let path = config.file_path.as_deref().unwrap_or("fugdb_local.sqlite");
        let url = format!("sqlite://{}?mode=rwc", path);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| AppError::ConnectionError(e.to_string()))?;

        Ok(Self {
            pool,
            _db_path: path.to_string(),
        })
    }
}

fn extract_sqlite_value(row: &SqliteRow, col: &SqliteColumn) -> serde_json::Value {
    let name = col.name();

    if let Ok(val) = row.try_get::<Option<String>, _>(name) {
        return val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i64>, _>(name) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<f64>, _>(name) {
        return val.and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number)).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<bool>, _>(name) {
        return val.map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<Vec<u8>>, _>(name) {
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
impl DatabaseAdapter for SqliteAdapter {
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
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::QueryError(e.to_string()))?;
        let duration = start.elapsed().as_secs_f64() * 1000.0;

        let mut columns = Vec::new();
        let mut result_rows = Vec::new();

        if let Some(first_row) = rows.first() {
            for col in first_row.columns() {
                columns.push(ColumnMetadata {
                    name: col.name().to_string(),
                    data_type: col.type_info().to_string(),
                    is_primary_key: false,
                    is_foreign_key: false,
                    nullable: true,
                });
            }
        }

        for row in &rows {
            let mut row_values = Vec::new();
            for col in row.columns() {
                row_values.push(extract_sqlite_value(row, col));
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

    async fn fetch_schema_tree(&self) -> Result<SchemaTree, AppError> {
        let sql = r#"
            SELECT name, type 
            FROM sqlite_master 
            WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'
            ORDER BY name;
        "#;

        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut tables = Vec::new();
        for row in rows {
            let name: String = row.try_get("name").unwrap_or_default();
            let table_type: String = row.try_get("type").unwrap_or_else(|_| "table".into());

            if !name.is_empty() {
                tables.push(TableItem {
                    schema: "main".into(),
                    name,
                    table_type: if table_type == "view" { "view".into() } else { "table".into() },
                    row_count_estimate: None,
                });
            }
        }

        Ok(SchemaTree {
            databases: vec!["main".into()],
            current_database: "main".into(),
            tables,
        })
    }

    async fn generate_erd_metadata(&self) -> Result<Vec<RelationEdge>, AppError> {
        Ok(vec![])
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
