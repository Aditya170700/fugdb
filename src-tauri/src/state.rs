use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::drivers::DatabaseAdapter;

pub struct AppState {
    pub pools: Arc<RwLock<HashMap<String, Box<dyn DatabaseAdapter>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
