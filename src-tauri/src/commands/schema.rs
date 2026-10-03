use tauri::State;
use crate::error::AppError;
use crate::models::schema::{RelationEdge, SchemaTree};
use crate::schema::dictionary::{
    generate_html_dictionary, generate_json_dictionary, generate_markdown_dictionary,
};
use crate::state::AppState;
use tokio::fs;

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

#[tauri::command]
pub async fn generate_data_dictionary(
    connection_id: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let tree = adapter.fetch_schema_tree().await?;
    let conn_name = connection_id.clone();
    let driver = "Database Engine";

    let content = match format.to_lowercase().as_str() {
        "html" => generate_html_dictionary(&tree, &conn_name, driver),
        "json" => generate_json_dictionary(&tree, &conn_name, driver),
        _ => generate_markdown_dictionary(&tree, &conn_name, driver),
    };

    Ok(content)
}

#[tauri::command]
pub async fn export_data_dictionary_file(
    connection_id: String,
    format: String,
    target_path: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let content = generate_data_dictionary(connection_id, format, state).await?;
    fs::write(&target_path, &content)
        .await
        .map_err(AppError::IoError)?;

    Ok(target_path)
}
