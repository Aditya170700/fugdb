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
use crate::ssh::SshTunnelManager;

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
    // If DB password is not provided or empty, attempt lookup from OS Keyring
    if config.password.is_none() || config.password.as_deref() == Some("") {
        if let Ok(Some(secret)) = keyring::get_secret(&format!("conn_pwd_{}", &config.id)) {
            config.password = Some(secret);
        }
    }
    // If SSH password is not provided or empty, attempt lookup from OS Keyring
    if config.ssh_password.is_none() || config.ssh_password.as_deref() == Some("") {
        if let Ok(Some(secret)) = keyring::get_secret(&format!("conn_ssh_pwd_{}", &config.id)) {
            config.ssh_password = Some(secret);
        }
    }
    // If SSH key passphrase is not provided or empty, attempt lookup from OS Keyring
    if config.ssh_key_passphrase.is_none() || config.ssh_key_passphrase.as_deref() == Some("") {
        if let Ok(Some(secret)) = keyring::get_secret(&format!("conn_ssh_pass_{}", &config.id)) {
            config.ssh_key_passphrase = Some(secret);
        }
    }
    config
}

#[tauri::command]
pub async fn test_ssh_tunnel(config: ConnectionConfig) -> Result<TestConnectionResult, AppError> {
    let start = Instant::now();
    let effective_config = resolve_effective_config(config);

    if effective_config.use_ssh != Some(true) {
        return Err(AppError::ConnectionError("SSH Tunneling is not enabled in this connection config".into()));
    }

    let session = SshTunnelManager::create_session(&effective_config)?;
    if !session.authenticated() {
        return Err(AppError::ConnectionError("SSH Authentication failed".into()));
    }

    let latency = start.elapsed().as_millis();
    let host = effective_config.ssh_host.as_deref().unwrap_or("bastion");
    let user = effective_config.ssh_user.as_deref().unwrap_or("ssh");

    Ok(TestConnectionResult {
        success: true,
        message: format!("SSH Bastion Host handshake succeeded for user '{}' on {}!", user, host),
        latency_ms: latency,
    })
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<TestConnectionResult, AppError> {
    let start = Instant::now();
    let effective_config = resolve_effective_config(config);

    // If SSH Tunneling is enabled, start a temporary tunnel for the test
    let (_temp_tunnel, target_config) = if effective_config.use_ssh == Some(true) {
        let tunnel = SshTunnelManager::start_tunnel(&effective_config).await?;
        let mut routed_cfg = effective_config.clone();
        routed_cfg.host = Some("127.0.0.1".into());
        routed_cfg.port = Some(tunnel.local_port);
        (Some(tunnel), routed_cfg)
    } else {
        (None, effective_config)
    };

    if target_config.driver == DriverType::Redis {
        return crate::commands::redis::test_redis_connection(target_config).await;
    }

    let adapter: Box<dyn DatabaseAdapter> = match target_config.driver {
        DriverType::Postgres => Box::new(PostgresAdapter::new(&target_config).await?),
        DriverType::Mysql => Box::new(MySqlAdapter::new(&target_config).await?),
        DriverType::Sqlite => Box::new(SqliteAdapter::new(&target_config).await?),
        DriverType::Mssql => Box::new(MssqlAdapter::new(&target_config).await?),
        _ => return Err(AppError::ConnectionError("Driver not supported yet".into())),
    };

    adapter.ping().await?;
    let latency = start.elapsed().as_millis();

    let prefix = if target_config.use_ssh == Some(true) {
        "Connected via SSH Bastion Tunnel! "
    } else {
        ""
    };

    Ok(TestConnectionResult {
        success: true,
        message: format!("{}Connection test succeeded!", prefix),
        latency_ms: latency,
    })
}

#[tauri::command]
pub async fn connect_database(
    config: ConnectionConfig,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let effective_config = resolve_effective_config(config);

    // Save secrets to OS Keyring if provided
    if let Some(ref pwd) = effective_config.password {
        if !pwd.is_empty() {
            let _ = keyring::set_secret(&format!("conn_pwd_{}", &effective_config.id), pwd);
        }
    }
    if let Some(ref ssh_pwd) = effective_config.ssh_password {
        if !ssh_pwd.is_empty() {
            let _ = keyring::set_secret(&format!("conn_ssh_pwd_{}", &effective_config.id), ssh_pwd);
        }
    }
    if let Some(ref ssh_pass) = effective_config.ssh_key_passphrase {
        if !ssh_pass.is_empty() {
            let _ = keyring::set_secret(&format!("conn_ssh_pass_{}", &effective_config.id), ssh_pass);
        }
    }

    // If SSH Tunneling is enabled, start and register the tunnel
    let target_config = if effective_config.use_ssh == Some(true) {
        let mut tunnels = state.tunnels.write().await;
        // Close existing tunnel if reconnecting
        if let Some(mut old) = tunnels.remove(&effective_config.id) {
            old.close();
        }

        let tunnel = SshTunnelManager::start_tunnel(&effective_config).await?;
        let local_port = tunnel.local_port;
        tunnels.insert(effective_config.id.clone(), tunnel);

        let mut routed = effective_config.clone();
        routed.host = Some("127.0.0.1".into());
        routed.port = Some(local_port);
        routed
    } else {
        effective_config.clone()
    };

    if target_config.driver == DriverType::Redis {
        return crate::commands::redis::connect_redis(target_config, state).await;
    }

    let adapter: Box<dyn DatabaseAdapter> = match target_config.driver {
        DriverType::Postgres => Box::new(PostgresAdapter::new(&target_config).await?),
        DriverType::Mysql => Box::new(MySqlAdapter::new(&target_config).await?),
        DriverType::Sqlite => Box::new(SqliteAdapter::new(&target_config).await?),
        DriverType::Mssql => Box::new(MssqlAdapter::new(&target_config).await?),
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
    // 1. Close and remove DB pool
    {
        let mut pools = state.pools.write().await;
        pools.remove(&connection_id);
    }
    // 2. Close and remove Redis client
    {
        let mut redis_clients = state.redis_clients.write().await;
        redis_clients.remove(&connection_id);
    }
    // 3. Close and remove SSH Tunnel
    {
        let mut tunnels = state.tunnels.write().await;
        if let Some(mut tunnel) = tunnels.remove(&connection_id) {
            tunnel.close();
        }
    }
    Ok(())
}


