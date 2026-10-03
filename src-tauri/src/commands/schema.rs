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

#[tauri::command]
pub async fn save_image_file(
    target_path: String,
    base64_data: String,
) -> Result<String, AppError> {
    let clean_base64 = if let Some(idx) = base64_data.find(',') {
        &base64_data[idx + 1..]
    } else {
        &base64_data
    };

    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(clean_base64.trim())
        .map_err(|e| AppError::Internal(format!("Base64 decode error: {}", e)))?;

    fs::write(&target_path, &bytes)
        .await
        .map_err(AppError::IoError)?;

    Ok(target_path)
}

#[tauri::command]
pub async fn print_data_dictionary(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let tree = adapter.fetch_schema_tree().await?;
    let conn_name = connection_id.clone();
    let driver = "Database Engine";

    let mut html = generate_html_dictionary(&tree, &conn_name, driver);

    // Inject auto-print script before </body>
    let auto_print_script = r#"
  <script>
    window.addEventListener('load', function() {
      setTimeout(function() {
        window.print();
      }, 400);
    });
  </script>
</body>"#;
    html = html.replace("</body>", auto_print_script);

    let clean_name = conn_name.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let temp_file = std::env::temp_dir().join(format!("fugdb_dictionary_{}.html", clean_name));

    fs::write(&temp_file, &html)
        .await
        .map_err(AppError::IoError)?;

    let path_str = temp_file.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&path_str)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path_str])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn();
    }

    Ok(path_str)
}

#[tauri::command]
pub async fn compare_schemas(
    source_connection_id: String,
    target_connection_id: String,
    source_driver: Option<String>,
    target_driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<crate::models::diff::SchemaDiffResult, AppError> {
    let pools = state.pools.read().await;
    let src_adapter = pools
        .get(&source_connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(source_connection_id.clone()))?;

    let tgt_adapter = pools
        .get(&target_connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(target_connection_id.clone()))?;

    let src_tree = src_adapter.fetch_schema_tree().await?;
    let tgt_tree = tgt_adapter.fetch_schema_tree().await?;

    let src_drv = source_driver.unwrap_or_else(|| "postgres".to_string());
    let tgt_drv = target_driver.unwrap_or_else(|| "postgres".to_string());

    let result = crate::schema::diff::generate_schema_diff(
        &src_tree,
        &tgt_tree,
        &source_connection_id,
        &target_connection_id,
        &src_drv,
        &tgt_drv,
    );

    Ok(result)
}

#[tauri::command]
pub async fn apply_migration_script(
    target_connection_id: String,
    sql: String,
    state: State<'_, AppState>,
) -> Result<crate::models::query::QueryResult, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&target_connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(target_connection_id.clone()))?;

    adapter.execute_query(&sql, None, None).await
}

