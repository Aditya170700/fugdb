use std::collections::HashMap;
use std::time::Instant;
use tauri::State;
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, TestConnectionResult};
use crate::models::redis::{
    RedisCliResponse, RedisKeyDetail, RedisKeyItem, RedisScanResult, RedisServerInfo,
    RedisStreamEntry, RedisZSetMember,
};
use crate::state::AppState;

pub fn build_redis_url(config: &ConnectionConfig) -> String {
    let host = config.host.as_deref().unwrap_or("127.0.0.1");
    let port = config.port.unwrap_or(6379);
    let db = config.database.as_deref().unwrap_or("0");

    let auth = match (&config.username, &config.password) {
        (Some(user), Some(pwd)) if !user.is_empty() && !pwd.is_empty() => format!("{}:{}@", user, pwd),
        (_, Some(pwd)) if !pwd.is_empty() => format!(":{}@", pwd),
        _ => String::new(),
    };

    format!("redis://{}{}:{}/{}", auth, host, port, db)
}

async fn get_redis_conn(
    connection_id: &str,
    db_index: Option<u32>,
    state: &State<'_, AppState>,
) -> Result<redis::aio::MultiplexedConnection, AppError> {
    let clients = state.redis_clients.read().await;
    let client = clients
        .get(connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;

    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::ConnectionError(format!("Redis connection failed: {}", e)))?;

    if let Some(db) = db_index {
        let _: () = redis::cmd("SELECT")
            .arg(db)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::QueryError(format!("Failed to select DB {}: {}", db, e)))?;
    }

    Ok(conn)
}

#[tauri::command]
pub async fn test_redis_connection(config: ConnectionConfig) -> Result<TestConnectionResult, AppError> {
    let start = Instant::now();
    let url = build_redis_url(&config);
    let client = redis::Client::open(url.as_str())
        .map_err(|e| AppError::ConnectionError(format!("Invalid Redis URL: {}", e)))?;

    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::ConnectionError(format!("Could not connect to Redis: {}", e)))?;

    let pong: String = redis::cmd("PING")
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::ConnectionError(format!("Redis PING failed: {}", e)))?;

    let latency = start.elapsed().as_millis();
    Ok(TestConnectionResult {
        success: true,
        message: format!("Redis server responded '{}' successfully in {} ms!", pong, latency),
        latency_ms: latency,
    })
}

#[tauri::command]
pub async fn connect_redis(
    config: ConnectionConfig,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let url = build_redis_url(&config);
    let client = redis::Client::open(url.as_str())
        .map_err(|e| AppError::ConnectionError(format!("Invalid Redis URL: {}", e)))?;

    // Verify ping
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::ConnectionError(format!("Could not connect to Redis: {}", e)))?;

    let _: String = redis::cmd("PING")
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::ConnectionError(format!("Redis PING failed: {}", e)))?;

    let mut clients = state.redis_clients.write().await;
    clients.insert(config.id.clone(), client);
    Ok(())
}

#[tauri::command]
pub async fn disconnect_redis(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut clients = state.redis_clients.write().await;
    clients.remove(&connection_id);
    Ok(())
}

#[tauri::command]
pub async fn scan_redis_keys(
    connection_id: String,
    pattern: Option<String>,
    db_index: Option<u32>,
    cursor: Option<u64>,
    count: Option<usize>,
    state: State<'_, AppState>,
) -> Result<RedisScanResult, AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let scan_pattern = pattern.unwrap_or_else(|| "*".to_string());
    let cur = cursor.unwrap_or(0);
    let scan_count = count.unwrap_or(150);

    let (next_cursor, raw_keys): (u64, Vec<String>) = redis::cmd("SCAN")
        .arg(cur)
        .arg("MATCH")
        .arg(&scan_pattern)
        .arg("COUNT")
        .arg(scan_count)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("SCAN failed: {}", e)))?;

    let total_keys: usize = redis::cmd("DBSIZE")
        .query_async(&mut conn)
        .await
        .unwrap_or(raw_keys.len());

    let mut keys: Vec<RedisKeyItem> = Vec::new();

    for key_name in raw_keys {
        let key_type: String = redis::cmd("TYPE")
            .arg(&key_name)
            .query_async(&mut conn)
            .await
            .unwrap_or_else(|_| "string".to_string());

        let ttl: i64 = redis::cmd("TTL")
            .arg(&key_name)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        let size: Option<usize> = match key_type.as_str() {
            "string" => redis::cmd("STRLEN").arg(&key_name).query_async(&mut conn).await.ok(),
            "hash" => redis::cmd("HLEN").arg(&key_name).query_async(&mut conn).await.ok(),
            "list" => redis::cmd("LLEN").arg(&key_name).query_async(&mut conn).await.ok(),
            "set" => redis::cmd("SCARD").arg(&key_name).query_async(&mut conn).await.ok(),
            "zset" => redis::cmd("ZCARD").arg(&key_name).query_async(&mut conn).await.ok(),
            "stream" => redis::cmd("XLEN").arg(&key_name).query_async(&mut conn).await.ok(),
            _ => None,
        };

        let memory_bytes: Option<usize> = redis::cmd("MEMORY")
            .arg("USAGE")
            .arg(&key_name)
            .query_async(&mut conn)
            .await
            .ok();

        keys.push(RedisKeyItem {
            key: key_name,
            key_type,
            ttl,
            size,
            memory_bytes,
        });
    }

    Ok(RedisScanResult {
        cursor: next_cursor,
        keys,
        total_keys,
        db_index: db_index.unwrap_or(0),
    })
}

#[tauri::command]
pub async fn get_redis_key_detail(
    connection_id: String,
    key: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<RedisKeyDetail, AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;

    let key_type: String = redis::cmd("TYPE")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("TYPE failed: {}", e)))?;

    let ttl: i64 = redis::cmd("TTL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap_or(-1);

    let memory_usage_bytes: Option<usize> = redis::cmd("MEMORY")
        .arg("USAGE")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .ok();

    let mut detail = RedisKeyDetail {
        key: key.clone(),
        key_type: key_type.clone(),
        ttl,
        memory_usage_bytes,
        value_string: None,
        value_hash: None,
        value_list: None,
        value_set: None,
        value_zset: None,
        value_stream: None,
    };

    match key_type.as_str() {
        "string" => {
            let val: Option<String> = redis::cmd("GET").arg(&key).query_async(&mut conn).await.ok();
            detail.value_string = val;
        }
        "hash" => {
            let hash_map: HashMap<String, String> = redis::cmd("HGETALL")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .unwrap_or_default();
            detail.value_hash = Some(hash_map);
        }
        "list" => {
            let list_items: Vec<String> = redis::cmd("LRANGE")
                .arg(&key)
                .arg(0)
                .arg(500)
                .query_async(&mut conn)
                .await
                .unwrap_or_default();
            detail.value_list = Some(list_items);
        }
        "set" => {
            let set_members: Vec<String> = redis::cmd("SMEMBERS")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .unwrap_or_default();
            detail.value_set = Some(set_members);
        }
        "zset" => {
            let zset_raw: Vec<(String, f64)> = redis::cmd("ZRANGE")
                .arg(&key)
                .arg(0)
                .arg(500)
                .arg("WITHSCORES")
                .query_async(&mut conn)
                .await
                .unwrap_or_default();

            detail.value_zset = Some(
                zset_raw
                    .into_iter()
                    .map(|(member, score)| RedisZSetMember { member, score })
                    .collect(),
            );
        }
        "stream" => {
            // Read stream entries
            let raw_stream: redis::Value = redis::cmd("XREVRANGE")
                .arg(&key)
                .arg("+")
                .arg("-")
                .arg("COUNT")
                .arg(50)
                .query_async(&mut conn)
                .await
                .unwrap_or(redis::Value::Nil);

            let mut entries: Vec<RedisStreamEntry> = Vec::new();
            if let redis::Value::Array(items) = raw_stream {
                for item in items {
                    if let redis::Value::Array(entry_parts) = item {
                        if entry_parts.len() >= 2 {
                            let entry_id = match &entry_parts[0] {
                                redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
                                _ => String::new(),
                            };
                            let mut fields = HashMap::new();
                            if let redis::Value::Array(field_list) = &entry_parts[1] {
                                for chunk in field_list.chunks(2) {
                                    if chunk.len() == 2 {
                                        let k = match &chunk[0] {
                                            redis::Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
                                            _ => String::new(),
                                        };
                                        let v = match &chunk[1] {
                                            redis::Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
                                            _ => String::new(),
                                        };
                                        if !k.is_empty() {
                                            fields.insert(k, v);
                                        }
                                    }
                                }
                            }
                            entries.push(RedisStreamEntry { id: entry_id, fields });
                        }
                    }
                }
            }
            detail.value_stream = Some(entries);
        }
        _ => {}
    }

    Ok(detail)
}

#[tauri::command]
pub async fn set_redis_string(
    connection_id: String,
    key: String,
    value: String,
    ttl: Option<i64>,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let mut cmd = redis::cmd("SET");
    cmd.arg(&key).arg(&value);

    if let Some(seconds) = ttl {
        if seconds > 0 {
            cmd.arg("EX").arg(seconds);
        }
    }

    let _: () = cmd
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("SET failed: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn set_redis_hash_field(
    connection_id: String,
    key: String,
    field: String,
    value: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("HSET")
        .arg(&key)
        .arg(&field)
        .arg(&value)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("HSET failed: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn delete_redis_hash_field(
    connection_id: String,
    key: String,
    field: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("HDEL")
        .arg(&key)
        .arg(&field)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("HDEL failed: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn push_redis_list_element(
    connection_id: String,
    key: String,
    value: String,
    position: String, // "left" or "right"
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let op = if position.to_lowercase() == "left" { "LPUSH" } else { "RPUSH" };
    let _: () = redis::cmd(op)
        .arg(&key)
        .arg(&value)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("{} failed: {}", op, e)))?;
    Ok(())
}

#[tauri::command]
pub async fn remove_redis_list_element(
    connection_id: String,
    key: String,
    value: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("LREM")
        .arg(&key)
        .arg(1)
        .arg(&value)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("LREM failed: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn add_redis_set_member(
    connection_id: String,
    key: String,
    member: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("SADD")
        .arg(&key)
        .arg(&member)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("SADD failed: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn remove_redis_set_member(
    connection_id: String,
    key: String,
    member: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("SREM")
        .arg(&key)
        .arg(&member)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("SREM failed: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn add_redis_zset_member(
    connection_id: String,
    key: String,
    member: String,
    score: f64,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("ZADD")
        .arg(&key)
        .arg(score)
        .arg(&member)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("ZADD failed: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn remove_redis_zset_member(
    connection_id: String,
    key: String,
    member: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("ZREM")
        .arg(&key)
        .arg(&member)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("ZREM failed: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn set_redis_key_ttl(
    connection_id: String,
    key: String,
    ttl: i64,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    if ttl < 0 {
        // Persist (remove TTL)
        let _: () = redis::cmd("PERSIST")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::QueryError(format!("PERSIST failed: {}", e)))?;
    } else {
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(ttl)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::QueryError(format!("EXPIRE failed: {}", e)))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_redis_keys(
    connection_id: String,
    keys: Vec<String>,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<usize, AppError> {
    if keys.is_empty() {
        return Ok(0);
    }
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let mut cmd = redis::cmd("DEL");
    for k in &keys {
        cmd.arg(k);
    }
    let deleted_count: usize = cmd
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("DEL failed: {}", e)))?;
    Ok(deleted_count)
}

#[tauri::command]
pub async fn rename_redis_key(
    connection_id: String,
    old_key: String,
    new_key: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("RENAME")
        .arg(&old_key)
        .arg(&new_key)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("RENAME failed: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn get_redis_server_info(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<RedisServerInfo, AppError> {
    let mut conn = get_redis_conn(&connection_id, None, &state).await?;
    let raw: String = redis::cmd("INFO")
        .query_async(&mut conn)
        .await
        .unwrap_or_default();

    let mut raw_info = HashMap::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            raw_info.insert(k.to_string(), v.to_string());
        }
    }

    let version = raw_info.get("redis_version").cloned().unwrap_or_else(|| "Redis 7.x".into());
    let os = raw_info.get("os").cloned().unwrap_or_else(|| "Linux/Unix".into());
    let uptime_seconds: u64 = raw_info
        .get("uptime_in_seconds")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let connected_clients: u32 = raw_info
        .get("connected_clients")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let used_memory_human = raw_info.get("used_memory_human").cloned().unwrap_or_else(|| "1.2MB".into());
    let used_memory_peak_human = raw_info.get("used_memory_peak_human").cloned().unwrap_or_else(|| "1.8MB".into());

    let total_keys: usize = redis::cmd("DBSIZE")
        .query_async(&mut conn)
        .await
        .unwrap_or(0);

    Ok(RedisServerInfo {
        version,
        os,
        uptime_seconds,
        connected_clients,
        used_memory_human,
        used_memory_peak_human,
        total_keys,
        raw_info,
    })
}

#[tauri::command]
pub async fn execute_redis_cli_command(
    connection_id: String,
    command_line: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<RedisCliResponse, AppError> {
    let start = Instant::now();
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;

    // Parse command tokens
    let tokens: Vec<String> = command_line
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if tokens.is_empty() {
        return Err(AppError::QueryError("Empty Redis command".into()));
    }

    let main_cmd = &tokens[0];
    let mut cmd = redis::cmd(main_cmd);
    for arg in &tokens[1..] {
        cmd.arg(arg);
    }

    let raw_val: redis::Value = cmd
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("ERR: {}", e)))?;

    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    fn format_redis_val(val: &redis::Value) -> (String, String) {
        match val {
            redis::Value::Nil => ("(nil)".into(), "nil".into()),
            redis::Value::Int(i) => (format!("(integer) {}", i), "integer".into()),
            redis::Value::BulkString(bytes) => (String::from_utf8_lossy(bytes).to_string(), "string".into()),
            redis::Value::Array(items) => {
                let lines: Vec<String> = items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let (formatted, _) = format_redis_val(item);
                        format!("{}) {}", i + 1, formatted)
                    })
                    .collect();
                (lines.join("\n"), "array".into())
            }
            redis::Value::SimpleString(s) => (format!("OK: {}", s), "status".into()),
            redis::Value::Okay => ("OK".into(), "status".into()),
            _ => (format!("{:?}", val), "string".into()),
        }
    }

    let (response, response_type) = format_redis_val(&raw_val);

    Ok(RedisCliResponse {
        command: command_line,
        response,
        response_type,
        duration_ms,
    })
}

#[tauri::command]
pub async fn flush_redis_db(
    connection_id: String,
    db_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = get_redis_conn(&connection_id, db_index, &state).await?;
    let _: () = redis::cmd("FLUSHDB")
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::QueryError(format!("FLUSHDB failed: {}", e)))?;
    Ok(())
}
