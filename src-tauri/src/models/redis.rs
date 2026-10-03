use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisKeyItem {
    pub key: String,
    pub key_type: String, // "string", "hash", "list", "set", "zset", "stream"
    pub ttl: i64,         // -1 (no expiry), -2 (doesn't exist), >=0 (seconds)
    pub size: Option<usize>, // length or number of elements
    pub memory_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisScanResult {
    pub cursor: u64,
    pub keys: Vec<RedisKeyItem>,
    pub total_keys: usize,
    pub db_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisKeyDetail {
    pub key: String,
    pub key_type: String,
    pub ttl: i64,
    pub memory_usage_bytes: Option<usize>,
    pub value_string: Option<String>,
    pub value_hash: Option<HashMap<String, String>>,
    pub value_list: Option<Vec<String>>,
    pub value_set: Option<Vec<String>>,
    pub value_zset: Option<Vec<RedisZSetMember>>,
    pub value_stream: Option<Vec<RedisStreamEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisZSetMember {
    pub member: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisStreamEntry {
    pub id: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisServerInfo {
    pub version: String,
    pub os: String,
    pub uptime_seconds: u64,
    pub connected_clients: u32,
    pub used_memory_human: String,
    pub used_memory_peak_human: String,
    pub total_keys: usize,
    pub raw_info: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisCliResponse {
    pub command: String,
    pub response: String,
    pub response_type: String, // "string", "array", "integer", "status", "error"
    pub duration_ms: f64,
}
