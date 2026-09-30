use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::drivers::DatabaseAdapter;
use crate::error::AppError;
use crate::models::connection::ConnectionConfig;

pub struct AppState {
    pub pools: Arc<RwLock<HashMap<String, Box<dyn DatabaseAdapter>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_adapter(&self, id: &str) -> Result<Arc<tokio::sync::RwLock<HashMap<String, Box<dyn DatabaseAdapter>>>>, AppError> {
        Ok(self.pools.clone())
    }
}
