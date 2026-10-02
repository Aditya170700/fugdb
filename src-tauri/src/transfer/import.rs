use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use calamine::{open_workbook_auto, Reader};
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::transfer::{ConflictStrategy, ImportJobRequest, TransferFormat, TransferProgressEvent};

pub async fn run_import_job(
    app: AppHandle,
    job_id: String,
    adapter: Arc<Box<dyn DatabaseAdapter>>,
    req: ImportJobRequest,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), AppError> {
    let start_time = Instant::now();
    let path = Path::new(&req.source_path);
    if !path.exists() {
        return Err(AppError::InternalError(format!("Import source file does not exist: {}", req.source_path)));
    }

    let file_metadata = std::fs::metadata(path)
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let total_bytes = file_metadata.len();

    let table_name = req.table.clone();
    let schema_name = req.schema.clone().unwrap_or_else(|| "public".to_string());
    let full_table_name = format!("\"{}\".\"{}\"", schema_name, table_name);
    let conflict_strat = req.conflict_strategy.unwrap_or(ConflictStrategy::Fail);

    let mut rows_processed = 0u64;
    let mut bytes_read = 0u64;

    match &req.format {
        TransferFormat::Csv { delimiter, has_header, quote_char: _ } => {
            let delim_char = delimiter.as_deref().and_then(|d| d.chars().next()).unwrap_or(',');
            let mut csv_reader = csv::ReaderBuilder::new()
                .delimiter(delim_char as u8)
                .has_headers(has_header.unwrap_or(true))
                .flexible(true)
                .from_path(path)
                .map_err(|e| AppError::InternalError(format!("Failed to parse CSV: {}", e)))?;

            let headers = csv_reader.headers()
                .map_err(|e| AppError::InternalError(format!("Failed to read CSV headers: {}", e)))?;
            let columns: Vec<String> = headers.iter().map(|s| s.trim().to_string()).collect();

            let mut batch: Vec<Vec<Value>> = Vec::new();
            let batch_size = 500;

            for rec_res in csv_reader.records() {
                if cancel_flag.load(Ordering::SeqCst) {
                    emit_event(&app, &job_id, rows_processed, None, bytes_read, 0.0, None, Some(0.0), "cancelled", Some("Import cancelled by user".into()), None);
                    return Ok(());
                }

                let rec = rec_res.map_err(|e| AppError::InternalError(format!("CSV read record error: {}", e)))?;
                let row_vals: Vec<Value> = rec.iter().map(|s| parse_cell_value(s)).collect();
                batch.push(row_vals);
                bytes_read += rec.as_slice().len() as u64 + 1;

                if batch.len() >= batch_size {
                    adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                    rows_processed += batch.len() as u64;
                    batch.clear();

                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let rps = rows_processed as f64 / elapsed;
                    let pct = if total_bytes > 0 { (bytes_read as f64 / total_bytes as f64) * 100.0 } else { 0.0 };
                    let eta = if rps > 0.0 && total_bytes > bytes_read {
                        let rows_est = (total_bytes as f64 / (bytes_read as f64 / rows_processed as f64)) as u64;
                        if rows_est > rows_processed { Some(((rows_est - rows_processed) as f64 / rps) as u64) } else { None }
                    } else { None };

                    emit_event(&app, &job_id, rows_processed, None, bytes_read, rps, eta, Some(pct), "running", Some(format!("Imported {} rows...", rows_processed)), None);
                }
            }

            if !batch.is_empty() {
                adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                rows_processed += batch.len() as u64;
            }
        }

        TransferFormat::Tsv { has_header } => {
            let mut csv_reader = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(has_header.unwrap_or(true))
                .flexible(true)
                .from_path(path)
                .map_err(|e| AppError::InternalError(format!("Failed to parse TSV: {}", e)))?;

            let headers = csv_reader.headers()
                .map_err(|e| AppError::InternalError(format!("Failed to read TSV headers: {}", e)))?;
            let columns: Vec<String> = headers.iter().map(|s| s.trim().to_string()).collect();

            let mut batch: Vec<Vec<Value>> = Vec::new();
            let batch_size = 500;

            for rec_res in csv_reader.records() {
                if cancel_flag.load(Ordering::SeqCst) {
                    emit_event(&app, &job_id, rows_processed, None, bytes_read, 0.0, None, Some(0.0), "cancelled", Some("Import cancelled by user".into()), None);
                    return Ok(());
                }

                let rec = rec_res.map_err(|e| AppError::InternalError(format!("TSV read record error: {}", e)))?;
                let row_vals: Vec<Value> = rec.iter().map(|s| parse_cell_value(s)).collect();
                batch.push(row_vals);
                bytes_read += rec.as_slice().len() as u64 + 1;

                if batch.len() >= batch_size {
                    adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                    rows_processed += batch.len() as u64;
                    batch.clear();

                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let rps = rows_processed as f64 / elapsed;
                    let pct = if total_bytes > 0 { (bytes_read as f64 / total_bytes as f64) * 100.0 } else { 0.0 };

                    emit_event(&app, &job_id, rows_processed, None, bytes_read, rps, None, Some(pct), "running", Some(format!("Imported {} rows...", rows_processed)), None);
                }
            }

            if !batch.is_empty() {
                adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                rows_processed += batch.len() as u64;
            }
        }

        TransferFormat::Json { is_ndjson, pretty: _ } => {
            let is_nd = is_ndjson.unwrap_or(false);
            let file = File::open(path)
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            let reader = BufReader::new(file);

            if is_nd {
                let mut columns: Vec<String> = Vec::new();
                let mut batch: Vec<Vec<Value>> = Vec::new();
                let batch_size = 500;

                for line_res in reader.lines() {
                    if cancel_flag.load(Ordering::SeqCst) {
                        emit_event(&app, &job_id, rows_processed, None, bytes_read, 0.0, None, Some(0.0), "cancelled", Some("Import cancelled by user".into()), None);
                        return Ok(());
                    }

                    let line = line_res.map_err(|e| AppError::InternalError(e.to_string()))?;
                    let trimmed = line.trim();
                    if trimmed.is_empty() { continue; }
                    bytes_read += line.len() as u64 + 1;

                    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) {
                        if columns.is_empty() {
                            columns = map.keys().cloned().collect();
                        }
                        let row_vals: Vec<Value> = columns.iter()
                            .map(|c| map.get(c).cloned().unwrap_or(Value::Null))
                            .collect();
                        batch.push(row_vals);

                        if batch.len() >= batch_size {
                            adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                            rows_processed += batch.len() as u64;
                            batch.clear();

                            let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                            let rps = rows_processed as f64 / elapsed;
                            let pct = if total_bytes > 0 { (bytes_read as f64 / total_bytes as f64) * 100.0 } else { 0.0 };

                            emit_event(&app, &job_id, rows_processed, None, bytes_read, rps, None, Some(pct), "running", Some(format!("Imported {} JSON rows...", rows_processed)), None);
                        }
                    }
                }

                if !batch.is_empty() {
                    adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                    rows_processed += batch.len() as u64;
                }
            } else {
                // Standard JSON Array
                let mut full_content = String::new();
                std::fs::File::open(path)
                    .map_err(|e| AppError::InternalError(e.to_string()))?
                    .read_to_string(&mut full_content)
                    .map_err(|e| AppError::InternalError(e.to_string()))?;

                let parsed_val: Value = serde_json::from_str(&full_content)
                    .map_err(|e| AppError::InternalError(format!("Failed to parse JSON file: {}", e)))?;

                if let Value::Array(items) = parsed_val {
                    let mut columns: Vec<String> = Vec::new();
                    if let Some(Value::Object(map)) = items.first() {
                        columns = map.keys().cloned().collect();
                    }

                    let batch_size = 500;
                    for chunk in items.chunks(batch_size) {
                        if cancel_flag.load(Ordering::SeqCst) {
                            emit_event(&app, &job_id, rows_processed, None, bytes_read, 0.0, None, Some(0.0), "cancelled", Some("Import cancelled by user".into()), None);
                            return Ok(());
                        }

                        let mut batch = Vec::new();
                        for item in chunk {
                            if let Value::Object(map) = item {
                                let row_vals: Vec<Value> = columns.iter()
                                    .map(|c| map.get(c).cloned().unwrap_or(Value::Null))
                                    .collect();
                                batch.push(row_vals);
                            }
                        }

                        adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                        rows_processed += batch.len() as u64;

                        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                        let rps = rows_processed as f64 / elapsed;
                        let pct = if !items.is_empty() { (rows_processed as f64 / items.len() as f64) * 100.0 } else { 100.0 };

                        emit_event(&app, &job_id, rows_processed, Some(items.len() as u64), total_bytes, rps, None, Some(pct), "running", Some(format!("Imported {} JSON rows...", rows_processed)), None);
                    }
                }
            }
        }

        TransferFormat::Excel { sheet_name } => {
            let mut workbook = open_workbook_auto(path)
                .map_err(|e| AppError::InternalError(format!("Failed to open Excel workbook: {}", e)))?;

            let target_sheet = sheet_name.clone().unwrap_or_else(|| {
                workbook.sheet_names().first().cloned().unwrap_or_else(|| "Sheet1".to_string())
            });

            if let Ok(range) = workbook.worksheet_range(&target_sheet) {
                let mut row_iter = range.rows();
                let mut columns: Vec<String> = Vec::new();

                if let Some(header_row) = row_iter.next() {
                    for (idx, cell) in header_row.iter().enumerate() {
                        let col_name = cell.to_string();
                        columns.push(if col_name.is_empty() { format!("col_{}", idx + 1) } else { col_name });
                    }
                }

                let mut batch: Vec<Vec<Value>> = Vec::new();
                let batch_size = 500;

                for row in row_iter {
                    if cancel_flag.load(Ordering::SeqCst) {
                        emit_event(&app, &job_id, rows_processed, None, bytes_read, 0.0, None, Some(0.0), "cancelled", Some("Import cancelled by user".into()), None);
                        return Ok(());
                    }

                    let mut row_vals = Vec::new();
                    for cell in row {
                        let v = match cell {
                            calamine::Data::Empty => Value::Null,
                            calamine::Data::String(s) => Value::String(s.clone()),
                            calamine::Data::Float(f) => serde_json::Number::from_f64(*f).map(Value::Number).unwrap_or(Value::Null),
                            calamine::Data::Int(i) => Value::Number((*i).into()),
                            calamine::Data::Bool(b) => Value::Bool(*b),
                            calamine::Data::DateTime(dt) => Value::String(dt.to_string()),
                            calamine::Data::Error(e) => Value::String(format!("Error: {:?}", e)),
                            calamine::Data::DateTimeIso(s) | calamine::Data::DurationIso(s) => Value::String(s.clone()),
                        };
                        row_vals.push(v);
                    }
                    batch.push(row_vals);

                    if batch.len() >= batch_size {
                        adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                        rows_processed += batch.len() as u64;
                        batch.clear();

                        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                        let rps = rows_processed as f64 / elapsed;

                        emit_event(&app, &job_id, rows_processed, None, total_bytes, rps, None, None, "running", Some(format!("Imported {} Excel rows...", rows_processed)), None);
                    }
                }

                if !batch.is_empty() {
                    adapter.batch_insert_rows(&full_table_name, &columns, &batch, &conflict_strat).await?;
                    rows_processed += batch.len() as u64;
                }
            }
        }

        TransferFormat::SqlDump { include_ddl: _, batch_size: _ } => {
            // Read and execute SQL statements
            let sql_content = std::fs::read_to_string(path)
                .map_err(|e| AppError::InternalError(format!("Failed to read SQL file: {}", e)))?;

            adapter.execute_query(&sql_content, None, None).await?;
            rows_processed = 1;
        }
    }

    let total_elapsed = start_time.elapsed().as_secs_f64().max(0.001);
    let final_rps = rows_processed as f64 / total_elapsed;

    emit_event(
        &app,
        &job_id,
        rows_processed,
        Some(rows_processed),
        total_bytes,
        final_rps,
        Some(0),
        Some(100.0),
        "completed",
        Some(format!("Successfully imported {} rows in {:.2}s", rows_processed, total_elapsed)),
        None,
    );

    Ok(())
}

fn parse_cell_value(s: &str) -> Value {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        Value::Null
    } else if let Ok(i) = trimmed.parse::<i64>() {
        Value::Number(i.into())
    } else if let Ok(f) = trimmed.parse::<f64>() {
        serde_json::Number::from_f64(f).map(Value::Number).unwrap_or_else(|| Value::String(trimmed.to_string()))
    } else if trimmed.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if trimmed.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else {
        Value::String(trimmed.to_string())
    }
}

fn emit_event(
    app: &AppHandle,
    job_id: &str,
    rows_processed: u64,
    total_rows_estimated: Option<u64>,
    bytes_processed: u64,
    rows_per_second: f64,
    estimated_seconds_remaining: Option<u64>,
    percentage: Option<f64>,
    status: &str,
    message: Option<String>,
    error_message: Option<String>,
) {
    let event = TransferProgressEvent {
        job_id: job_id.to_string(),
        rows_processed,
        total_rows_estimated,
        bytes_processed,
        rows_per_second,
        estimated_seconds_remaining,
        percentage,
        status: status.to_string(),
        message,
        error_message,
    };
    app.emit("transfer-progress", event).ok();
}
