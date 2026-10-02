use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use calamine::{open_workbook_auto, Reader};
use serde_json::Value;

use crate::error::AppError;
use crate::models::transfer::FileInspectionResult;

pub fn inspect_file(file_path: &str) -> Result<FileInspectionResult, AppError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::InternalError(format!("File does not exist: {}", file_path)));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| AppError::InternalError(format!("Failed to read file metadata: {}", e)))?;
    let total_bytes = metadata.len();

    let ext = path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "xlsx" | "xls" | "ods" => inspect_excel_file(path, total_bytes),
        "json" | "jsonl" | "ndjson" => inspect_json_file(path, total_bytes),
        "sql" => inspect_sql_file(path, total_bytes),
        _ => inspect_delimited_file(path, total_bytes),
    }
}

fn inspect_excel_file(path: &Path, total_bytes: u64) -> Result<FileInspectionResult, AppError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::InternalError(format!("Failed to open Excel workbook: {}", e)))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let first_sheet = sheet_names.first().cloned().unwrap_or_else(|| "Sheet1".to_string());

    let mut columns = Vec::new();
    let mut sample_rows = Vec::new();

    if let Ok(range) = workbook.worksheet_range(&first_sheet) {
        let mut row_iter = range.rows();
        if let Some(header_row) = row_iter.next() {
            for (idx, cell) in header_row.iter().enumerate() {
                let col_name = cell.to_string();
                columns.push(if col_name.is_empty() { format!("col_{}", idx + 1) } else { col_name });
            }
        }

        for row in row_iter.take(10) {
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
            sample_rows.push(row_vals);
        }
    }

    Ok(FileInspectionResult {
        detected_format: "excel".to_string(),
        delimiter: None,
        has_header: true,
        columns,
        sample_rows,
        total_bytes,
        sheet_names: Some(sheet_names),
    })
}

fn inspect_json_file(path: &Path, total_bytes: u64) -> Result<FileInspectionResult, AppError> {
    let file = File::open(path)
        .map_err(|e| AppError::InternalError(format!("Failed to open file: {}", e)))?;
    let mut reader = BufReader::new(file);

    // Read first chunk to detect whether it's an array or ndjson (JSON Lines)
    let mut sample_rows = Vec::new();
    let mut columns = Vec::new();

    let mut first_line = String::new();
    reader.read_line(&mut first_line)
        .map_err(|e| AppError::InternalError(format!("Failed to read line: {}", e)))?;
    let trimmed = first_line.trim();

    if trimmed.starts_with('[') {
        // Standard JSON Array
        let mut full_file = trimmed.to_string();
        reader.read_to_string(&mut full_file).ok();
        
        // Parse sample items
        if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(&full_file) {
            if let Some(Value::Object(map)) = items.first() {
                columns = map.keys().cloned().collect();
            }
            for item in items.iter().take(10) {
                if let Value::Object(map) = item {
                    let row_vals: Vec<Value> = columns.iter()
                        .map(|col| map.get(col).cloned().unwrap_or(Value::Null))
                        .collect();
                    sample_rows.push(row_vals);
                }
            }
        }

        return Ok(FileInspectionResult {
            detected_format: "json".to_string(),
            delimiter: None,
            has_header: true,
            columns,
            sample_rows,
            total_bytes,
            sheet_names: None,
        });
    }

    // Otherwise treat as NDJSON (JSON lines)
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) {
        columns = map.keys().cloned().collect();
        let row_vals: Vec<Value> = columns.iter()
            .map(|col| map.get(col).cloned().unwrap_or(Value::Null))
            .collect();
        sample_rows.push(row_vals);
    }

    for line_res in reader.lines().take(9) {
        if let Ok(line) = line_res {
            let l_trimmed = line.trim();
            if !l_trimmed.is_empty() {
                if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(l_trimmed) {
                    let row_vals: Vec<Value> = columns.iter()
                        .map(|col| map.get(col).cloned().unwrap_or(Value::Null))
                        .collect();
                    sample_rows.push(row_vals);
                }
            }
        }
    }

    Ok(FileInspectionResult {
        detected_format: "ndjson".to_string(),
        delimiter: None,
        has_header: true,
        columns,
        sample_rows,
        total_bytes,
        sheet_names: None,
    })
}

fn inspect_delimited_file(path: &Path, total_bytes: u64) -> Result<FileInspectionResult, AppError> {
    let file = File::open(path)
        .map_err(|e| AppError::InternalError(format!("Failed to open file: {}", e)))?;
    let mut reader = BufReader::new(file);

    let mut lines = Vec::new();
    for _ in 0..15 {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        lines.push(line);
    }

    if lines.is_empty() {
        return Ok(FileInspectionResult {
            detected_format: "csv".to_string(),
            delimiter: Some(",".to_string()),
            has_header: true,
            columns: vec![],
            sample_rows: vec![],
            total_bytes,
            sheet_names: None,
        });
    }

    // Guess delimiter by counting occurrences across sample lines
    let delimiters = [',', '\t', ';', '|'];
    let mut best_delim = ',';
    let mut best_count = 0;

    for &d in &delimiters {
        let count = lines[0].chars().filter(|&c| c == d).count();
        if count > best_count {
            best_count = count;
            best_delim = d;
        }
    }

    let delim_byte = best_delim as u8;
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(delim_byte)
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| AppError::InternalError(format!("Failed to parse CSV: {}", e)))?;

    let headers = csv_reader.headers()
        .map_err(|e| AppError::InternalError(format!("Failed to read CSV header: {}", e)))?;
    let columns: Vec<String> = headers.iter().map(|s| s.trim().to_string()).collect();

    let mut sample_rows = Vec::new();
    for rec_res in csv_reader.records().take(10) {
        if let Ok(rec) = rec_res {
            let row_vals: Vec<Value> = rec.iter().map(|s| {
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
            }).collect();
            sample_rows.push(row_vals);
        }
    }

    let detected_format = if best_delim == '\t' { "tsv" } else { "csv" };

    Ok(FileInspectionResult {
        detected_format: detected_format.to_string(),
        delimiter: Some(best_delim.to_string()),
        has_header: true,
        columns,
        sample_rows,
        total_bytes,
        sheet_names: None,
    })
}

fn inspect_sql_file(_path: &Path, total_bytes: u64) -> Result<FileInspectionResult, AppError> {
    Ok(FileInspectionResult {
        detected_format: "sql".to_string(),
        delimiter: None,
        has_header: false,
        columns: vec![],
        sample_rows: vec![],
        total_bytes,
        sheet_names: None,
    })
}
