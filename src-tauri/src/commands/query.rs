use tauri::State;
use crate::error::AppError;
use crate::models::query::{QueryResult, ExplainResult};
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

#[tauri::command]
pub async fn explain_query(
    connection_id: String,
    sql: String,
    analyze: bool,
    _driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<ExplainResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    adapter.explain_query(&sql, analyze).await
}

#[tauri::command]
pub async fn begin_transaction(
    connection_id: String,
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<QueryResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let sql = match driver.as_deref().unwrap_or("").to_lowercase().as_str() {
        "mysql" => "START TRANSACTION;",
        "mssql" | "sqlserver" => "BEGIN TRANSACTION;",
        _ => "BEGIN;",
    };

    adapter.execute_query(sql, None, None).await
}

#[tauri::command]
pub async fn commit_transaction(
    connection_id: String,
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<QueryResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let sql = match driver.as_deref().unwrap_or("").to_lowercase().as_str() {
        "mssql" | "sqlserver" => "COMMIT TRANSACTION;",
        _ => "COMMIT;",
    };

    adapter.execute_query(sql, None, None).await
}

#[tauri::command]
pub async fn rollback_transaction(
    connection_id: String,
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<QueryResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let sql = match driver.as_deref().unwrap_or("").to_lowercase().as_str() {
        "mssql" | "sqlserver" => "ROLLBACK TRANSACTION;",
        _ => "ROLLBACK;",
    };

    adapter.execute_query(sql, None, None).await
}


