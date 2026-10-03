use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffAction {
    Create,
    Drop,
    Alter,
    Identical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDiff {
    pub name: String,
    pub action: DiffAction,
    pub source_type: Option<String>,
    pub target_type: Option<String>,
    pub source_nullable: Option<bool>,
    pub target_nullable: Option<bool>,
    pub source_pk: Option<bool>,
    pub target_pk: Option<bool>,
    pub diff_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableDiff {
    pub table_name: String,
    pub action: DiffAction,
    pub source_row_count: Option<u64>,
    pub target_row_count: Option<u64>,
    pub columns: Vec<ColumnDiff>,
    pub sync_sql: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDiffResult {
    pub source_connection_id: String,
    pub target_connection_id: String,
    pub source_driver: String,
    pub target_driver: String,
    pub total_source_tables: usize,
    pub total_target_tables: usize,
    pub tables_to_create: usize,
    pub tables_to_drop: usize,
    pub tables_to_alter: usize,
    pub tables_identical: usize,
    pub table_diffs: Vec<TableDiff>,
    pub full_migration_sql: String,
    pub execution_time_ms: f64,
}
