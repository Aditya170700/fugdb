use tauri::State;
use crate::error::AppError;
use crate::models::schema::{RelationEdge, SchemaTree};
use crate::state::AppState;

#[tauri::command]
pub async fn fetch_schema_tree(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<SchemaTree, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    adapter.fetch_schema_tree().await
}

#[tauri::command]
pub async fn generate_erd_metadata(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<RelationEdge>, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    adapter.generate_erd_metadata().await
}
