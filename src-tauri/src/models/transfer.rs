use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "config", rename_all = "camelCase")]
pub enum TransferSource {
    File { path: String, format: TransferFormat },
    Table { connection_id: String, schema: Option<String>, table: String },
    Query { connection_id: String, sql: String },
    Url { url: String, format: TransferFormat },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "config", rename_all = "camelCase")]
pub enum TransferTarget {
    File { path: String, format: TransferFormat },
    Table { connection_id: String, schema: Option<String>, table: String, conflict_strategy: Option<ConflictStrategy> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TransferFormat {
    Csv { 
        delimiter: Option<String>, 
        has_header: Option<bool>, 
        quote_char: Option<char> 
    },
    Tsv { 
        has_header: Option<bool> 
    },
    Json { 
        is_ndjson: Option<bool>, 
        pretty: Option<bool> 
    },
    Excel { 
        sheet_name: Option<String> 
    },
    SqlDump { 
        include_ddl: Option<bool>, 
        batch_size: Option<usize> 
    },
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
pub struct ExportJobRequest {
    pub connection_id: String,
    pub schema: Option<String>,
    pub table: Option<String>,
    pub query: Option<String>,
    pub target_path: String,
    pub format: TransferFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportJobRequest {
    pub connection_id: String,
    pub schema: Option<String>,
    pub table: String,
    pub source_path: String,
    pub format: TransferFormat,
    pub conflict_strategy: Option<ConflictStrategy>,
    pub create_table_if_missing: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DbToDbTransferRequest {
    pub source_connection_id: String,
    pub source_schema: Option<String>,
    pub source_table: Option<String>,
    pub source_query: Option<String>,
    pub target_connection_id: String,
    pub target_schema: Option<String>,
    pub target_table: String,
    pub conflict_strategy: Option<ConflictStrategy>,
    pub create_table_if_missing: Option<bool>,
    pub truncate_target_first: Option<bool>,
    pub batch_size: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferProgressEvent {
    pub job_id: String,
    pub rows_processed: u64,
    pub total_rows_estimated: Option<u64>,
    pub bytes_processed: u64,
    pub rows_per_second: f64,
    pub estimated_seconds_remaining: Option<u64>,
    pub percentage: Option<f64>,
    pub status: String, // "running", "completed", "cancelled", "failed"
    pub message: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileInspectionResult {
    pub detected_format: String,
    pub delimiter: Option<String>,
    pub has_header: bool,
    pub columns: Vec<String>,
    pub sample_rows: Vec<Vec<serde_json::Value>>,
    pub total_bytes: u64,
    pub sheet_names: Option<Vec<String>>,
}
