use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use std::time::Instant;
use tiberius::{AuthMethod, ColumnData, Config};
use tokio::sync::mpsc;

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::{
    connection::ConnectionConfig,
    query::{ColumnMetadata, QueryResult},
    schema::{RelationEdge, SchemaTree, TableItem},
    transfer::ConflictStrategy,
};

pub struct MssqlAdapter {
    pool: Pool<ConnectionManager>,
    database_name: String,
}

impl MssqlAdapter {
    pub async fn new(config: &ConnectionConfig) -> Result<Self, AppError> {
        let host = config.host.as_deref().unwrap_or("localhost");
        let port = config.port.unwrap_or(1433);
        let database = config.database.as_deref().unwrap_or("master");
        let user = config.username.as_deref().unwrap_or("sa");
        let password = config.password.as_deref().unwrap_or("");

        let mut tiberius_cfg = Config::new();
        tiberius_cfg.host(host);
        tiberius_cfg.port(port);
        tiberius_cfg.authentication(AuthMethod::sql_server(user, password));
        tiberius_cfg.trust_cert();
        tiberius_cfg.database(database);

        let mgr = ConnectionManager::build(tiberius_cfg)
            .map_err(|e| AppError::ConnectionError(format!("MSSQL config error: {}", e)))?;

        let pool = Pool::builder()
            .max_size(5)
            .build(mgr)
            .await
            .map_err(|e| AppError::ConnectionError(format!("MSSQL pool creation failed: {}", e)))?;

        // Test immediate connection in separate scope
        {
            let mut conn = pool.get().await
                .map_err(|e| AppError::ConnectionError(format!("MSSQL connection failed: {}", e)))?;
            conn.simple_query("SELECT 1").await
                .map_err(|e| AppError::ConnectionError(format!("MSSQL ping failed: {}", e)))?;
        }

        Ok(Self {
            pool,
            database_name: database.to_string(),
        })
    }
}

fn column_data_to_json(data: &ColumnData) -> serde_json::Value {
    match data {
        ColumnData::U8(v) => v.map(|n| serde_json::Value::Number(n.into())).unwrap_or(serde_json::Value::Null),
        ColumnData::I16(v) => v.map(|n| serde_json::Value::Number(n.into())).unwrap_or(serde_json::Value::Null),
        ColumnData::I32(v) => v.map(|n| serde_json::Value::Number(n.into())).unwrap_or(serde_json::Value::Null),
        ColumnData::I64(v) => v.map(|n| serde_json::Value::Number(n.into())).unwrap_or(serde_json::Value::Null),
        ColumnData::F32(v) => v.and_then(|n| serde_json::Number::from_f64(n as f64).map(serde_json::Value::Number)).unwrap_or(serde_json::Value::Null),
        ColumnData::F64(v) => v.and_then(|n| serde_json::Number::from_f64(n).map(serde_json::Value::Number)).unwrap_or(serde_json::Value::Null),
        ColumnData::Bit(v) => v.map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null),
        ColumnData::String(v) => v.as_deref().map(|s| serde_json::Value::String(s.to_string())).unwrap_or(serde_json::Value::Null),
        ColumnData::Guid(v) => v.map(|g| serde_json::Value::String(g.to_string())).unwrap_or(serde_json::Value::Null),
        ColumnData::Binary(v) => v.as_deref().map(|b| serde_json::Value::String(format!("<binary {} bytes>", b.len()))).unwrap_or(serde_json::Value::Null),
        ColumnData::Numeric(v) => v.map(|n| serde_json::Value::String(n.to_string())).unwrap_or(serde_json::Value::Null),
        ColumnData::DateTime(v) => v.map(|dt| serde_json::Value::String(format!("{:?}", dt))).unwrap_or(serde_json::Value::Null),
        ColumnData::SmallDateTime(v) => v.map(|dt| serde_json::Value::String(format!("{:?}", dt))).unwrap_or(serde_json::Value::Null),
        ColumnData::Time(v) => v.map(|t| serde_json::Value::String(format!("{:?}", t))).unwrap_or(serde_json::Value::Null),
        ColumnData::Date(v) => v.map(|d| serde_json::Value::String(format!("{:?}", d))).unwrap_or(serde_json::Value::Null),
        ColumnData::DateTime2(v) => v.map(|dt| serde_json::Value::String(format!("{:?}", dt))).unwrap_or(serde_json::Value::Null),
        ColumnData::DateTimeOffset(v) => v.map(|dto| serde_json::Value::String(format!("{:?}", dto))).unwrap_or(serde_json::Value::Null),
        ColumnData::Xml(v) => v.as_deref().map(|x| serde_json::Value::String(x.to_string())).unwrap_or(serde_json::Value::Null),
    }
}

#[async_trait]
impl DatabaseAdapter for MssqlAdapter {
    async fn ping(&self) -> Result<(), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::ConnectionError(e.to_string()))?;
        conn.simple_query("SELECT 1").await
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
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::ConnectionError(e.to_string()))?;

        let stream = conn.simple_query(sql).await
            .map_err(|e| AppError::QueryError(e.to_string()))?;

        let rows = stream.into_first_result().await
            .map_err(|e| AppError::QueryError(e.to_string()))?;

        let duration = start.elapsed().as_secs_f64() * 1000.0;

        let mut columns = Vec::new();
        if let Some(first_row) = rows.first() {
            for col in first_row.columns() {
                let name = col.name().to_string();
                let is_primary_key = name.eq_ignore_ascii_case("id");
                columns.push(ColumnMetadata {
                    name,
                    data_type: format!("{:?}", col.column_type()),
                    is_primary_key,
                    is_foreign_key: false,
                    nullable: true,
                });
            }
        }

        let mut result_rows = Vec::new();
        for row in rows {
            let row_vals: Vec<serde_json::Value> = row.into_iter().map(|d| column_data_to_json(&d)).collect();
            result_rows.push(row_vals);
        }

        let affected = result_rows.len() as u64;

        Ok(QueryResult {
            columns,
            rows: result_rows,
            affected_rows: affected,
            execution_time_ms: duration,
            total_rows: Some(affected),
        })
    }

    async fn fetch_schema_tree(&self) -> Result<SchemaTree, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::ConnectionError(e.to_string()))?;

        // 1. Fetch available databases
        let db_stream = conn.simple_query("SELECT name FROM sys.databases WHERE state_desc = 'ONLINE' ORDER BY name").await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let db_rows = db_stream.into_first_result().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut databases = Vec::new();
        for row in db_rows {
            if let Some(name) = row.get::<&str, _>("name") {
                databases.push(name.to_string());
            }
        }
        if databases.is_empty() {
            databases.push(self.database_name.clone());
        }

        // 2. Fetch tables and views for current database
        let tbl_sql = r#"
            SELECT 
                COALESCE(TABLE_SCHEMA, 'dbo') AS table_schema,
                COALESCE(TABLE_NAME, '') AS table_name,
                COALESCE(TABLE_TYPE, 'BASE TABLE') AS table_type
            FROM INFORMATION_SCHEMA.TABLES
            ORDER BY TABLE_SCHEMA, TABLE_NAME;
        "#;

        let tbl_stream = conn.simple_query(tbl_sql).await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let tbl_rows = tbl_stream.into_first_result().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut tables = Vec::new();
        for row in tbl_rows {
            let schema = row.get::<&str, _>("table_schema").unwrap_or("dbo").to_string();
            let name = row.get::<&str, _>("table_name").unwrap_or_default().to_string();
            let table_type = row.get::<&str, _>("table_type").unwrap_or("BASE TABLE").to_string();

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
            databases,
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
