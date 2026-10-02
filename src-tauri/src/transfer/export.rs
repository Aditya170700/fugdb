use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::transfer::{ExportJobRequest, TransferFormat, TransferProgressEvent};

pub async fn run_export_job(
    app: AppHandle,
    job_id: String,
    adapter: Arc<Box<dyn DatabaseAdapter>>,
    req: ExportJobRequest,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), AppError> {
    let start_time = Instant::now();
    let mut rows_processed = 0u64;

    // 1. Determine SQL to execute
    let sql = if let Some(ref q) = req.query {
        q.clone()
    } else if let Some(ref t) = req.table {
        if let Some(ref s) = req.schema {
            format!("SELECT * FROM \"{}\".\"{}\"", s, t)
        } else {
            format!("SELECT * FROM \"{}\"", t)
        }
    } else {
        return Err(AppError::InternalError("Neither query nor table provided for export".into()));
    };

    // 2. Fetch data via query execution
    // (For ultra large tables, we can stream in chunks or execute)
    let query_result = adapter.execute_query(&sql, None, None).await?;
    let total_rows_estimated = query_result.total_rows.or(Some(query_result.rows.len() as u64));
    let total_count = query_result.rows.len() as u64;

    let columns = query_result.columns;
    let col_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();
    let rows = query_result.rows;

    let target_path = req.target_path.clone();

    // 3. Write data according to selected format
    match &req.format {
        TransferFormat::Csv { delimiter, has_header, quote_char: _ } => {
            let delim_char = delimiter.as_deref().and_then(|d| d.chars().next()).unwrap_or(',');
            let file = File::create(&target_path)
                .map_err(|e| AppError::InternalError(format!("Failed to create export file: {}", e)))?;
            let mut writer = csv::WriterBuilder::new()
                .delimiter(delim_char as u8)
                .has_headers(has_header.unwrap_or(true))
                .from_writer(BufWriter::new(file));

            if has_header.unwrap_or(true) {
                writer.write_record(&col_names)
                    .map_err(|e| AppError::InternalError(format!("CSV write header error: {}", e)))?;
            }

            for (idx, row) in rows.iter().enumerate() {
                if cancel_flag.load(Ordering::SeqCst) {
                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, 0, 0.0, None, Some(0.0), "cancelled", Some("Export cancelled by user".into()), None);
                    return Ok(());
                }

                let record: Vec<String> = row.iter().map(|v| match v {
                    serde_json::Value::Null => "".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                }).collect();

                writer.write_record(&record)
                    .map_err(|e| AppError::InternalError(format!("CSV write record error: {}", e)))?;

                rows_processed += 1;

                if idx % 500 == 0 || idx == rows.len() - 1 {
                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let rps = rows_processed as f64 / elapsed;
                    let pct = if total_count > 0 { (rows_processed as f64 / total_count as f64) * 100.0 } else { 0.0 };
                    let eta = if rps > 0.0 && total_count > rows_processed {
                        Some(((total_count - rows_processed) as f64 / rps) as u64)
                    } else {
                        None
                    };

                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, rows_processed * 64, rps, eta, Some(pct), "running", Some(format!("Exporting CSV row {}/{}", rows_processed, total_count)), None);
                }
            }
            writer.flush().map_err(|e| AppError::InternalError(format!("CSV flush error: {}", e)))?;
        }

        TransferFormat::Tsv { has_header } => {
            let file = File::create(&target_path)
                .map_err(|e| AppError::InternalError(format!("Failed to create TSV file: {}", e)))?;
            let mut writer = csv::WriterBuilder::new()
                .delimiter(b'\t')
                .has_headers(has_header.unwrap_or(true))
                .from_writer(BufWriter::new(file));

            if has_header.unwrap_or(true) {
                writer.write_record(&col_names)
                    .map_err(|e| AppError::InternalError(format!("TSV write header error: {}", e)))?;
            }

            for (idx, row) in rows.iter().enumerate() {
                if cancel_flag.load(Ordering::SeqCst) {
                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, 0, 0.0, None, Some(0.0), "cancelled", Some("Export cancelled by user".into()), None);
                    return Ok(());
                }

                let record: Vec<String> = row.iter().map(|v| match v {
                    serde_json::Value::Null => "".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                }).collect();

                writer.write_record(&record)
                    .map_err(|e| AppError::InternalError(format!("TSV write record error: {}", e)))?;

                rows_processed += 1;

                if idx % 500 == 0 || idx == rows.len() - 1 {
                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let rps = rows_processed as f64 / elapsed;
                    let pct = if total_count > 0 { (rows_processed as f64 / total_count as f64) * 100.0 } else { 0.0 };
                    let eta = if rps > 0.0 && total_count > rows_processed {
                        Some(((total_count - rows_processed) as f64 / rps) as u64)
                    } else {
                        None
                    };

                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, rows_processed * 64, rps, eta, Some(pct), "running", Some(format!("Exporting TSV row {}/{}", rows_processed, total_count)), None);
                }
            }
            writer.flush().map_err(|e| AppError::InternalError(format!("TSV flush error: {}", e)))?;
        }

        TransferFormat::Json { is_ndjson, pretty } => {
            let file = File::create(&target_path)
                .map_err(|e| AppError::InternalError(format!("Failed to create JSON file: {}", e)))?;
            let mut writer = BufWriter::new(file);

            let is_nd = is_ndjson.unwrap_or(false);
            let is_pretty = pretty.unwrap_or(false) && !is_nd;

            if !is_nd {
                writer.write_all(if is_pretty { b"[\n" } else { b"[" })
                    .map_err(|e| AppError::InternalError(e.to_string()))?;
            }

            for (idx, row) in rows.iter().enumerate() {
                if cancel_flag.load(Ordering::SeqCst) {
                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, 0, 0.0, None, Some(0.0), "cancelled", Some("Export cancelled by user".into()), None);
                    return Ok(());
                }

                let mut obj = serde_json::Map::new();
                for (c_idx, val) in row.iter().enumerate() {
                    let c_name = col_names.get(c_idx).cloned().unwrap_or_else(|| format!("col_{}", c_idx));
                    obj.insert(c_name, val.clone());
                }
                let json_val = serde_json::Value::Object(obj);

                if is_nd {
                    let line = serde_json::to_string(&json_val).map_err(|e| AppError::InternalError(e.to_string()))?;
                    writer.write_all(line.as_bytes()).map_err(|e| AppError::InternalError(e.to_string()))?;
                    writer.write_all(b"\n").map_err(|e| AppError::InternalError(e.to_string()))?;
                } else {
                    if idx > 0 {
                        writer.write_all(if is_pretty { b",\n" } else { b"," })
                            .map_err(|e| AppError::InternalError(e.to_string()))?;
                    }
                    let serialized = if is_pretty {
                        serde_json::to_string_pretty(&json_val).map_err(|e| AppError::InternalError(e.to_string()))?
                    } else {
                        serde_json::to_string(&json_val).map_err(|e| AppError::InternalError(e.to_string()))?
                    };
                    writer.write_all(serialized.as_bytes()).map_err(|e| AppError::InternalError(e.to_string()))?;
                }

                rows_processed += 1;

                if idx % 500 == 0 || idx == rows.len() - 1 {
                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let rps = rows_processed as f64 / elapsed;
                    let pct = if total_count > 0 { (rows_processed as f64 / total_count as f64) * 100.0 } else { 0.0 };
                    let eta = if rps > 0.0 && total_count > rows_processed {
                        Some(((total_count - rows_processed) as f64 / rps) as u64)
                    } else {
                        None
                    };

                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, rows_processed * 128, rps, eta, Some(pct), "running", Some(format!("Exporting JSON object {}/{}", rows_processed, total_count)), None);
                }
            }

            if !is_nd {
                writer.write_all(if is_pretty { b"\n]" } else { b"]" })
                    .map_err(|e| AppError::InternalError(e.to_string()))?;
            }
            writer.flush().map_err(|e| AppError::InternalError(format!("JSON flush error: {}", e)))?;
        }

        TransferFormat::Excel { sheet_name } => {
            let mut workbook = rust_xlsxwriter::Workbook::new();
            let worksheet = workbook.add_worksheet();
            if let Some(name) = sheet_name {
                worksheet.set_name(name).ok();
            }

            let header_format = rust_xlsxwriter::Format::new()
                .set_bold()
                .set_background_color(rust_xlsxwriter::Color::RGB(0x4338CA))
                .set_font_color(rust_xlsxwriter::Color::RGB(0xFFFFFF));

            // Write headers
            for (col_idx, col_name) in col_names.iter().enumerate() {
                worksheet.write_string_with_format(0, col_idx as u16, col_name, &header_format)
                    .map_err(|e| AppError::InternalError(format!("Excel write header error: {}", e)))?;
            }

            for (row_idx, row) in rows.iter().enumerate() {
                if cancel_flag.load(Ordering::SeqCst) {
                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, 0, 0.0, None, Some(0.0), "cancelled", Some("Export cancelled by user".into()), None);
                    return Ok(());
                }

                let excel_row = (row_idx + 1) as u32;
                for (col_idx, val) in row.iter().enumerate() {
                    let col_u16 = col_idx as u16;
                    match val {
                        serde_json::Value::Null => {
                            worksheet.write_blank(excel_row, col_u16, &rust_xlsxwriter::Format::new()).ok();
                        }
                        serde_json::Value::Bool(b) => {
                            worksheet.write_boolean(excel_row, col_u16, *b).ok();
                        }
                        serde_json::Value::Number(num) => {
                            if let Some(i) = num.as_i64() {
                                worksheet.write_number(excel_row, col_u16, i as f64).ok();
                            } else if let Some(f) = num.as_f64() {
                                worksheet.write_number(excel_row, col_u16, f).ok();
                            } else {
                                worksheet.write_string(excel_row, col_u16, num.to_string()).ok();
                            }
                        }
                        serde_json::Value::String(s) => {
                            worksheet.write_string(excel_row, col_u16, s).ok();
                        }
                        other => {
                            worksheet.write_string(excel_row, col_u16, other.to_string()).ok();
                        }
                    }
                }

                rows_processed += 1;

                if row_idx % 500 == 0 || row_idx == rows.len() - 1 {
                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let rps = rows_processed as f64 / elapsed;
                    let pct = if total_count > 0 { (rows_processed as f64 / total_count as f64) * 100.0 } else { 0.0 };
                    let eta = if rps > 0.0 && total_count > rows_processed {
                        Some(((total_count - rows_processed) as f64 / rps) as u64)
                    } else {
                        None
                    };

                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, rows_processed * 128, rps, eta, Some(pct), "running", Some(format!("Exporting Excel row {}/{}", rows_processed, total_count)), None);
                }
            }

            worksheet.autofit();
            workbook.save(&target_path)
                .map_err(|e| AppError::InternalError(format!("Failed to save Excel file: {}", e)))?;
        }

        TransferFormat::SqlDump { include_ddl, batch_size } => {
            let file = File::create(&target_path)
                .map_err(|e| AppError::InternalError(format!("Failed to create SQL Dump file: {}", e)))?;
            let mut writer = BufWriter::new(file);

            let table_name = req.table.as_deref().unwrap_or("exported_table");
            let schema_name = req.schema.as_deref().unwrap_or("public");
            let full_table_name = format!("\"{}\".\"{}\"", schema_name, table_name);

            // 1. DDL
            if include_ddl.unwrap_or(true) {
                let mut ddl = format!("-- FugDB SQL Dump Export\n-- Generated on: {}\n\n", chrono::Utc::now().to_rfc3339());
                ddl.push_str(&format!("CREATE TABLE IF NOT EXISTS {} (\n", full_table_name));
                let col_defs: Vec<String> = columns.iter().map(|c| {
                    let mut s = format!("  \"{}\" {}", c.name, c.data_type);
                    if c.is_primary_key {
                        s.push_str(" PRIMARY KEY");
                    } else if !c.nullable {
                        s.push_str(" NOT NULL");
                    }
                    s
                }).collect();
                ddl.push_str(&col_defs.join(",\n"));
                ddl.push_str("\n);\n\n");
                writer.write_all(ddl.as_bytes()).map_err(|e| AppError::InternalError(e.to_string()))?;
            }

            // 2. Batch INSERT statements
            let batch_chunk_size = batch_size.unwrap_or(250).max(1);
            let quoted_cols = col_names.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", ");

            for chunk in rows.chunks(batch_chunk_size) {
                if cancel_flag.load(Ordering::SeqCst) {
                    emit_event(&app, &job_id, rows_processed, total_rows_estimated, 0, 0.0, None, Some(0.0), "cancelled", Some("Export cancelled by user".into()), None);
                    return Ok(());
                }

                let mut insert_sql = format!("INSERT INTO {} ({}) VALUES\n", full_table_name, quoted_cols);
                let mut row_strings = Vec::new();

                for row in chunk {
                    let val_literals: Vec<String> = row.iter().map(|v| format_sql_literal(v)).collect();
                    row_strings.push(format!("  ({})", val_literals.join(", ")));
                    rows_processed += 1;
                }

                insert_sql.push_str(&row_strings.join(",\n"));
                insert_sql.push_str(";\n\n");

                writer.write_all(insert_sql.as_bytes()).map_err(|e| AppError::InternalError(e.to_string()))?;

                let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                let rps = rows_processed as f64 / elapsed;
                let pct = if total_count > 0 { (rows_processed as f64 / total_count as f64) * 100.0 } else { 0.0 };
                let eta = if rps > 0.0 && total_count > rows_processed {
                    Some(((total_count - rows_processed) as f64 / rps) as u64)
                } else {
                    None
                };

                emit_event(&app, &job_id, rows_processed, total_rows_estimated, rows_processed * 100, rps, eta, Some(pct), "running", Some(format!("Exporting SQL Dump rows {}/{}", rows_processed, total_count)), None);
            }

            writer.flush().map_err(|e| AppError::InternalError(format!("SQL flush error: {}", e)))?;
        }
    }

    // Finished successfully
    let total_elapsed = start_time.elapsed().as_secs_f64().max(0.001);
    let final_rps = rows_processed as f64 / total_elapsed;
    let file_bytes = std::fs::metadata(&target_path).map(|m| m.len()).unwrap_or(0);

    emit_event(
        &app, 
        &job_id, 
        rows_processed, 
        Some(rows_processed), 
        file_bytes, 
        final_rps, 
        Some(0), 
        Some(100.0), 
        "completed", 
        Some(format!("Successfully exported {} rows ({:.2} KB) in {:.2}s", rows_processed, file_bytes as f64 / 1024.0, total_elapsed)), 
        None
    );

    Ok(())
}

fn format_sql_literal(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        other => format!("'{}'", other.to_string().replace('\'', "''")),
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
