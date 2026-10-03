use async_trait::async_trait;
use sqlx::{
    sqlite::{SqlitePoolOptions, SqliteRow},
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

fn extract_sqlite_value(row: &SqliteRow, col_idx: usize) -> serde_json::Value {
    if let Ok(val) = row.try_get::<Option<String>, _>(col_idx) {
        return val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i64>, _>(col_idx) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<f64>, _>(col_idx) {
        return val.and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number)).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<bool>, _>(col_idx) {
        return val.map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null);
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
                row_values.push(extract_sqlite_value(row, col_idx));
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
        let mut relations = Vec::new();

        for row in rows {
            let name: String = row.try_get("name").unwrap_or_default();
            let table_type: String = row.try_get("type").unwrap_or_else(|_| "table".into());

            if !name.is_empty() {
                // Fetch columns via PRAGMA table_info
                let pragma_sql = format!("PRAGMA table_info(\"{}\");", name);
                let col_rows = sqlx::query(&pragma_sql)
                    .fetch_all(&self.pool)
                    .await
                    .unwrap_or_default();

                let mut columns = Vec::new();
                for cr in col_rows {
                    let col_name: String = cr.try_get("name").unwrap_or_default();
                    let data_type: String = cr.try_get("type").unwrap_or_else(|_| "TEXT".into());
                    let not_null: i64 = cr.try_get("notnull").unwrap_or(0);
                    let is_pk: i64 = cr.try_get("pk").unwrap_or(0);

                    columns.push(ColumnMetadata {
                        name: col_name.clone(),
                        data_type,
                        is_primary_key: is_pk > 0 || col_name.eq_ignore_ascii_case("id"),
                        is_foreign_key: col_name.ends_with("_id"),
                        nullable: not_null == 0,
                    });
                }

                // Fetch foreign keys via PRAGMA foreign_key_list
                let fk_pragma = format!("PRAGMA foreign_key_list(\"{}\");", name);
                if let Ok(fk_rows) = sqlx::query(&fk_pragma).fetch_all(&self.pool).await {
                    for fkr in fk_rows {
                        let id: i64 = fkr.try_get("id").unwrap_or(0);
                        let to_table: String = fkr.try_get("table").unwrap_or_default();
                        let from_col: String = fkr.try_get("from").unwrap_or_default();
                        let to_col: String = fkr.try_get("to").unwrap_or_default();

                        if !to_table.is_empty() {
                            relations.push(RelationEdge {
                                id: format!("fk_{}_{}_{}", name, from_col, id),
                                from_table: name.clone(),
                                from_column: from_col,
                                to_table,
                                to_column: to_col,
                            });
                        }
                    }
                }

                let count_sql = format!("SELECT COUNT(*) AS c FROM \"{}\"", name);
                let row_count = if table_type != "view" {
                    sqlx::query(&count_sql)
                        .fetch_one(&self.pool)
                        .await
                        .ok()
                        .and_then(|r| r.try_get::<i64, _>("c").ok())
                } else {
                    None
                };

                tables.push(TableItem {
                    schema: "main".into(),
                    name,
                    table_type: if table_type == "view" { "view".into() } else { "table".into() },
                    row_count_estimate: row_count,
                    columns,
                });
            }
        }

        Ok(SchemaTree {
            databases: vec!["main".into()],
            current_database: "main".into(),
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
