use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::transfer::{ConflictStrategy, DbToDbTransferRequest, TransferProgressEvent};

pub async fn run_db_to_db_transfer_job(
    app: AppHandle,
    job_id: String,
    source_adapter: Arc<Box<dyn DatabaseAdapter>>,
    target_adapter: Arc<Box<dyn DatabaseAdapter>>,
    req: DbToDbTransferRequest,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), AppError> {
    let start_time = Instant::now();
    let mut rows_processed = 0u64;

    // 1. Determine Source Query
    let source_sql = if let Some(ref q) = req.source_query {
        q.clone()
    } else if let Some(ref t) = req.source_table {
        if let Some(ref s) = req.source_schema {
            format!("SELECT * FROM \"{}\".\"{}\"", s, t)
        } else {
            format!("SELECT * FROM \"{}\"", t)
        }
    } else {
        return Err(AppError::InternalError("Neither source table nor query provided".into()));
    };

    let target_schema = req.target_schema.as_deref().unwrap_or("public");
    let target_table = &req.target_table;
    let full_target_table = format!("\"{}\".\"{}\"", target_schema, target_table);
    let conflict_strat = req.conflict_strategy.unwrap_or(ConflictStrategy::Fail);
    let batch_chunk_size = req.batch_size.unwrap_or(500).max(1);

    emit_event(
        &app,
        &job_id,
        0,
        None,
        0,
        0.0,
        None,
        Some(0.0),
        "running",
        Some("Connecting to source database and executing extraction query...".into()),
        None,
    );

    // 2. Fetch source query results
    let source_result = source_adapter.execute_query(&source_sql, None, None).await?;
    let total_count = source_result.rows.len() as u64;
    let total_rows_estimated = source_result.total_rows.or(Some(total_count));

    let columns = source_result.columns;
    let col_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();
    let rows = source_result.rows;

    if columns.is_empty() {
        return Err(AppError::InternalError("Source query returned 0 columns".into()));
    }

    // 3. Auto-create target table if requested
    if req.create_table_if_missing.unwrap_or(false) {
        emit_event(
            &app,
            &job_id,
            0,
            total_rows_estimated,
            0,
            0.0,
            None,
            Some(2.0),
            "running",
            Some(format!("Ensuring target table {} exists with smart type mapping...", full_target_table)),
            None,
        );

        let mut ddl = format!("CREATE TABLE IF NOT EXISTS {} (\n", full_target_table);
        let col_defs: Vec<String> = columns.iter().map(|c| {
            let mapped_type = map_source_type_to_generic(&c.data_type);
            let mut s = format!("  \"{}\" {}", c.name, mapped_type);
            if c.is_primary_key {
                s.push_str(" PRIMARY KEY");
            } else if !c.nullable {
                s.push_str(" NOT NULL");
            }
            s
        }).collect();
        ddl.push_str(&col_defs.join(",\n"));
        ddl.push_str("\n);");

        // Execute CREATE TABLE on target (ignore if already exists)
        target_adapter.execute_query(&ddl, None, None).await.ok();
    }

    // 4. Truncate target if requested
    if req.truncate_target_first.unwrap_or(false) {
        emit_event(
            &app,
            &job_id,
            0,
            total_rows_estimated,
            0,
            0.0,
            None,
            Some(5.0),
            "running",
            Some(format!("Truncating target table {} before transfer...", full_target_table)),
            None,
        );
        let truncate_sql = format!("DELETE FROM {};", full_target_table);
        target_adapter.execute_query(&truncate_sql, None, None).await.ok();
    }

    // 5. Stream data in batches into target database
    for chunk in rows.chunks(batch_chunk_size) {
        if cancel_flag.load(Ordering::SeqCst) {
            emit_event(
                &app,
                &job_id,
                rows_processed,
                total_rows_estimated,
                0,
                0.0,
                None,
                Some(0.0),
                "cancelled",
                Some("Cross-database migration cancelled by user".into()),
                None,
            );
            return Ok(());
        }

        let inserted = target_adapter.batch_insert_rows(&full_target_table, &col_names, chunk, &conflict_strat).await?;
        rows_processed += inserted;

        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
        let rps = rows_processed as f64 / elapsed;
        let pct = if total_count > 0 { (rows_processed as f64 / total_count as f64) * 100.0 } else { 100.0 };
        let eta = if rps > 0.0 && total_count > rows_processed {
            Some(((total_count - rows_processed) as f64 / rps) as u64)
        } else {
            None
        };

        emit_event(
            &app,
            &job_id,
            rows_processed,
            total_rows_estimated,
            rows_processed * 128,
            rps,
            eta,
            Some(pct),
            "running",
            Some(format!("Transferred {}/{} rows ({:.1}%)", rows_processed, total_count, pct)),
            None,
        );
    }

    let total_elapsed = start_time.elapsed().as_secs_f64().max(0.001);
    let final_rps = rows_processed as f64 / total_elapsed;

    emit_event(
        &app,
        &job_id,
        rows_processed,
        Some(rows_processed),
        rows_processed * 128,
        final_rps,
        Some(0),
        Some(100.0),
        "completed",
        Some(format!("Successfully migrated {} rows to {} in {:.2}s ({:.0} rows/s)", rows_processed, full_target_table, total_elapsed, final_rps)),
        None,
    );

    Ok(())
}

fn map_source_type_to_generic(raw_type: &str) -> &'static str {
    let lower = raw_type.to_lowercase();
    if lower.contains("int8") || lower.contains("bigint") || lower.contains("serial8") {
        "BIGINT"
    } else if lower.contains("int") || lower.contains("serial") {
        "INTEGER"
    } else if lower.contains("bool") {
        "BOOLEAN"
    } else if lower.contains("float") || lower.contains("double") || lower.contains("real") {
        "DOUBLE PRECISION"
    } else if lower.contains("numeric") || lower.contains("decimal") || lower.contains("money") {
        "DECIMAL(18,4)"
    } else if lower.contains("timestamp") || lower.contains("datetime") {
        "TIMESTAMP"
    } else if lower.contains("date") {
        "DATE"
    } else if lower.contains("json") {
        "TEXT"
    } else if lower.contains("uuid") {
        "VARCHAR(36)"
    } else if lower.contains("bytea") || lower.contains("blob") {
        "BYTEA"
    } else {
        "TEXT"
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
