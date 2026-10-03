use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DriverType {
    Postgres,
    Mysql,
    Sqlite,
    Mssql,
    Duckdb,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Dev,
    Staging,
    Production,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub driver: DriverType,
    pub environment: Environment,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub file_path: Option<String>,
    pub use_ssh: Option<bool>,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    pub ssh_auth_type: Option<String>,
    pub ssh_password: Option<String>,
    pub ssh_key_path: Option<String>,
    pub ssh_key_passphrase: Option<String>,
    pub ssl_mode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: u128,
}
