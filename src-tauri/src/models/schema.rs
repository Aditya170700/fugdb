use serde::{Deserialize, Serialize};
use crate::models::query::ColumnMetadata;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TableItem {
    pub schema: String,
    pub name: String,
    pub table_type: String, // "table" | "view" | "materialized_view"
    pub row_count_estimate: Option<i64>,
    #[serde(default)]
    pub columns: Vec<ColumnMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaTree {
    pub databases: Vec<String>,
    pub current_database: String,
    pub tables: Vec<TableItem>,
    #[serde(default)]
    pub relations: Vec<RelationEdge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelationEdge {
    pub id: String,
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
}
