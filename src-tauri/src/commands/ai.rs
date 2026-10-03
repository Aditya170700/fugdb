use tauri::State;
use crate::error::AppError;
use crate::state::AppState;
use crate::ai::{
    AiClient, AiProviderConfig, AiSqlResponse, OllamaModelInfo,
    build_schema_context, build_system_prompt,
};

#[tauri::command]
pub async fn test_ai_connection(
    config: AiProviderConfig,
) -> Result<String, AppError> {
    AiClient::test_connection(&config).await
}

#[tauri::command]
pub async fn list_ollama_models(
    endpoint: Option<String>,
) -> Result<Vec<OllamaModelInfo>, AppError> {
    AiClient::list_ollama_models(endpoint.as_deref()).await
}

#[tauri::command]
pub async fn generate_sql_from_prompt(
    connection_id: String,
    config: AiProviderConfig,
    user_prompt: String,
    selected_tables: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<AiSqlResponse, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    // Fetch live schema tree (metadata only, zero row data leak)
    let schema_tree = adapter.fetch_schema_tree().await?;

    // Detect dialect based on schema/database properties
    let dialect = if schema_tree.current_database.ends_with(".db") || schema_tree.current_database.ends_with(".sqlite") {
        "SQLite"
    } else {
        "PostgreSQL"
    };

    let schema_context = build_schema_context(
        &schema_tree,
        dialect,
        selected_tables.as_deref(),
    );

    let system_prompt = format!(
        "{}\n\nDATABASE SCHEMA CONTEXT (ZERO DATA LEAK - STRUCTURE ONLY):\n{}",
        build_system_prompt(dialect),
        schema_context
    );

    AiClient::generate_sql(&config, &system_prompt, &user_prompt, dialect).await
}

#[tauri::command]
pub async fn fix_sql_error(
    connection_id: String,
    config: AiProviderConfig,
    sql: String,
    error_message: String,
    state: State<'_, AppState>,
) -> Result<AiSqlResponse, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let schema_tree = adapter.fetch_schema_tree().await?;
    let dialect = if schema_tree.current_database.ends_with(".db") || schema_tree.current_database.ends_with(".sqlite") {
        "SQLite"
    } else {
        "PostgreSQL"
    };

    let schema_context = build_schema_context(&schema_tree, dialect, None);

    let system_prompt = format!(
        "{}\n\nDATABASE SCHEMA CONTEXT:\n{}",
        build_system_prompt(dialect),
        schema_context
    );

    let fix_prompt = format!(
        "The following SQL query failed with an error:\n\nFAILED QUERY:\n```sql\n{}\n```\n\nDATABASE ERROR MESSAGE:\n{}\n\nPlease fix the query so it runs successfully in {} dialect and explain what caused the error and how you fixed it.",
        sql, error_message, dialect
    );

    AiClient::generate_sql(&config, &system_prompt, &fix_prompt, dialect).await
}
