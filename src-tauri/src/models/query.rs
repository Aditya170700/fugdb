use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    pub connection_id: String,
    pub sql: String,
    pub page_size: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub nullable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Vec<serde_json::Value>>, // 2D array for optimal Tauri IPC serialization
    pub affected_rows: u64,
    pub execution_time_ms: f64,
    pub total_rows: Option<u64>,
}

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

