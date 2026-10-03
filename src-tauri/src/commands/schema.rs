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
