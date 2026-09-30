use tauri::State;
use crate::error::AppError;
use crate::models::query::QueryResult;
use crate::state::AppState;

#[tauri::command]
pub async fn execute_query(
    connection_id: String,
    sql: String,
    page_size: Option<u64>,
    offset: Option<u64>,
    state: State<'_, AppState>,
) -> Result<QueryResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    adapter.execute_query(&sql, page_size, offset).await
}
