use async_trait::async_trait;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode},
    MySqlPool, Row, Column,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::{
    connection::ConnectionConfig,
    query::{ColumnMetadata, QueryResult},
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
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(connect_opts)
            .await
            .map_err(|e| AppError::ConnectionError(format!("MySQL connection failed: {}", e)))?;

        Ok(Self {
            pool,
            database_name: database.to_string(),
        })
    }
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
                let val: Option<String> = row.try_get(col.name()).ok();
                row_values.push(match val {
                    Some(v) => serde_json::Value::String(v),
                    None => serde_json::Value::Null,
                });
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
            SELECT TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE
            FROM information_schema.tables
            WHERE TABLE_SCHEMA = DATABASE()
            ORDER BY TABLE_NAME;
        "#;

        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut tables = Vec::new();
        for row in rows {
            let schema: String = row.get("TABLE_SCHEMA");
            let name: String = row.get("TABLE_NAME");
            let table_type: String = row.get("TABLE_TYPE");

            tables.push(TableItem {
                schema,
                name,
                table_type: if table_type == "VIEW" { "view".into() } else { "table".into() },
                row_count_estimate: None,
            });
        }

        Ok(SchemaTree {
            databases: vec![self.database_name.clone()],
            current_database: self.database_name.clone(),
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
