pub mod client;
pub mod context;

pub use client::{AiClient, AiProviderConfig, AiSqlResponse, OllamaModelInfo};
pub use context::{build_schema_context, build_system_prompt};
