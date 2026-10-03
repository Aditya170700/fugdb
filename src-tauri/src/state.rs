use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::drivers::DatabaseAdapter;
use crate::transfer::JobManager;
use crate::ssh::SshTunnelHandle;
use crate::scheduler::SchedulerManager;

pub struct AppState {
    pub pools: Arc<RwLock<HashMap<String, Arc<Box<dyn DatabaseAdapter>>>>>,
    pub redis_clients: Arc<RwLock<HashMap<String, redis::Client>>>,
    pub tunnels: Arc<RwLock<HashMap<String, SshTunnelHandle>>>,
    pub job_manager: JobManager,
    pub scheduler: Arc<SchedulerManager>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            redis_clients: Arc::new(RwLock::new(HashMap::new())),
            tunnels: Arc::new(RwLock::new(HashMap::new())),
            job_manager: JobManager::new(),
            scheduler: Arc::new(SchedulerManager::new()),
        }
    }
}
