use async_trait::async_trait;
use sqlx::{
    mysql::{MySqlColumn, MySqlConnectOptions, MySqlPoolOptions, MySqlRow, MySqlSslMode},
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

fn extract_mysql_value(row: &MySqlRow, col: &MySqlColumn) -> serde_json::Value {
    let name = col.name();

    if let Ok(val) = row.try_get::<Option<String>, _>(name) {
        return val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i64>, _>(name) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<u64>, _>(name) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i32>, _>(name) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<u32>, _>(name) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<bool>, _>(name) {
        return val.map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<f64>, _>(name) {
        return val.and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number)).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(name) {
        return val.unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<chrono::NaiveDateTime>, _>(name) {
        return val.map(|v| serde_json::Value::String(v.to_string())).unwrap_or(serde_json::Value::Null);
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
                row_values.push(extract_mysql_value(row, col));
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
            SELECT 
                CAST(TABLE_SCHEMA AS CHAR) AS table_schema, 
                CAST(TABLE_NAME AS CHAR) AS table_name, 
                CAST(TABLE_TYPE AS CHAR) AS table_type
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
            let schema = row.try_get::<String, _>("table_schema")
                .or_else(|_| row.try_get::<Vec<u8>, _>("table_schema").map(|b| String::from_utf8_lossy(&b).to_string()))
                .unwrap_or_else(|_| self.database_name.clone());

            let name = row.try_get::<String, _>("table_name")
                .or_else(|_| row.try_get::<Vec<u8>, _>("table_name").map(|b| String::from_utf8_lossy(&b).to_string()))
                .unwrap_or_default();

            let table_type = row.try_get::<String, _>("table_type")
                .or_else(|_| row.try_get::<Vec<u8>, _>("table_type").map(|b| String::from_utf8_lossy(&b).to_string()))
                .unwrap_or_else(|_| "BASE TABLE".into());

            if !name.is_empty() {
                tables.push(TableItem {
                    schema,
                    name,
                    table_type: if table_type.contains("VIEW") { "view".into() } else { "table".into() },
                    row_count_estimate: None,
                });
            }
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
