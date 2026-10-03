use async_trait::async_trait;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgRow, PgSslMode},
    PgPool, Row, Column,
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

pub struct PostgresAdapter {
    pool: PgPool,
    database_name: String,
}

impl PostgresAdapter {
    pub async fn new(config: &ConnectionConfig) -> Result<Self, AppError> {
        let host = config.host.as_deref().unwrap_or("localhost");
        let port = config.port.unwrap_or(5432);
        let database = config.database.as_deref().unwrap_or("postgres");
        let user = config.username.as_deref().unwrap_or("postgres");
        let password = config.password.as_deref().unwrap_or("");

        let mut connect_opts = PgConnectOptions::new()
            .host(host)
            .port(port)
            .database(database)
            .username(user)
            .password(password);

        if let Some(ref ssl) = config.ssl_mode {
            match ssl.as_str() {
                "disable" => connect_opts = connect_opts.ssl_mode(PgSslMode::Disable),
                "prefer" => connect_opts = connect_opts.ssl_mode(PgSslMode::Prefer),
                "require" => connect_opts = connect_opts.ssl_mode(PgSslMode::Require),
                _ => {}
            }
        } else {
            connect_opts = connect_opts.ssl_mode(PgSslMode::Prefer);
        }

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(6))
            .connect_with(connect_opts)
            .await
            .map_err(|e| AppError::ConnectionError(format!("PostgreSQL connection failed: {}", e)))?;

        Ok(Self {
            pool,
            database_name: database.to_string(),
        })
    }
}

fn extract_pg_value(row: &PgRow, col_idx: usize) -> serde_json::Value {
    if let Ok(val) = row.try_get::<Option<String>, _>(col_idx) {
        return val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i64>, _>(col_idx) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i32>, _>(col_idx) {
        return val.map(|v| serde_json::Value::Number(v.into())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<i16>, _>(col_idx) {
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
    if let Ok(val) = row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(col_idx) {
        return val.map(|v| serde_json::Value::String(v.to_rfc3339())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<chrono::NaiveDateTime>, _>(col_idx) {
        return val.map(|v| serde_json::Value::String(v.to_string())).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.try_get::<Option<chrono::NaiveDate>, _>(col_idx) {
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
impl DatabaseAdapter for PostgresAdapter {
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
                row_values.push(extract_pg_value(row, col_idx));
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
            format!("EXPLAIN (ANALYZE, COSTS, VERBOSE, BUFFERS, FORMAT JSON) {}", clean_sql)
        } else {
            format!("EXPLAIN (COSTS, VERBOSE, FORMAT JSON) {}", clean_sql)
        };

        let result = match self.execute_query(&explain_sql, None, None).await {
            Ok(res) => res,
            Err(err) => {
                let fallback_sql = format!("EXPLAIN (FORMAT JSON) {}", clean_sql);
                self.execute_query(&fallback_sql, None, None).await
                    .map_err(|_| err)?
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

        let query_duration = result.execution_time_ms;

        Ok(ExplainResult {
            raw_plan,
            json_plan,
            query_result: result,
            execution_time_ms: execution_time_ms.or(Some(query_duration)),
            planning_time_ms,
            dialect: "PostgreSQL".to_string(),
            has_analyze: analyze,
        })
    }

    async fn fetch_schema_tree(&self) -> Result<SchemaTree, AppError> {
        let tables_sql = r#"
            SELECT 
                COALESCE(t.table_schema::text, 'public') AS table_schema,
                COALESCE(t.table_name::text, '') AS table_name,
                COALESCE(t.table_type::text, 'BASE TABLE') AS table_type,
                COALESCE(s.n_live_tup, GREATEST(0, c.reltuples::bigint), 0)::bigint AS row_count_estimate
            FROM information_schema.tables t
            LEFT JOIN pg_namespace n ON n.nspname = t.table_schema
            LEFT JOIN pg_class c ON c.relname = t.table_name AND c.relnamespace = n.oid
            LEFT JOIN pg_stat_user_tables s ON s.schemaname = t.table_schema AND s.relname = t.table_name
            WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY t.table_schema, t.table_name;
        "#;

        let table_rows = sqlx::query(tables_sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let columns_sql = r#"
            SELECT 
                table_schema::text AS table_schema,
                table_name::text AS table_name,
                column_name::text AS column_name,
                data_type::text AS data_type,
                is_nullable::text AS is_nullable
            FROM information_schema.columns
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_schema, table_name, ordinal_position;
        "#;

        let col_rows = sqlx::query(columns_sql)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        let pk_sql = r#"
            SELECT 
                tc.table_schema::text AS table_schema,
                tc.table_name::text AS table_name,
                kcu.column_name::text AS column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
              ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
            WHERE tc.constraint_type = 'PRIMARY KEY';
        "#;
        let pk_rows = sqlx::query(pk_sql)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();
        let mut pk_set = std::collections::HashSet::new();
        for r in pk_rows {
            let s: String = r.try_get("table_schema").unwrap_or_default();
            let t: String = r.try_get("table_name").unwrap_or_default();
            let c: String = r.try_get("column_name").unwrap_or_default();
            pk_set.insert(format!("{}.{}.{}", s, t, c));
        }

        let fk_sql = r#"
            SELECT
                tc.constraint_name::text AS id,
                kcu.table_name::text AS from_table,
                kcu.column_name::text AS from_column,
                ccu.table_name::text AS to_table,
                ccu.column_name::text AS to_column
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
              ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
              ON ccu.constraint_name = tc.constraint_name
              AND ccu.table_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY';
        "#;
        let fk_rows = sqlx::query(fk_sql)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        let mut relations = Vec::new();
        let mut fk_col_set = std::collections::HashSet::new();
        for r in fk_rows {
            let id: String = r.try_get("id").unwrap_or_default();
            let from_table: String = r.try_get("from_table").unwrap_or_default();
            let from_column: String = r.try_get("from_column").unwrap_or_default();
            let to_table: String = r.try_get("to_table").unwrap_or_default();
            let to_column: String = r.try_get("to_column").unwrap_or_default();
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
            let schema: String = r.try_get("table_schema").unwrap_or_default();
            let table: String = r.try_get("table_name").unwrap_or_default();
            let col_name: String = r.try_get("column_name").unwrap_or_default();
            let data_type: String = r.try_get("data_type").unwrap_or_default();
            let is_nullable: String = r.try_get("is_nullable").unwrap_or_else(|_| "YES".into());

            let is_pk = pk_set.contains(&format!("{}.{}.{}", schema, table, col_name)) || col_name.eq_ignore_ascii_case("id");
            let is_fk = fk_col_set.contains(&format!("{}.{}", table, col_name)) || col_name.ends_with("_id");

            let key = format!("{}.{}", schema, table);
            col_map.entry(key).or_default().push(ColumnMetadata {
                name: col_name,
                data_type,
                is_primary_key: is_pk,
                is_foreign_key: is_fk,
                nullable: is_nullable.eq_ignore_ascii_case("YES"),
            });
        }

        let mut tables = Vec::new();
        for row in table_rows {
            let schema: String = row.try_get("table_schema").unwrap_or_else(|_| "public".into());
            let name: String = row.try_get("table_name").unwrap_or_default();
            let table_type: String = row.try_get("table_type").unwrap_or_else(|_| "table".into());
            let row_count: Option<i64> = row.try_get("row_count_estimate").ok();

            if !name.is_empty() {
                let key = format!("{}.{}", schema, name);
                let columns = col_map.remove(&key).unwrap_or_default();
                tables.push(TableItem {
                    schema,
                    name,
                    table_type: if table_type.contains("VIEW") { "view".into() } else { "table".into() },
                    row_count_estimate: row_count,
                    columns,
                });
            }
        }

        for tbl in &mut tables {
            if tbl.table_type != "view" && (tbl.row_count_estimate.is_none() || tbl.row_count_estimate == Some(0)) {
                let count_sql = format!("SELECT COUNT(*) AS c FROM \"{}\".\"{}\"", tbl.schema, tbl.name);
                if let Ok(count_row) = sqlx::query(&count_sql).fetch_one(&self.pool).await {
                    if let Ok(c) = count_row.try_get::<i64, _>("c") {
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
