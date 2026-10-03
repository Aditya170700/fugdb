use tauri::State;
use std::collections::HashMap;
use crate::error::AppError;
use crate::models::monitor::{ServerProcess, ServerHealthStats};
use crate::state::AppState;

fn get_val<'a>(cols: &[String], row: &'a [serde_json::Value], key: &str) -> Option<&'a serde_json::Value> {
    let lower_key = key.to_lowercase();
    let idx = cols.iter().position(|c| c.to_lowercase() == lower_key)?;
    row.get(idx)
}

fn get_string(cols: &[String], row: &[serde_json::Value], key: &str) -> String {
    match get_val(cols, row, key) {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn get_opt_string(cols: &[String], row: &[serde_json::Value], key: &str) -> Option<String> {
    let s = get_string(cols, row, key);
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn get_f64(cols: &[String], row: &[serde_json::Value], key: &str) -> f64 {
    match get_val(cols, row, key) {
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(serde_json::Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

#[tauri::command]
pub async fn get_server_processes(
    connection_id: String,
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<ServerHealthStats, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let drv = driver.as_deref().unwrap_or("").to_lowercase();

    let (proc_sql, max_conn_sql, ver_sql) = match drv.as_str() {
        "postgres" | "postgresql" => (
            r#"
            SELECT 
                pid::text as pid,
                coalesce(usename, '') as "user",
                coalesce(datname, '') as "database",
                coalesce(client_addr::text, 'local') as client_addr,
                coalesce(application_name, '') as application_name,
                coalesce(state, 'unknown') as state,
                coalesce(query, '') as query,
                coalesce(round(extract(epoch from (clock_timestamp() - query_start))::numeric, 2), 0)::float8 as duration_seconds,
                coalesce(wait_event_type || ': ' || wait_event, wait_event_type, wait_event, '') as wait_event,
                (SELECT coalesce(string_agg(blocking_pids.pid::text, ', '), '')
                 FROM pg_locks blocked_locks
                 JOIN pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
                 JOIN pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
                     AND blocking_locks.database IS NOT DISTINCT FROM blocked_locks.database
                     AND blocking_locks.relation IS NOT DISTINCT FROM blocked_locks.relation
                     AND blocking_locks.page IS NOT DISTINCT FROM blocked_locks.page
                     AND blocking_locks.tuple IS NOT DISTINCT FROM blocked_locks.tuple
                     AND blocking_locks.virtualxid IS NOT DISTINCT FROM blocked_locks.virtualxid
                     AND blocking_locks.transactionid IS NOT DISTINCT FROM blocked_locks.transactionid
                     AND blocking_locks.classid IS NOT DISTINCT FROM blocked_locks.classid
                     AND blocking_locks.objid IS NOT DISTINCT FROM blocked_locks.objid
                     AND blocking_locks.objsubid IS NOT DISTINCT FROM blocked_locks.objsubid
                     AND blocking_locks.pid != blocked_locks.pid
                 JOIN pg_stat_activity blocking_pids ON blocking_pids.pid = blocking_locks.pid
                 WHERE blocked_locks.pid = pg_stat_activity.pid
                ) as blocked_by,
                to_char(query_start, 'YYYY-MM-DD HH24:MI:SS') as started_at
            FROM pg_stat_activity
            WHERE pid <> pg_backend_pid()
            ORDER BY duration_seconds DESC;
            "#,
            Some("SHOW max_connections;"),
            Some("SELECT version();"),
        ),
        "mysql" | "mariadb" => (
            r#"
            SELECT 
                CAST(ID AS CHAR) as pid,
                coalesce(USER, '') as "user",
                coalesce(DB, '') as "database",
                coalesce(HOST, '') as client_addr,
                '' as application_name,
                coalesce(COMMAND, 'unknown') as state,
                coalesce(INFO, '') as query,
                coalesce(TIME, 0) as duration_seconds,
                coalesce(STATE, '') as wait_event,
                '' as blocked_by,
                '' as started_at
            FROM information_schema.processlist
            WHERE ID <> CONNECTION_ID()
            ORDER BY duration_seconds DESC;
            "#,
            Some("SHOW VARIABLES LIKE 'max_connections';"),
            Some("SELECT VERSION();"),
        ),
        "mssql" | "sqlserver" => (
            r#"
            SELECT 
                CAST(s.session_id AS VARCHAR(20)) as pid,
                coalesce(s.login_name, '') as [user],
                coalesce(DB_NAME(r.database_id), DB_NAME(s.database_id), '') as [database],
                coalesce(c.client_net_address, s.host_name, 'local') as client_addr,
                coalesce(s.program_name, '') as application_name,
                coalesce(r.status, s.status) as [state],
                coalesce(t.text, '') as query,
                CAST(coalesce(r.total_elapsed_time / 1000.0, 0) AS FLOAT) as duration_seconds,
                coalesce(r.wait_type, '') as wait_event,
                CASE WHEN r.blocking_session_id IS NOT NULL AND r.blocking_session_id > 0 
                     THEN CAST(r.blocking_session_id AS VARCHAR(20)) 
                     ELSE '' END as blocked_by,
                CONVERT(VARCHAR(19), coalesce(r.start_time, s.last_request_start_time), 120) as started_at
            FROM sys.dm_exec_sessions s
            OUTER APPLY (
                SELECT TOP 1 client_net_address 
                FROM sys.dm_exec_connections 
                WHERE session_id = s.session_id
            ) c
            OUTER APPLY (
                SELECT TOP 1 database_id, status, sql_handle, total_elapsed_time, wait_type, blocking_session_id, start_time
                FROM sys.dm_exec_requests 
                WHERE session_id = s.session_id
                ORDER BY total_elapsed_time DESC
            ) r
            OUTER APPLY sys.dm_exec_sql_text(r.sql_handle) t
            WHERE s.session_id <> @@SPID AND s.is_user_process = 1
            ORDER BY duration_seconds DESC;
            "#,
            Some("SELECT @@MAX_CONNECTIONS;"),
            Some("SELECT @@VERSION;"),
        ),
        "sqlite" => (
            "PRAGMA database_list;",
            None,
            Some("SELECT sqlite_version();"),
        ),
        _ => (
            "SELECT 1 as pid, 'local' as user, 'main' as database, 'active' as state, 0 as duration_seconds;",
            None,
            None,
        ),
    };

    let mut processes: Vec<ServerProcess> = Vec::new();

    if drv == "sqlite" {
        // Build SQLite specialized info
        let version_res = if let Some(v_sql) = ver_sql {
            adapter.execute_query(v_sql, None, None).await.ok()
        } else {
            None
        };
        let ver = version_res
            .and_then(|r| r.rows.first()?.first()?.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "SQLite 3.x".to_string());

        processes.push(ServerProcess {
            pid: "1".to_string(),
            user: "local".to_string(),
            database: "main".to_string(),
            client_addr: Some("embedded (in-process)".to_string()),
            application_name: Some("FugDB".to_string()),
            state: "active".to_string(),
            query: Some("PRAGMA journal_mode;".to_string()),
            duration_seconds: 0.0,
            wait_event: None,
            blocked_by: None,
            started_at: None,
        });

        let mut summary_counts = HashMap::new();
        summary_counts.insert("active".to_string(), 1);

        return Ok(ServerHealthStats {
            active_connections: 1,
            idle_connections: 0,
            total_connections: 1,
            max_connections: Some(1),
            uptime_seconds: None,
            version: ver,
            processes,
            summary_counts,
        });
    }

    // Execute query to get processes
    let res = adapter.execute_query(proc_sql, Some(500), None).await?;
    let col_names: Vec<String> = res.columns.iter().map(|c| c.name.clone()).collect();

    let mut active_count: u32 = 0;
    let mut idle_count: u32 = 0;
    let mut summary_counts: HashMap<String, u32> = HashMap::new();

    for row in res.rows {
        let pid = get_string(&col_names, &row, "pid");
        if pid.is_empty() {
            continue;
        }
        let user = get_string(&col_names, &row, "user");
        let database = get_string(&col_names, &row, "database");
        let client_addr = get_opt_string(&col_names, &row, "client_addr");
        let application_name = get_opt_string(&col_names, &row, "application_name");
        let state_val = get_string(&col_names, &row, "state").to_lowercase();
        let query = get_opt_string(&col_names, &row, "query");
        let duration_seconds = get_f64(&col_names, &row, "duration_seconds");
        let wait_event = get_opt_string(&col_names, &row, "wait_event");
        let blocked_by = get_opt_string(&col_names, &row, "blocked_by");
        let started_at = get_opt_string(&col_names, &row, "started_at");

        if state_val.contains("active") || state_val.contains("run") || state_val.contains("exec") || state_val.contains("query") {
            active_count += 1;
        } else if state_val.contains("idle") || state_val.contains("sleep") {
            idle_count += 1;
        }

        *summary_counts.entry(state_val.clone()).or_insert(0) += 1;

        processes.push(ServerProcess {
            pid,
            user,
            database,
            client_addr,
            application_name,
            state: if state_val.is_empty() { "unknown".to_string() } else { state_val },
            query,
            duration_seconds,
            wait_event,
            blocked_by,
            started_at,
        });
    }

    // Fetch version
    let mut version = format!("{drv} database engine");
    if let Some(v_sql) = ver_sql {
        if let Ok(v_res) = adapter.execute_query(v_sql, None, None).await {
            if let Some(first_row) = v_res.rows.first() {
                if let Some(val) = first_row.first() {
                    let s = match val {
                        serde_json::Value::String(s) => s.clone(),
                        _ => val.to_string(),
                    };
                    if !s.is_empty() {
                        version = s.lines().next().unwrap_or(&s).to_string();
                    }
                }
            }
        }
    }

    // Fetch max connections
    let mut max_connections = None;
    if let Some(m_sql) = max_conn_sql {
        if let Ok(m_res) = adapter.execute_query(m_sql, None, None).await {
            if let Some(first_row) = m_res.rows.first() {
                // In MySQL: ["max_connections", "151"], In PG: ["100"], In MSSQL: [32767]
                for cell in first_row {
                    if let Some(num) = cell.as_u64() {
                        max_connections = Some(num as u32);
                        break;
                    } else if let Some(s) = cell.as_str() {
                        if let Ok(num) = s.parse::<u32>() {
                            max_connections = Some(num);
                            break;
                        }
                    }
                }
            }
        }
    }

    let total_connections = processes.len() as u32;

    Ok(ServerHealthStats {
        active_connections: active_count,
        idle_connections: idle_count,
        total_connections,
        max_connections,
        uptime_seconds: None,
        version,
        processes,
        summary_counts,
    })
}

#[tauri::command]
pub async fn kill_server_process(
    connection_id: String,
    pid: String,
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let clean_pid = pid.trim().replace('\'', "").replace('"', "").replace(';', "");
    let drv = driver.as_deref().unwrap_or("").to_lowercase();

    let sql = match drv.as_str() {
        "postgres" | "postgresql" => format!("SELECT pg_terminate_backend({});", clean_pid),
        "mysql" | "mariadb" => format!("KILL {};", clean_pid),
        "mssql" | "sqlserver" => format!("KILL {};", clean_pid),
        _ => return Err(AppError::QueryError(format!("Killing processes is not supported for driver: {drv}"))),
    };

    adapter.execute_query(&sql, None, None).await?;
    Ok(true)
}

#[tauri::command]
pub async fn cancel_server_query(
    connection_id: String,
    pid: String,
    driver: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    let pools = state.pools.read().await;
    let adapter = pools
        .get(&connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.clone()))?;

    let clean_pid = pid.trim().replace('\'', "").replace('"', "").replace(';', "");
    let drv = driver.as_deref().unwrap_or("").to_lowercase();

    let sql = match drv.as_str() {
        "postgres" | "postgresql" => format!("SELECT pg_cancel_backend({});", clean_pid),
        "mysql" | "mariadb" => format!("KILL QUERY {};", clean_pid),
        "mssql" | "sqlserver" => format!("KILL {};", clean_pid),
        _ => return Err(AppError::QueryError(format!("Cancelling query is not supported for driver: {drv}"))),
    };

    adapter.execute_query(&sql, None, None).await?;
    Ok(true)
}
