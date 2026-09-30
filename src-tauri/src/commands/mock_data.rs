use tauri::State;
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn generate_mock_batch(
    connection_id: String,
    table: String,
    count: u64,
    state: State<'_, AppState>,
) -> Result<u64, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    adapter.insert_mock_batch(&table, count).await
}
