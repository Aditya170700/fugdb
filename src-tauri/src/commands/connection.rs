use tauri::State;
use std::time::Instant;

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

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<TestConnectionResult, AppError> {
    let start = Instant::now();
    let adapter: Box<dyn DatabaseAdapter> = match config.driver {
        DriverType::Postgres => Box::new(PostgresAdapter::new(&config).await?),
        DriverType::Mysql => Box::new(MySqlAdapter::new(&config).await?),
        DriverType::Sqlite => Box::new(SqliteAdapter::new(&config).await?),
        DriverType::Mssql => Box::new(MssqlAdapter::new(&config).await?),
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
    let adapter: Box<dyn DatabaseAdapter> = match config.driver {
        DriverType::Postgres => Box::new(PostgresAdapter::new(&config).await?),
        DriverType::Mysql => Box::new(MySqlAdapter::new(&config).await?),
        DriverType::Sqlite => Box::new(SqliteAdapter::new(&config).await?),
        DriverType::Mssql => Box::new(MssqlAdapter::new(&config).await?),
        _ => return Err(AppError::ConnectionError("Driver not supported yet".into())),
    };

    let mut pools = state.pools.write().await;
    pools.insert(config.id.clone(), adapter);

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
