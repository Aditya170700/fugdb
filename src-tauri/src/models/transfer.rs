use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "config", rename_all = "camelCase")]
pub enum TransferSource {
    File { path: String, format: TransferFormat },
    Table { connection_id: String, schema: String, table: String },
    Query { connection_id: String, sql: String },
    Url { url: String, format: TransferFormat },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "config", rename_all = "camelCase")]
pub enum TransferTarget {
    File { path: String, format: TransferFormat },
    Table { connection_id: String, schema: String, table: String, conflict_strategy: ConflictStrategy },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TransferFormat {
    Csv { delimiter: char, has_header: bool },
    Json { is_ndjson: bool },
    Excel { sheet_name: Option<String> },
    Parquet,
    SqlDump { include_ddl: bool, batch_size: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ConflictStrategy {
    Fail,
    Ignore,
    Upsert { match_columns: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferProgressEvent {
    pub job_id: String,
    pub rows_processed: u64,
    pub bytes_processed: u64,
    pub rows_per_second: f64,
    pub estimated_seconds_remaining: Option<u64>,
    pub status: String,
    pub error_message: Option<String>,
}
