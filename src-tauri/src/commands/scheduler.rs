use chrono::Local;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::models::scheduler::{ScheduleJob, ScheduleLog};
use crate::scheduler::{calculate_next_run, open_folder_in_os};
use crate::state::AppState;

#[tauri::command]
pub async fn list_schedules(state: State<'_, AppState>) -> Result<Vec<ScheduleJob>, AppError> {
    Ok(state.scheduler.load_jobs())
}

#[tauri::command]
pub async fn create_or_update_schedule(
    job: ScheduleJob,
    state: State<'_, AppState>,
) -> Result<ScheduleJob, AppError> {
    let mut jobs = state.scheduler.load_jobs();
    let now = Local::now().timestamp_millis();

    let mut updated_job = job;
    updated_job.updated_at = now;

    // Calculate next run if enabled
    if updated_job.enabled && updated_job.next_run_at.is_none() {
        updated_job.next_run_at = Some(calculate_next_run(&updated_job.cron_expression, now));
    }

    if let Some(pos) = jobs.iter().position(|j| j.id == updated_job.id) {
        jobs[pos] = updated_job.clone();
    } else {
        if updated_job.created_at == 0 {
            updated_job.created_at = now;
        }
        jobs.push(updated_job.clone());
    }

    state.scheduler.save_jobs(&jobs)?;
    Ok(updated_job)
}

#[tauri::command]
pub async fn delete_schedule(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut jobs = state.scheduler.load_jobs();
    jobs.retain(|j| j.id != id);
    state.scheduler.save_jobs(&jobs)?;
    let _ = state.scheduler.clear_logs(Some(&id));
    Ok(())
}

#[tauri::command]
pub async fn toggle_schedule_enabled(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<ScheduleJob, AppError> {
    let mut jobs = state.scheduler.load_jobs();
    let pos = jobs.iter().position(|j| j.id == id)
        .ok_or_else(|| AppError::InternalError(format!("Job not found: {}", id)))?;

    let now = Local::now().timestamp_millis();
    jobs[pos].enabled = enabled;
    jobs[pos].updated_at = now;
    if enabled {
        jobs[pos].next_run_at = Some(calculate_next_run(&jobs[pos].cron_expression, now));
    } else {
        jobs[pos].next_run_at = None;
    }

    let updated = jobs[pos].clone();
    state.scheduler.save_jobs(&jobs)?;
    Ok(updated)
}

#[tauri::command]
pub async fn run_schedule_now(
    app: AppHandle,
    id: String,
    state: State<'_, AppState>,
) -> Result<ScheduleLog, AppError> {
    state.scheduler.execute_job(&app, &state.pools, &id).await
}

#[tauri::command]
pub async fn get_schedule_logs(
    job_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ScheduleLog>, AppError> {
    Ok(state.scheduler.get_logs(job_id.as_deref()))
}

#[tauri::command]
pub async fn clear_schedule_logs(
    job_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.scheduler.clear_logs(job_id.as_deref())
}

#[tauri::command]
pub async fn open_schedule_output_folder(path: String) -> Result<(), AppError> {
    open_folder_in_os(&path)
}

#[tauri::command]
pub async fn get_default_backup_directory() -> Result<String, AppError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let default_dir = std::path::PathBuf::from(home).join("Downloads");
    Ok(default_dir.to_string_lossy().to_string())
}
