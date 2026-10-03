use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleJobType {
    QueryExport,
    DatabaseBackup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleRunStatus {
    Pending,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledQueryConfig {
    pub sql: String,
    pub format: String, // "csv", "json", "tsv", "excel", "ndjson"
    pub target_dir: String,
    pub filename_pattern: String, // e.g. "{name}_{timestamp}.{ext}"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledBackupConfig {
    pub tables: Vec<String>, // empty = all tables
    pub include_schema: bool,
    pub include_data: bool,
    pub target_dir: String,
    pub filename_pattern: String, // e.g. "backup_{db}_{timestamp}.sql"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleJob {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub job_type: ScheduleJobType,
    pub connection_id: String,
    pub database: Option<String>,
    pub cron_expression: String,
    pub frequency_display: String,
    pub query_config: Option<ScheduledQueryConfig>,
    pub backup_config: Option<ScheduledBackupConfig>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_run_at: Option<i64>,
    pub last_run_status: Option<ScheduleRunStatus>,
    pub last_run_duration_ms: Option<u64>,
    pub last_run_error: Option<String>,
    pub last_run_file: Option<String>,
    pub last_run_rows: Option<u64>,
    pub last_run_bytes: Option<u64>,
    pub next_run_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleLog {
    pub id: String,
    pub job_id: String,
    pub job_name: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<u64>,
    pub status: ScheduleRunStatus,
    pub file_path: Option<String>,
    pub rows_processed: Option<u64>,
    pub bytes_written: Option<u64>,
    pub error_message: Option<String>,
}
