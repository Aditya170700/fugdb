use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerProcess {
    pub pid: String,
    pub user: String,
    pub database: String,
    pub client_addr: Option<String>,
    pub application_name: Option<String>,
    pub state: String,
    pub query: Option<String>,
    pub duration_seconds: f64,
    pub wait_event: Option<String>,
    pub blocked_by: Option<String>,
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerHealthStats {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_connections: u32,
    pub max_connections: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub version: String,
    pub processes: Vec<ServerProcess>,
    pub summary_counts: HashMap<String, u32>,
}
