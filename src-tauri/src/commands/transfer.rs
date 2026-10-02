use tauri::{AppHandle, State};
use crate::error::AppError;
use crate::models::transfer::{DbToDbTransferRequest, ExportJobRequest, FileInspectionResult, ImportJobRequest};
use crate::state::AppState;
use crate::transfer::{inspect_file as detect_file, run_db_to_db_transfer_job, run_export_job, run_import_job};

#[tauri::command]
pub async fn inspect_file(path: String) -> Result<FileInspectionResult, AppError> {
    detect_file(&path)
}

#[tauri::command]
pub async fn start_export_job(
    request: ExportJobRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let job_id = format!("export_{}_{}", request.connection_id, chrono::Utc::now().timestamp_millis());
    let adapter = {
        let pools = state.pools.read().await;
        pools
            .get(&request.connection_id)
            .cloned()
            .ok_or_else(|| AppError::ConnectionNotFound(request.connection_id.clone()))?
    };

    let cancel_flag = state.job_manager.register_job(&job_id).await;
    let job_id_clone = job_id.clone();
    let job_manager = state.job_manager.clone();

    tokio::spawn(async move {
        let res = run_export_job(app, job_id_clone.clone(), adapter, request, cancel_flag).await;
        if let Err(e) = res {
            eprintln!("Export job error: {:?}", e);
        }
        job_manager.unregister_job(&job_id_clone).await;
    });

    Ok(job_id)
}

#[tauri::command]
pub async fn start_import_job(
    request: ImportJobRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let job_id = format!("import_{}_{}", request.connection_id, chrono::Utc::now().timestamp_millis());
    let adapter = {
        let pools = state.pools.read().await;
        pools
            .get(&request.connection_id)
            .cloned()
            .ok_or_else(|| AppError::ConnectionNotFound(request.connection_id.clone()))?
    };

    let cancel_flag = state.job_manager.register_job(&job_id).await;
    let job_id_clone = job_id.clone();
    let job_manager = state.job_manager.clone();

    tokio::spawn(async move {
        let res = run_import_job(app, job_id_clone.clone(), adapter, request, cancel_flag).await;
        if let Err(e) = res {
            eprintln!("Import job error: {:?}", e);
        }
        job_manager.unregister_job(&job_id_clone).await;
    });

    Ok(job_id)
}

#[tauri::command]
pub async fn start_db_to_db_job(
    request: DbToDbTransferRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let job_id = format!("db2db_{}_{}_{}", request.source_connection_id, request.target_connection_id, chrono::Utc::now().timestamp_millis());
    
    let (source_adapter, target_adapter) = {
        let pools = state.pools.read().await;
        let src = pools
            .get(&request.source_connection_id)
            .cloned()
            .ok_or_else(|| AppError::ConnectionNotFound(request.source_connection_id.clone()))?;
        let tgt = pools
            .get(&request.target_connection_id)
            .cloned()
            .ok_or_else(|| AppError::ConnectionNotFound(request.target_connection_id.clone()))?;
        (src, tgt)
    };

    let cancel_flag = state.job_manager.register_job(&job_id).await;
    let job_id_clone = job_id.clone();
    let job_manager = state.job_manager.clone();

    tokio::spawn(async move {
        let res = run_db_to_db_transfer_job(app, job_id_clone.clone(), source_adapter, target_adapter, request, cancel_flag).await;
        if let Err(e) = res {
            eprintln!("DB-to-DB transfer error: {:?}", e);
        }
        job_manager.unregister_job(&job_id_clone).await;
    });

    Ok(job_id)
}

#[tauri::command]
pub async fn cancel_transfer_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    Ok(state.job_manager.cancel_job(&job_id).await)
}
