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

fn resolve_dialect(driver: Option<&str>, database_name: &str) -> &'static str {
    match driver.unwrap_or("").to_lowercase().as_str() {
        "sqlite" => "SQLite",
        "mysql" => "MySQL",
        "mssql" | "sqlserver" => "Microsoft SQL Server (T-SQL)",
        "duckdb" => "DuckDB",
        "postgres" | "postgresql" => "PostgreSQL",
        _ => {
            if database_name.ends_with(".db") || database_name.ends_with(".sqlite") {
                "SQLite"
            } else {
                "PostgreSQL"
            }
        }
    }
}

#[tauri::command]
pub async fn generate_sql_from_prompt(
    connection_id: String,
    config: AiProviderConfig,
    user_prompt: String,
    selected_tables: Option<Vec<String>>,
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<AiSqlResponse, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    // Fetch live schema tree (metadata only, zero row data leak)
    let schema_tree = adapter.fetch_schema_tree().await?;
    let dialect = resolve_dialect(driver.as_deref(), &schema_tree.current_database);

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
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<AiSqlResponse, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let schema_tree = adapter.fetch_schema_tree().await?;
    let dialect = resolve_dialect(driver.as_deref(), &schema_tree.current_database);

    let schema_context = build_schema_context(&schema_tree, dialect, None);

    let system_prompt = format!(
        "{}\n\nDATABASE SCHEMA CONTEXT:\n{}",
        build_system_prompt(dialect),
        schema_context
    );

    let fix_prompt = format!(
        "The following SQL query failed with an error in {} database:\n\nFAILED QUERY:\n```sql\n{}\n```\n\nDATABASE ERROR MESSAGE:\n{}\n\nPlease fix the query so it runs successfully in {} dialect.\nIn your explanation:\n1. State the exact root cause of the error.\n2. Explain what changes were made in the fixed query.\n3. Mention any relevant {} syntax nuances.",
        dialect, sql, error_message, dialect, dialect
    );

    AiClient::generate_sql(&config, &system_prompt, &fix_prompt, dialect).await
}
