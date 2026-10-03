use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use chrono::{DateTime, Datelike, Local, NaiveTime};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::scheduler::{
    ScheduleJob, ScheduleJobType, ScheduleLog, ScheduleRunStatus,
};

pub struct SchedulerManager {
    base_dir: PathBuf,
}

impl SchedulerManager {
    pub fn new() -> Self {
        let base_dir = dirs_base().join(".fugdb");
        let _ = fs::create_dir_all(&base_dir);
        Self { base_dir }
    }

    fn schedules_path(&self) -> PathBuf {
        self.base_dir.join("schedules.json")
    }

    fn logs_path(&self) -> PathBuf {
        self.base_dir.join("schedule_logs.json")
    }

    pub fn load_jobs(&self) -> Vec<ScheduleJob> {
        let path = self.schedules_path();
        if !path.exists() {
            return Vec::new();
        }
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    pub fn save_jobs(&self, jobs: &[ScheduleJob]) -> Result<(), AppError> {
        let path = self.schedules_path();
        let content = serde_json::to_string_pretty(jobs)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize schedules: {}", e)))?;
        fs::write(&path, content)
            .map_err(|e| AppError::InternalError(format!("Failed to save schedules file: {}", e)))?;
        Ok(())
    }

    pub fn get_logs(&self, job_id: Option<&str>) -> Vec<ScheduleLog> {
        let path = self.logs_path();
        if !path.exists() {
            return Vec::new();
        }
        let logs: Vec<ScheduleLog> = match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        if let Some(jid) = job_id {
            logs.into_iter().filter(|l| l.job_id == jid).collect()
        } else {
            logs
        }
    }

    pub fn append_log(&self, log: ScheduleLog) -> Result<(), AppError> {
        let mut logs = self.get_logs(None);
        logs.insert(0, log);
        if logs.len() > 200 {
            logs.truncate(200);
        }
        let path = self.logs_path();
        let content = serde_json::to_string_pretty(&logs)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize schedule logs: {}", e)))?;
        fs::write(&path, content)
            .map_err(|e| AppError::InternalError(format!("Failed to save schedule logs: {}", e)))?;
        Ok(())
    }

    pub fn clear_logs(&self, job_id: Option<&str>) -> Result<(), AppError> {
        let path = self.logs_path();
        if let Some(jid) = job_id {
            let mut logs = self.get_logs(None);
            logs.retain(|l| l.job_id != jid);
            let content = serde_json::to_string_pretty(&logs)
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            fs::write(&path, content).map_err(|e| AppError::InternalError(e.to_string()))?;
        } else {
            let empty: Vec<ScheduleLog> = Vec::new();
            let content = serde_json::to_string_pretty(&empty)
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            fs::write(&path, content).map_err(|e| AppError::InternalError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn execute_job(
        &self,
        app: &AppHandle,
        pools: &Arc<RwLock<HashMap<String, Arc<Box<dyn DatabaseAdapter>>>>>,
        job_id: &str,
    ) -> Result<ScheduleLog, AppError> {
        let mut jobs = self.load_jobs();
        let job_idx = jobs.iter().position(|j| j.id == job_id)
            .ok_or_else(|| AppError::InternalError(format!("Schedule job not found: {}", job_id)))?;

        let mut job = jobs[job_idx].clone();
        let start_time = Instant::now();
        let started_at = Local::now().timestamp_millis();

        // Emit job-started event
        let _ = app.emit("scheduler:job-started", serde_json::json!({
            "job_id": job.id,
            "job_name": job.name,
            "started_at": started_at,
        }));

        // Retrieve database adapter pool
        let adapter = {
            let pools_guard = pools.read().await;
            pools_guard.get(&job.connection_id).cloned()
        };

        let adapter = match adapter {
            Some(a) => a,
            None => {
                let err_msg = format!("Database connection '{}' is currently not connected in FugDB.", job.connection_id);
                let log = ScheduleLog {
                    id: format!("log_{}_{}", job.id, started_at),
                    job_id: job.id.clone(),
                    job_name: job.name.clone(),
                    started_at,
                    completed_at: Some(Local::now().timestamp_millis()),
                    duration_ms: Some(start_time.elapsed().as_millis() as u64),
                    status: ScheduleRunStatus::Failed,
                    file_path: None,
                    rows_processed: Some(0),
                    bytes_written: Some(0),
                    error_message: Some(err_msg.clone()),
                };

                let _ = self.append_log(log.clone());
                job.last_run_at = Some(started_at);
                job.last_run_status = Some(ScheduleRunStatus::Failed);
                job.last_run_error = Some(err_msg.clone());
                job.last_run_duration_ms = Some(start_time.elapsed().as_millis() as u64);
                job.next_run_at = Some(calculate_next_run(&job.cron_expression, Local::now().timestamp_millis()));
                jobs[job_idx] = job;
                let _ = self.save_jobs(&jobs);

                let _ = app.emit("scheduler:job-failed", serde_json::json!({
                    "job_id": job_id,
                    "error": err_msg,
                }));

                return Ok(log);
            }
        };

        // Perform execution based on job type
        let execution_result = match job.job_type {
            ScheduleJobType::QueryExport => {
                self.execute_query_export(&job, &adapter).await
            },
            ScheduleJobType::DatabaseBackup => {
                self.execute_database_backup(&job, &adapter).await
            },
        };

        let completed_at = Local::now().timestamp_millis();
        let duration_ms = start_time.elapsed().as_millis() as u64;

        let log = match execution_result {
            Ok((output_file, rows, bytes)) => {
                job.last_run_at = Some(started_at);
                job.last_run_status = Some(ScheduleRunStatus::Success);
                job.last_run_error = None;
                job.last_run_duration_ms = Some(duration_ms);
                job.last_run_file = Some(output_file.clone());
                job.last_run_rows = Some(rows);
                job.last_run_bytes = Some(bytes);
                job.next_run_at = Some(calculate_next_run(&job.cron_expression, completed_at));
                jobs[job_idx] = job.clone();
                let _ = self.save_jobs(&jobs);

                let log = ScheduleLog {
                    id: format!("log_{}_{}", job.id, started_at),
                    job_id: job.id.clone(),
                    job_name: job.name.clone(),
                    started_at,
                    completed_at: Some(completed_at),
                    duration_ms: Some(duration_ms),
                    status: ScheduleRunStatus::Success,
                    file_path: Some(output_file.clone()),
                    rows_processed: Some(rows),
                    bytes_written: Some(bytes),
                    error_message: None,
                };
                let _ = self.append_log(log.clone());

                let _ = app.emit("scheduler:job-completed", serde_json::json!({
                    "job_id": job.id,
                    "job_name": job.name,
                    "output_file": output_file,
                    "rows": rows,
                    "bytes": bytes,
                    "duration_ms": duration_ms,
                }));

                log
            },
            Err(err) => {
                let err_msg = err.to_string();
                job.last_run_at = Some(started_at);
                job.last_run_status = Some(ScheduleRunStatus::Failed);
                job.last_run_error = Some(err_msg.clone());
                job.last_run_duration_ms = Some(duration_ms);
                job.next_run_at = Some(calculate_next_run(&job.cron_expression, completed_at));
                jobs[job_idx] = job.clone();
                let _ = self.save_jobs(&jobs);

                let log = ScheduleLog {
                    id: format!("log_{}_{}", job.id, started_at),
                    job_id: job.id.clone(),
                    job_name: job.name.clone(),
                    started_at,
                    completed_at: Some(completed_at),
                    duration_ms: Some(duration_ms),
                    status: ScheduleRunStatus::Failed,
                    file_path: None,
                    rows_processed: Some(0),
                    bytes_written: Some(0),
                    error_message: Some(err_msg.clone()),
                };
                let _ = self.append_log(log.clone());

                let _ = app.emit("scheduler:job-failed", serde_json::json!({
                    "job_id": job.id,
                    "job_name": job.name,
                    "error": err_msg,
                }));

                log
            }
        };

        Ok(log)
    }

    async fn execute_query_export(
        &self,
        job: &ScheduleJob,
        adapter: &Arc<Box<dyn DatabaseAdapter>>,
    ) -> Result<(String, u64, u64), AppError> {
        let config = job.query_config.as_ref()
            .ok_or_else(|| AppError::InternalError("Missing query configuration for export job".into()))?;

        if config.sql.trim().is_empty() {
            return Err(AppError::InternalError("SQL query is empty".into()));
        }

        let query_result = adapter.execute_query(&config.sql, None, None).await?;
        let col_names: Vec<String> = query_result.columns.iter().map(|c| c.name.clone()).collect();
        let rows = query_result.rows;
        let row_count = rows.len() as u64;

        // Ensure target directory exists
        let target_dir = PathBuf::from(&config.target_dir);
        let _ = fs::create_dir_all(&target_dir);

        // Format target filename
        let now = Local::now();
        let ts_str = now.format("%Y%m%d_%H%M%S").to_string();
        let date_str = now.format("%Y%m%d").to_string();
        let clean_job_name = sanitize_filename(&job.name);
        let ext = match config.format.to_lowercase().as_str() {
            "json" => "json",
            "ndjson" => "ndjson",
            "tsv" => "tsv",
            "excel" | "xlsx" => "xlsx",
            _ => "csv",
        };

        let mut filename = config.filename_pattern.clone();
        if filename.is_empty() {
            filename = format!("{}_{}.{}", clean_job_name, ts_str, ext);
        } else {
            filename = filename
                .replace("{name}", &clean_job_name)
                .replace("{job_name}", &clean_job_name)
                .replace("{timestamp}", &ts_str)
                .replace("{date}", &date_str)
                .replace("{db}", job.database.as_deref().unwrap_or("db"))
                .replace("{ext}", ext);
            if !filename.ends_with(&format!(".{}", ext)) {
                filename.push_str(&format!(".{}", ext));
            }
        }

        let output_path = target_dir.join(&filename);
        let output_path_str = output_path.to_string_lossy().to_string();

        match ext {
            "csv" => {
                let file = File::create(&output_path)
                    .map_err(|e| AppError::InternalError(format!("Failed to create CSV file: {}", e)))?;
                let mut writer = csv::WriterBuilder::new()
                    .has_headers(true)
                    .from_writer(BufWriter::new(file));

                writer.write_record(&col_names)
                    .map_err(|e| AppError::InternalError(format!("CSV write header error: {}", e)))?;

                for row in &rows {
                    let record: Vec<String> = row.iter().map(|v| match v {
                        serde_json::Value::Null => "".to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    }).collect();
                    writer.write_record(&record)
                        .map_err(|e| AppError::InternalError(format!("CSV write record error: {}", e)))?;
                }
                writer.flush().map_err(|e| AppError::InternalError(format!("CSV flush error: {}", e)))?;
            },
            "tsv" => {
                let file = File::create(&output_path)
                    .map_err(|e| AppError::InternalError(format!("Failed to create TSV file: {}", e)))?;
                let mut writer = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(true)
                    .from_writer(BufWriter::new(file));

                writer.write_record(&col_names)
                    .map_err(|e| AppError::InternalError(format!("TSV write header error: {}", e)))?;

                for row in &rows {
                    let record: Vec<String> = row.iter().map(|v| match v {
                        serde_json::Value::Null => "".to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    }).collect();
                    writer.write_record(&record)
                        .map_err(|e| AppError::InternalError(format!("TSV write record error: {}", e)))?;
                }
                writer.flush().map_err(|e| AppError::InternalError(format!("TSV flush error: {}", e)))?;
            },
            "json" => {
                let file = File::create(&output_path)
                    .map_err(|e| AppError::InternalError(format!("Failed to create JSON file: {}", e)))?;
                let mut writer = BufWriter::new(file);

                writer.write_all(b"[\n").map_err(|e| AppError::InternalError(e.to_string()))?;
                for (idx, row) in rows.iter().enumerate() {
                    let mut obj = serde_json::Map::new();
                    for (c_idx, val) in row.iter().enumerate() {
                        let c_name = col_names.get(c_idx).cloned().unwrap_or_else(|| format!("col_{}", c_idx));
                        obj.insert(c_name, val.clone());
                    }
                    let json_val = serde_json::Value::Object(obj);
                    if idx > 0 {
                        writer.write_all(b",\n").map_err(|e| AppError::InternalError(e.to_string()))?;
                    }
                    let serialized = serde_json::to_string_pretty(&json_val)
                        .map_err(|e| AppError::InternalError(e.to_string()))?;
                    writer.write_all(serialized.as_bytes()).map_err(|e| AppError::InternalError(e.to_string()))?;
                }
                writer.write_all(b"\n]\n").map_err(|e| AppError::InternalError(e.to_string()))?;
                writer.flush().map_err(|e| AppError::InternalError(e.to_string()))?;
            },
            "ndjson" => {
                let file = File::create(&output_path)
                    .map_err(|e| AppError::InternalError(format!("Failed to create NDJSON file: {}", e)))?;
                let mut writer = BufWriter::new(file);

                for row in &rows {
                    let mut obj = serde_json::Map::new();
                    for (c_idx, val) in row.iter().enumerate() {
                        let c_name = col_names.get(c_idx).cloned().unwrap_or_else(|| format!("col_{}", c_idx));
                        obj.insert(c_name, val.clone());
                    }
                    let json_val = serde_json::Value::Object(obj);
                    let line = serde_json::to_string(&json_val).map_err(|e| AppError::InternalError(e.to_string()))?;
                    writer.write_all(line.as_bytes()).map_err(|e| AppError::InternalError(e.to_string()))?;
                    writer.write_all(b"\n").map_err(|e| AppError::InternalError(e.to_string()))?;
                }
                writer.flush().map_err(|e| AppError::InternalError(e.to_string()))?;
            },
            "xlsx" => {
                use rust_xlsxwriter::{Workbook, Format};
                let mut workbook = Workbook::new();
                let worksheet = workbook.add_worksheet();
                let header_format = Format::new().set_bold();

                for (col_idx, col_name) in col_names.iter().enumerate() {
                    worksheet.write_string_with_format(0, col_idx as u16, col_name, &header_format)
                        .map_err(|e| AppError::InternalError(format!("Excel header write error: {}", e)))?;
                }

                for (row_idx, row) in rows.iter().enumerate() {
                    for (col_idx, val) in row.iter().enumerate() {
                        match val {
                            serde_json::Value::Null => {
                                let _ = worksheet.write_string((row_idx + 1) as u32, col_idx as u16, "");
                            },
                            serde_json::Value::Bool(b) => {
                                let _ = worksheet.write_boolean((row_idx + 1) as u32, col_idx as u16, *b);
                            },
                            serde_json::Value::Number(n) => {
                                if let Some(f) = n.as_f64() {
                                    let _ = worksheet.write_number((row_idx + 1) as u32, col_idx as u16, f);
                                } else {
                                    let _ = worksheet.write_string((row_idx + 1) as u32, col_idx as u16, &n.to_string());
                                }
                            },
                            serde_json::Value::String(s) => {
                                let _ = worksheet.write_string((row_idx + 1) as u32, col_idx as u16, s);
                            },
                            other => {
                                let _ = worksheet.write_string((row_idx + 1) as u32, col_idx as u16, &other.to_string());
                            }
                        }
                    }
                }
                workbook.save(&output_path)
                    .map_err(|e| AppError::InternalError(format!("Excel save error: {}", e)))?;
            },
            _ => {},
        }

        let file_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
        Ok((output_path_str, row_count, file_size))
    }

    async fn execute_database_backup(
        &self,
        job: &ScheduleJob,
        adapter: &Arc<Box<dyn DatabaseAdapter>>,
    ) -> Result<(String, u64, u64), AppError> {
        let config = job.backup_config.as_ref()
            .ok_or_else(|| AppError::InternalError("Missing backup configuration for backup job".into()))?;

        let schema_tree = adapter.fetch_schema_tree().await?;
        let target_tables: Vec<_> = if config.tables.is_empty() {
            schema_tree.tables
        } else {
            schema_tree.tables.into_iter().filter(|t| config.tables.contains(&t.name)).collect()
        };

        if target_tables.is_empty() {
            return Err(AppError::InternalError("No tables selected or found for backup".into()));
        }

        let target_dir = PathBuf::from(&config.target_dir);
        let _ = fs::create_dir_all(&target_dir);

        let now = Local::now();
        let ts_str = now.format("%Y%m%d_%H%M%S").to_string();
        let date_str = now.format("%Y%m%d").to_string();
        let db_name = job.database.as_deref().unwrap_or("database");
        let clean_db = sanitize_filename(db_name);

        let mut filename = config.filename_pattern.clone();
        if filename.is_empty() {
            filename = format!("backup_{}_{}.sql", clean_db, ts_str);
        } else {
            filename = filename
                .replace("{name}", &sanitize_filename(&job.name))
                .replace("{timestamp}", &ts_str)
                .replace("{date}", &date_str)
                .replace("{db}", &clean_db);
            if !filename.ends_with(".sql") {
                filename.push_str(".sql");
            }
        }

        let output_path = target_dir.join(&filename);
        let output_path_str = output_path.to_string_lossy().to_string();

        let file = File::create(&output_path)
            .map_err(|e| AppError::InternalError(format!("Failed to create SQL backup file: {}", e)))?;
        let mut writer = BufWriter::new(file);

        // Header Comment
        let header = format!(
            "-- ==========================================================\n\
             -- FugDB Automated Database Backup\n\
             -- Job: {}\n\
             -- Database: {}\n\
             -- Date: {}\n\
             -- Tables Backed Up: {}\n\
             -- ==========================================================\n\n\
             BEGIN;\n\n",
            job.name,
            db_name,
            now.format("%Y-%m-%d %H:%M:%S"),
            target_tables.len()
        );
        writer.write_all(header.as_bytes()).map_err(|e| AppError::InternalError(e.to_string()))?;

        let mut total_dumped_rows = 0u64;

        for table in &target_tables {
            // 1. Schema DDL
            if config.include_schema {
                writer.write_all(format!("-- --------------------------------------------------------\n-- Table structure for \"{}\"\n-- --------------------------------------------------------\n", table.name).as_bytes())
                    .map_err(|e| AppError::InternalError(e.to_string()))?;
                writer.write_all(format!("DROP TABLE IF EXISTS \"{}\";\n", table.name).as_bytes())
                    .map_err(|e| AppError::InternalError(e.to_string()))?;
                writer.write_all(format!("CREATE TABLE \"{}\" (\n", table.name).as_bytes())
                    .map_err(|e| AppError::InternalError(e.to_string()))?;

                let col_defs: Vec<String> = table.columns.iter().map(|col| {
                    let mut def = format!("    \"{}\" {}", col.name, col.data_type);
                    if !col.nullable {
                        def.push_str(" NOT NULL");
                    }
                    if col.is_primary_key {
                        def.push_str(" PRIMARY KEY");
                    }
                    def
                }).collect();

                writer.write_all(col_defs.join(",\n").as_bytes())
                    .map_err(|e| AppError::InternalError(e.to_string()))?;
                writer.write_all(b"\n);\n\n").map_err(|e| AppError::InternalError(e.to_string()))?;
            }

            // 2. Data INSERTs
            if config.include_data {
                let s = &table.schema;
                let sql = if !s.is_empty() && s != "public" && s != "dbo" {
                    format!("SELECT * FROM \"{}\".\"{}\"", s, table.name)
                } else {
                    format!("SELECT * FROM \"{}\"", table.name)
                };

                if let Ok(res) = adapter.execute_query(&sql, None, None).await {
                    let count = res.rows.len() as u64;
                    total_dumped_rows += count;

                    if !res.rows.is_empty() {
                        writer.write_all(format!("-- Dumping data for table \"{}\" ({} rows)\n", table.name, count).as_bytes())
                            .map_err(|e| AppError::InternalError(e.to_string()))?;

                        let col_names: Vec<String> = res.columns.iter().map(|c| format!("\"{}\"", c.name)).collect();
                        let col_names_str = col_names.join(", ");

                        for chunk in res.rows.chunks(100) {
                            writer.write_all(format!("INSERT INTO \"{}\" ({}) VALUES\n", table.name, col_names_str).as_bytes())
                                .map_err(|e| AppError::InternalError(e.to_string()))?;

                            let values_rows: Vec<String> = chunk.iter().map(|row| {
                                let vals: Vec<String> = row.iter().map(|v| match v {
                                    serde_json::Value::Null => "NULL".to_string(),
                                    serde_json::Value::Number(n) => n.to_string(),
                                    serde_json::Value::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
                                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                                    other => format!("'{}'", other.to_string().replace('\'', "''")),
                                }).collect();
                                format!("    ({})", vals.join(", "))
                            }).collect();

                            writer.write_all(values_rows.join(",\n").as_bytes())
                                .map_err(|e| AppError::InternalError(e.to_string()))?;
                            writer.write_all(b";\n").map_err(|e| AppError::InternalError(e.to_string()))?;
                        }
                        writer.write_all(b"\n").map_err(|e| AppError::InternalError(e.to_string()))?;
                    }
                }
            }
        }

        writer.write_all(b"COMMIT;\n-- End of FugDB Backup\n").map_err(|e| AppError::InternalError(e.to_string()))?;
        writer.flush().map_err(|e| AppError::InternalError(e.to_string()))?;

        let file_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
        Ok((output_path_str, total_dumped_rows, file_size))
    }
}

pub fn calculate_next_run(cron_expr: &str, from_timestamp_ms: i64) -> i64 {
    let from_secs = from_timestamp_ms / 1000;
    let from_dt = DateTime::from_timestamp(from_secs, 0)
        .unwrap_or_else(|| Local::now().to_utc())
        .with_timezone(&Local);

    let expr = cron_expr.trim().to_lowercase();

    // 1. Preset intervals
    if let Some(rest) = expr.strip_prefix("interval:") {
        if let Some(m_str) = rest.strip_suffix('m') {
            if let Ok(mins) = m_str.parse::<i64>() {
                return from_timestamp_ms + (mins * 60 * 1000);
            }
        }
        if let Some(h_str) = rest.strip_suffix('h') {
            if let Ok(hrs) = h_str.parse::<i64>() {
                return from_timestamp_ms + (hrs * 3600 * 1000);
            }
        }
        if let Some(d_str) = rest.strip_suffix('d') {
            if let Ok(days) = d_str.parse::<i64>() {
                return from_timestamp_ms + (days * 86400 * 1000);
            }
        }
    }

    // 2. Daily at HH:MM
    if let Some(rest) = expr.strip_prefix("daily:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() >= 2 {
            if let (Ok(h), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                if let Some(target_time) = NaiveTime::from_hms_opt(h, m, 0) {
                    let current_time = from_dt.time();
                    let target_dt = if current_time < target_time {
                        from_dt.date_naive().and_time(target_time).and_local_timezone(Local).unwrap()
                    } else {
                        (from_dt.date_naive() + chrono::Duration::days(1)).and_time(target_time).and_local_timezone(Local).unwrap()
                    };
                    return target_dt.timestamp_millis();
                }
            }
        }
    }

    // 3. Weekly on DAY at HH:MM
    if let Some(rest) = expr.strip_prefix("weekly:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() >= 3 {
            let day_num = match parts[0] {
                "mon" | "monday" | "1" => 1,
                "tue" | "tuesday" | "2" => 2,
                "wed" | "wednesday" | "3" => 3,
                "thu" | "thursday" | "4" => 4,
                "fri" | "friday" | "5" => 5,
                "sat" | "saturday" | "6" => 6,
                "sun" | "sunday" | "0" | "7" => 7,
                _ => 1,
            };
            if let (Ok(h), Ok(m)) = (parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
                if let Some(target_time) = NaiveTime::from_hms_opt(h, m, 0) {
                    for add_days in 0..8 {
                        let candidate_date = from_dt.date_naive() + chrono::Duration::days(add_days);
                        if candidate_date.weekday().number_from_monday() == day_num {
                            let candidate_dt = candidate_date.and_time(target_time).and_local_timezone(Local).unwrap();
                            if candidate_dt.timestamp_millis() > from_timestamp_ms {
                                return candidate_dt.timestamp_millis();
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Standard Cron approximations (e.g. "*/15 * * * *" -> 15 min, "0 * * * *" -> 1 hour, "0 2 * * *" -> daily 02:00)
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() == 5 {
        if parts[0].starts_with("*/") {
            if let Ok(mins) = parts[0][2..].parse::<i64>() {
                return from_timestamp_ms + (mins * 60 * 1000);
            }
        }
        if parts[0] == "0" && parts[1] == "*" {
            return from_timestamp_ms + (3600 * 1000);
        }
        if parts[0] == "0" && parts[1] != "*" {
            if let Ok(h) = parts[1].parse::<u32>() {
                if let Some(target_time) = NaiveTime::from_hms_opt(h, 0, 0) {
                    let current_time = from_dt.time();
                    let target_dt = if current_time < target_time {
                        from_dt.date_naive().and_time(target_time).and_local_timezone(Local).unwrap()
                    } else {
                        (from_dt.date_naive() + chrono::Duration::days(1)).and_time(target_time).and_local_timezone(Local).unwrap()
                    };
                    return target_dt.timestamp_millis();
                }
            }
        }
    }

    // Default fallback: 1 hour
    from_timestamp_ms + (3600 * 1000)
}

pub fn start_scheduler_background_worker(
    app: AppHandle,
    manager: Arc<SchedulerManager>,
    pools: Arc<RwLock<HashMap<String, Arc<Box<dyn DatabaseAdapter>>>>>,
) {
    tauri::async_runtime::spawn(async move {
        // Initial wait after app boot
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        loop {
            let now = Local::now().timestamp_millis();
            let jobs = manager.load_jobs();

            for job in jobs {
                if !job.enabled {
                    continue;
                }

                // If next_run_at is unset or reached
                let is_due = match job.next_run_at {
                    Some(next_ts) => next_ts <= now,
                    None => true,
                };

                if is_due {
                    let app_clone = app.clone();
                    let manager_clone = manager.clone();
                    let pools_clone = pools.clone();
                    let jid = job.id.clone();

                    tauri::async_runtime::spawn(async move {
                        let _ = manager_clone.execute_job(&app_clone, &pools_clone, &jid).await;
                    });
                }
            }

            // Check every 10 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });
}

pub fn open_folder_in_os(path_str: &str) -> Result<(), AppError> {
    let path = Path::new(path_str);
    let dir_to_open = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(dir_to_open)
            .spawn()
            .map_err(|e| AppError::InternalError(format!("Failed to open folder in Finder: {}", e)))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(dir_to_open)
            .spawn()
            .map_err(|e| AppError::InternalError(format!("Failed to open folder in Explorer: {}", e)))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(dir_to_open)
            .spawn()
            .map_err(|e| AppError::InternalError(format!("Failed to open folder: {}", e)))?;
    }

    Ok(())
}

fn dirs_base() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata);
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }

    PathBuf::from(".")
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}
