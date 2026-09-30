pub mod postgres;
pub mod mysql;
pub mod sqlite;

use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{
    query::QueryResult,
    schema::{RelationEdge, SchemaTree},
    transfer::ConflictStrategy,
};
use tokio::sync::mpsc;

#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    async fn ping(&self) -> Result<(), AppError>;
    async fn execute_query(&self, sql: &str, page_size: Option<u64>, offset: Option<u64>) -> Result<QueryResult, AppError>;
    async fn fetch_schema_tree(&self) -> Result<SchemaTree, AppError>;
    async fn generate_erd_metadata(&self) -> Result<Vec<RelationEdge>, AppError>;
    async fn insert_mock_batch(&self, table: &str, count: u64) -> Result<u64, AppError>;
    
    // Streaming batch operations for ETL
    async fn stream_rows(&self, sql: &str, tx: mpsc::Sender<Vec<serde_json::Value>>) -> Result<(), AppError>;
    async fn batch_insert_rows(&self, table: &str, columns: &[String], rows: &[Vec<serde_json::Value>], strategy: &ConflictStrategy) -> Result<u64, AppError>;
}
