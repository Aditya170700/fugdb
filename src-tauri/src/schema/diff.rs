use std::collections::{HashMap, HashSet};
use crate::models::{
    diff::{ColumnDiff, DiffAction, SchemaDiffResult, TableDiff},
    query::ColumnMetadata,
    schema::{SchemaTree, TableItem},
};

fn quote_identifier(name: &str, driver: &str) -> String {
    let drv = driver.to_lowercase();
    match drv.as_str() {
        "mysql" | "mariadb" => format!("`{}`", name),
        "mssql" | "sqlserver" => format!("[{}]", name),
        _ => format!("\"{}\"", name),
    }
}

fn map_data_type(source_type: &str, target_driver: &str) -> String {
    let s = source_type.to_lowercase();
    let drv = target_driver.to_lowercase();

    match drv.as_str() {
        "mysql" | "mariadb" => {
            if s.contains("int8") || s.contains("bigint") {
                "BIGINT".to_string()
            } else if s.contains("int4") || s.contains("int") {
                "INT".to_string()
            } else if s.contains("bool") {
                "TINYINT(1)".to_string()
            } else if s.contains("timestamp") || s.contains("timestamptz") {
                "DATETIME(6)".to_string()
            } else if s.contains("json") {
                "JSON".to_string()
            } else if s.contains("uuid") {
                "VARCHAR(36)".to_string()
            } else if s.contains("text") {
                "LONGTEXT".to_string()
            } else if s.contains("float") || s.contains("double") {
                "DOUBLE".to_string()
            } else {
                source_type.to_uppercase()
            }
        }
        "mssql" | "sqlserver" => {
            if s.contains("int8") || s.contains("bigint") {
                "BIGINT".to_string()
            } else if s.contains("int4") || s.contains("int") {
                "INT".to_string()
            } else if s.contains("bool") {
                "BIT".to_string()
            } else if s.contains("timestamp") || s.contains("timestamptz") {
                "DATETIME2".to_string()
            } else if s.contains("json") || s.contains("text") {
                "NVARCHAR(MAX)".to_string()
            } else if s.contains("uuid") {
                "UNIQUEIDENTIFIER".to_string()
            } else if s.contains("varchar") {
                "NVARCHAR(255)".to_string()
            } else {
                source_type.to_uppercase()
            }
        }
        "postgres" | "postgresql" => {
            if s.contains("int8") || s.contains("bigint") {
                "BIGINT".to_string()
            } else if s.contains("int4") || s.contains("int") {
                "INTEGER".to_string()
            } else if s.contains("bool") || s.contains("tinyint(1)") {
                "BOOLEAN".to_string()
            } else if s.contains("datetime") || s.contains("timestamp") {
                "TIMESTAMPTZ".to_string()
            } else if s.contains("json") {
                "JSONB".to_string()
            } else if s.contains("uuid") {
                "UUID".to_string()
            } else {
                source_type.to_uppercase()
            }
        }
        _ => source_type.to_uppercase(),
    }
}

fn generate_create_table_ddl(table: &TableItem, target_driver: &str) -> String {
    let q_tbl = quote_identifier(&table.name, target_driver);
    let mut cols_ddl = Vec::new();
    let mut pks = Vec::new();

    for col in &table.columns {
        let q_col = quote_identifier(&col.name, target_driver);
        let mapped_type = map_data_type(&col.data_type, target_driver);
        let null_str = if col.nullable { "" } else { " NOT NULL" };

        if col.is_primary_key {
            pks.push(q_col.clone());
        }

        cols_ddl.push(format!("  {} {}{}", q_col, mapped_type, null_str));
    }

    if !pks.is_empty() {
        cols_ddl.push(format!("  PRIMARY KEY ({})", pks.join(", ")));
    }

    format!("CREATE TABLE {} (\n{}\n);", q_tbl, cols_ddl.join(",\n"))
}

pub fn generate_schema_diff(
    source_tree: &SchemaTree,
    target_tree: &SchemaTree,
    source_conn_id: &str,
    target_conn_id: &str,
    source_driver: &str,
    target_driver: &str,
) -> SchemaDiffResult {
    let start_time = std::time::Instant::now();

    let mut source_map: HashMap<String, &TableItem> = HashMap::new();
    for t in &source_tree.tables {
        source_map.insert(t.name.to_lowercase(), t);
    }

    let mut target_map: HashMap<String, &TableItem> = HashMap::new();
    for t in &target_tree.tables {
        target_map.insert(t.name.to_lowercase(), t);
    }

    let mut all_table_names: HashSet<String> = HashSet::new();
    for k in source_map.keys() {
        all_table_names.insert(k.clone());
    }
    for k in target_map.keys() {
        all_table_names.insert(k.clone());
    }

    let mut table_names_sorted: Vec<String> = all_table_names.into_iter().collect();
    table_names_sorted.sort();

    let mut table_diffs: Vec<TableDiff> = Vec::new();
    let mut tables_to_create = 0;
    let mut tables_to_drop = 0;
    let mut tables_to_alter = 0;
    let mut tables_identical = 0;

    for tbl_key in table_names_sorted {
        let in_source = source_map.get(&tbl_key);
        let in_target = target_map.get(&tbl_key);

        match (in_source, in_target) {
            (Some(src), None) => {
                // Table in Source but missing in Target -> Create
                tables_to_create += 1;
                let cols: Vec<ColumnDiff> = src.columns
                    .iter()
                    .map(|c| ColumnDiff {
                        name: c.name.clone(),
                        action: DiffAction::Create,
                        source_type: Some(c.data_type.clone()),
                        target_type: None,
                        source_nullable: Some(c.nullable),
                        target_nullable: None,
                        source_pk: Some(c.is_primary_key),
                        target_pk: None,
                        diff_reason: Some("Table missing in target".to_string()),
                    })
                    .collect();

                let sync_sql = generate_create_table_ddl(src, target_driver);

                table_diffs.push(TableDiff {
                    table_name: src.name.clone(),
                    action: DiffAction::Create,
                    source_row_count: src.row_count_estimate.map(|r| r.max(0) as u64),
                    target_row_count: None,
                    columns: cols,
                    sync_sql,
                });
            }
            (None, Some(tgt)) => {
                // Table in Target but missing in Source -> Drop (commented for safety)
                tables_to_drop += 1;
                let cols: Vec<ColumnDiff> = tgt.columns
                    .iter()
                    .map(|c| ColumnDiff {
                        name: c.name.clone(),
                        action: DiffAction::Drop,
                        source_type: None,
                        target_type: Some(c.data_type.clone()),
                        source_nullable: None,
                        target_nullable: Some(c.nullable),
                        source_pk: None,
                        target_pk: Some(c.is_primary_key),
                        diff_reason: Some("Table exists only in target".to_string()),
                    })
                    .collect();

                let q_tbl = quote_identifier(&tgt.name, target_driver);
                let sync_sql = format!("-- DROP TABLE {};", q_tbl);

                table_diffs.push(TableDiff {
                    table_name: tgt.name.clone(),
                    action: DiffAction::Drop,
                    source_row_count: None,
                    target_row_count: tgt.row_count_estimate.map(|r| r.max(0) as u64),
                    columns: cols,
                    sync_sql,
                });
            }
            (Some(src), Some(tgt)) => {
                // Table in both -> Compare Columns
                let src_cols = &src.columns;
                let tgt_cols = &tgt.columns;

                let mut src_col_map: HashMap<String, &ColumnMetadata> = HashMap::new();
                for c in src_cols {
                    src_col_map.insert(c.name.to_lowercase(), c);
                }

                let mut tgt_col_map: HashMap<String, &ColumnMetadata> = HashMap::new();
                for c in tgt_cols {
                    tgt_col_map.insert(c.name.to_lowercase(), c);
                }

                let mut all_cols: HashSet<String> = HashSet::new();
                for k in src_col_map.keys() { all_cols.insert(k.clone()); }
                for k in tgt_col_map.keys() { all_cols.insert(k.clone()); }

                let mut sorted_cols: Vec<String> = all_cols.into_iter().collect();
                sorted_cols.sort();

                let mut col_diffs: Vec<ColumnDiff> = Vec::new();
                let mut alter_statements: Vec<String> = Vec::new();
                let q_tbl = quote_identifier(&src.name, target_driver);

                for col_key in sorted_cols {
                    let sc = src_col_map.get(&col_key);
                    let tc = tgt_col_map.get(&col_key);

                    match (sc, tc) {
                        (Some(s), None) => {
                            // Column in Source not in Target -> ADD COLUMN
                            col_diffs.push(ColumnDiff {
                                name: s.name.clone(),
                                action: DiffAction::Create,
                                source_type: Some(s.data_type.clone()),
                                target_type: None,
                                source_nullable: Some(s.nullable),
                                target_nullable: None,
                                source_pk: Some(s.is_primary_key),
                                target_pk: None,
                                diff_reason: Some("Missing column in target".to_string()),
                            });

                            let q_col = quote_identifier(&s.name, target_driver);
                            let mapped_type = map_data_type(&s.data_type, target_driver);
                            let null_str = if s.nullable { "" } else { " NOT NULL" };

                            let add_stmt = match target_driver.to_lowercase().as_str() {
                                "mssql" | "sqlserver" => format!("ALTER TABLE {} ADD {} {}{};", q_tbl, q_col, mapped_type, null_str),
                                _ => format!("ALTER TABLE {} ADD COLUMN {} {}{};", q_tbl, q_col, mapped_type, null_str),
                            };
                            alter_statements.push(add_stmt);
                        }
                        (None, Some(t)) => {
                            // Column in Target not in Source -> DROP COLUMN (Commented)
                            col_diffs.push(ColumnDiff {
                                name: t.name.clone(),
                                action: DiffAction::Drop,
                                source_type: None,
                                target_type: Some(t.data_type.clone()),
                                source_nullable: None,
                                target_nullable: Some(t.nullable),
                                source_pk: None,
                                target_pk: Some(t.is_primary_key),
                                diff_reason: Some("Column exists only in target".to_string()),
                            });

                            let q_col = quote_identifier(&t.name, target_driver);
                            alter_statements.push(format!("-- ALTER TABLE {} DROP COLUMN {};", q_tbl, q_col));
                        }
                        (Some(s), Some(t)) => {
                            let type_match = s.data_type.eq_ignore_ascii_case(&t.data_type);
                            let null_match = s.nullable == t.nullable;
                            let pk_match = s.is_primary_key == t.is_primary_key;

                            if type_match && null_match && pk_match {
                                col_diffs.push(ColumnDiff {
                                    name: s.name.clone(),
                                    action: DiffAction::Identical,
                                    source_type: Some(s.data_type.clone()),
                                    target_type: Some(t.data_type.clone()),
                                    source_nullable: Some(s.nullable),
                                    target_nullable: Some(t.nullable),
                                    source_pk: Some(s.is_primary_key),
                                    target_pk: Some(t.is_primary_key),
                                    diff_reason: None,
                                });
                            } else {
                                let mut reasons = Vec::new();
                                if !type_match { reasons.push(format!("Type changed ({} -> {})", t.data_type, s.data_type)); }
                                if !null_match { reasons.push(format!("Nullability changed ({} -> {})", t.nullable, s.nullable)); }
                                if !pk_match { reasons.push("PK status changed".to_string()); }

                                col_diffs.push(ColumnDiff {
                                    name: s.name.clone(),
                                    action: DiffAction::Alter,
                                    source_type: Some(s.data_type.clone()),
                                    target_type: Some(t.data_type.clone()),
                                    source_nullable: Some(s.nullable),
                                    target_nullable: Some(t.nullable),
                                    source_pk: Some(s.is_primary_key),
                                    target_pk: Some(t.is_primary_key),
                                    diff_reason: Some(reasons.join(", ")),
                                });

                                let q_col = quote_identifier(&s.name, target_driver);
                                let mapped_type = map_data_type(&s.data_type, target_driver);
                                let null_str = if s.nullable { "" } else { " NOT NULL" };

                                let alter_col_stmt = match target_driver.to_lowercase().as_str() {
                                    "mysql" | "mariadb" => format!("ALTER TABLE {} MODIFY COLUMN {} {}{};", q_tbl, q_col, mapped_type, null_str),
                                    "mssql" | "sqlserver" => format!("ALTER TABLE {} ALTER COLUMN {} {}{};", q_tbl, q_col, mapped_type, null_str),
                                    _ => format!("ALTER TABLE {} ALTER COLUMN {} TYPE {}{};", q_tbl, q_col, mapped_type, null_str),
                                };
                                alter_statements.push(alter_col_stmt);
                            }
                        }
                        _ => {}
                    }
                }

                let has_changes = col_diffs.iter().any(|c| c.action != DiffAction::Identical);

                if has_changes {
                    tables_to_alter += 1;
                    table_diffs.push(TableDiff {
                        table_name: src.name.clone(),
                        action: DiffAction::Alter,
                        source_row_count: src.row_count_estimate.map(|r| r.max(0) as u64),
                        target_row_count: tgt.row_count_estimate.map(|r| r.max(0) as u64),
                        columns: col_diffs,
                        sync_sql: alter_statements.join("\n"),
                    });
                } else {
                    tables_identical += 1;
                    table_diffs.push(TableDiff {
                        table_name: src.name.clone(),
                        action: DiffAction::Identical,
                        source_row_count: src.row_count_estimate.map(|r| r.max(0) as u64),
                        target_row_count: tgt.row_count_estimate.map(|r| r.max(0) as u64),
                        columns: col_diffs,
                        sync_sql: String::new(),
                    });
                }
            }
            _ => {}
        }
    }

    // Generate Full Transactional Migration Script
    let drv = target_driver.to_lowercase();
    let tx_begin = match drv.as_str() {
        "mysql" | "mariadb" => "START TRANSACTION;",
        "mssql" | "sqlserver" => "BEGIN TRANSACTION;",
        _ => "BEGIN;",
    };
    let tx_commit = match drv.as_str() {
        "mssql" | "sqlserver" => "COMMIT TRANSACTION;",
        _ => "COMMIT;",
    };

    let mut script_lines = Vec::new();
    script_lines.push("-- ===========================================================================".to_string());
    script_lines.push("-- FugDB Automated Schema Migration & Sync Script".to_string());
    script_lines.push(format!("-- Source: {} ({})", source_conn_id, source_driver));
    script_lines.push(format!("-- Target: {} ({})", target_conn_id, target_driver));
    script_lines.push(format!("-- Tables to Create: {}, Alter: {}, Drop: {}, Identical: {}", tables_to_create, tables_to_alter, tables_to_drop, tables_identical));
    script_lines.push("-- ===========================================================================\n".to_string());
    script_lines.push(format!("{}\n", tx_begin));

    for diff in &table_diffs {
        if !diff.sync_sql.trim().is_empty() {
            script_lines.push(format!("-- Table: {}", diff.table_name));
            script_lines.push(diff.sync_sql.clone());
            script_lines.push(String::new());
        }
    }

    script_lines.push(format!("{}\n", tx_commit));
    let full_migration_sql = script_lines.join("\n");

    let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    SchemaDiffResult {
        source_connection_id: source_conn_id.to_string(),
        target_connection_id: target_conn_id.to_string(),
        source_driver: source_driver.to_string(),
        target_driver: target_driver.to_string(),
        total_source_tables: source_tree.tables.len(),
        total_target_tables: target_tree.tables.len(),
        tables_to_create,
        tables_to_drop,
        tables_to_alter,
        tables_identical,
        table_diffs,
        full_migration_sql,
        execution_time_ms: duration_ms,
    }
}
