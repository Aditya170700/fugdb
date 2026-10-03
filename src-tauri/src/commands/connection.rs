use tauri::State;
use std::time::Instant;
use std::sync::Arc;

use crate::drivers::{
    postgres::PostgresAdapter, 
    mysql::MySqlAdapter, 
    sqlite::SqliteAdapter, 
    mssql::MssqlAdapter,
    DatabaseAdapter
};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DriverType, TestConnectionResult};
use crate::state::AppState;
use crate::keyring;

#[tauri::command]
pub async fn save_keyring_credential(key: String, secret: String) -> Result<(), AppError> {
    keyring::set_secret(&key, &secret)
}

#[tauri::command]
pub async fn get_keyring_credential(key: String) -> Result<Option<String>, AppError> {
    keyring::get_secret(&key)
}

#[tauri::command]
pub async fn delete_keyring_credential(key: String) -> Result<bool, AppError> {
    keyring::delete_secret(&key)
}

fn resolve_effective_config(mut config: ConnectionConfig) -> ConnectionConfig {
    // If password is not provided or empty, attempt lookup from OS Keyring
    if config.password.is_none() || config.password.as_deref() == Some("") {
        if let Ok(Some(secret)) = keyring::get_secret(&format!("conn_pwd_{}", &config.id)) {
            config.password = Some(secret);
        }
    }
    config
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<TestConnectionResult, AppError> {
    let start = Instant::now();
    let effective_config = resolve_effective_config(config);

    let adapter: Box<dyn DatabaseAdapter> = match effective_config.driver {
        DriverType::Postgres => Box::new(PostgresAdapter::new(&effective_config).await?),
        DriverType::Mysql => Box::new(MySqlAdapter::new(&effective_config).await?),
        DriverType::Sqlite => Box::new(SqliteAdapter::new(&effective_config).await?),
        DriverType::Mssql => Box::new(MssqlAdapter::new(&effective_config).await?),
        _ => return Err(AppError::ConnectionError("Driver not supported yet".into())),
    };

    adapter.ping().await?;
    let latency = start.elapsed().as_millis();

    Ok(TestConnectionResult {
        success: true,
        message: "Connection test succeeded!".into(),
        latency_ms: latency,
    })
}

#[tauri::command]
pub async fn connect_database(
    config: ConnectionConfig,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let effective_config = resolve_effective_config(config);

    // Save to OS Keyring if password is provided
    if let Some(ref pwd) = effective_config.password {
        if !pwd.is_empty() {
            let _ = keyring::set_secret(&format!("conn_pwd_{}", &effective_config.id), pwd);
        }
    }

    let adapter: Box<dyn DatabaseAdapter> = match effective_config.driver {
        DriverType::Postgres => Box::new(PostgresAdapter::new(&effective_config).await?),
        DriverType::Mysql => Box::new(MySqlAdapter::new(&effective_config).await?),
        DriverType::Sqlite => Box::new(SqliteAdapter::new(&effective_config).await?),
        DriverType::Mssql => Box::new(MssqlAdapter::new(&effective_config).await?),
        _ => return Err(AppError::ConnectionError("Driver not supported yet".into())),
    };

    let mut pools = state.pools.write().await;
    pools.insert(effective_config.id.clone(), Arc::new(adapter));

    Ok(())
}

#[tauri::command]
pub async fn disconnect_database(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut pools = state.pools.write().await;
    pools.remove(&connection_id);
    Ok(())
}

