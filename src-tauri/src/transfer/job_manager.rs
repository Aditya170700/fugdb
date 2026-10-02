use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct JobManager {
    active_jobs: Arc<RwLock<HashMap<String, Arc<AtomicBool>>>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_job(&self, job_id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        let mut map = self.active_jobs.write().await;
        map.insert(job_id.to_string(), flag.clone());
        flag
    }

    pub async fn cancel_job(&self, job_id: &str) -> bool {
        let map = self.active_jobs.read().await;
        if let Some(flag) = map.get(job_id) {
            flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub async fn unregister_job(&self, job_id: &str) {
        let mut map = self.active_jobs.write().await;
        map.remove(job_id);
    }
}
