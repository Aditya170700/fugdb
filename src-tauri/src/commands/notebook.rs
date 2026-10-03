use tokio::fs;
use crate::error::AppError;

#[tauri::command]
pub async fn save_fugpad_file(path: String, content: String) -> Result<(), AppError> {
    fs::write(&path, content)
        .await
        .map_err(AppError::IoError)?;
    Ok(())
}

#[tauri::command]
pub async fn read_fugpad_file(path: String) -> Result<String, AppError> {
    let content = fs::read_to_string(&path)
        .await
        .map_err(AppError::IoError)?;
    Ok(content)
}

#[tauri::command]
pub async fn export_notebook_html_file(target_path: String, html_content: String) -> Result<String, AppError> {
    fs::write(&target_path, html_content)
        .await
        .map_err(AppError::IoError)?;
    Ok(target_path)
}
