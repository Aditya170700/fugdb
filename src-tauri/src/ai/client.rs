use std::time::Instant;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    pub provider: String, // "ollama" | "openai" | "gemini" | "anthropic" | "deepseek" | "custom"
    pub model: String,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSqlResponse {
    pub sql: String,
    pub explanation: String,
    pub tables_used: Vec<String>,
    pub dialect: String,
    pub model_used: String,
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModelInfo {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

pub struct AiClient;

impl AiClient {
    pub async fn list_ollama_models(endpoint: Option<&str>) -> Result<Vec<OllamaModelInfo>, AppError> {
        let base_url = endpoint.unwrap_or("http://localhost:11434").trim_end_matches('/');
        let url = format!("{}/api/tags", base_url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| AppError::Internal(format!("HTTP client error: {}", e)))?;

        let resp = client.get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to Ollama at {}: {}", base_url, e)))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(format!("Ollama returned HTTP {}", resp.status())));
        }

        let json_body: Value = resp.json().await
            .map_err(|e| AppError::Internal(format!("Failed to parse Ollama tags response: {}", e)))?;

        let mut models = Vec::new();
        if let Some(models_array) = json_body.get("models").and_then(|m| m.as_array()) {
            for m in models_array {
                let name = m.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string();
                let size = m.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                let modified_at = m.get("modified_at").and_then(|d| d.as_str()).unwrap_or_default().to_string();
                if !name.is_empty() {
                    models.push(OllamaModelInfo { name, size, modified_at });
                }
            }
        }

        Ok(models)
    }

    pub async fn test_connection(config: &AiProviderConfig) -> Result<String, AppError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| AppError::Internal(format!("HTTP client init failed: {}", e)))?;

        match config.provider.to_lowercase().as_str() {
            "ollama" => {
                let base_url = config.endpoint.as_deref().unwrap_or("http://localhost:11434").trim_end_matches('/');
                let url = format!("{}/api/tags", base_url);
                let res = client.get(&url).send().await
                    .map_err(|e| AppError::Internal(format!("Ollama connection failed: {}", e)))?;
                if res.status().is_success() {
                    Ok("Successfully connected to local Ollama instance!".into())
                } else {
                    let status = res.status();
                    let err_text = res.text().await.unwrap_or_default();
                    Err(AppError::Internal(format_api_error("Ollama", status, &err_text)))
                }
            }
            "openai" | "deepseek" | "custom" => {
                let api_key = config.api_key.as_deref().unwrap_or_default();
                if api_key.is_empty() && config.provider != "custom" {
                    return Err(AppError::Internal("API Key is required".into()));
                }
                let base_url = config.endpoint.as_deref().unwrap_or_else(|| {
                    if config.provider == "deepseek" {
                        "https://api.deepseek.com/v1"
                    } else {
                        "https://api.openai.com/v1"
                    }
                }).trim_end_matches('/');

                let url = format!("{}/chat/completions", base_url);
                let body = json!({
                    "model": config.model,
                    "max_tokens": 5,
                    "messages": [
                        { "role": "user", "content": "ping" }
                    ]
                });
                let mut req = client.post(&url).json(&body);
                if !api_key.is_empty() {
                    req = req.header("Authorization", format!("Bearer {}", api_key));
                }
                let res = req.send().await
                    .map_err(|e| AppError::Internal(format!("Failed to reach {}: {}", config.provider, e)))?;
                if res.status().is_success() {
                    Ok(format!("Successfully connected to {} ({})!", config.provider, config.model))
                } else {
                    let status = res.status();
                    let err_text = res.text().await.unwrap_or_default();
                    Err(AppError::Internal(format_api_error(&config.provider, status, &err_text)))
                }
            }
            "gemini" => {
                let api_key = config.api_key.as_deref().unwrap_or_default();
                if api_key.is_empty() {
                    return Err(AppError::Internal("Gemini API Key is required".into()));
                }

                let base_model = config.model.trim_start_matches("models/");
                let endpoints = [
                    format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", base_model, api_key),
                    format!("https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}", base_model, api_key),
                    format!("https://generativelanguage.googleapis.com/v1beta/models/{}-latest:generateContent?key={}", base_model, api_key),
                    format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash-latest:generateContent?key={}", api_key),
                    format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", api_key),
                ];

                let body = json!({
                    "contents": [{ "parts": [{ "text": "ping" }] }],
                    "generationConfig": { "maxOutputTokens": 5 }
                });

                let mut last_err = String::new();
                let mut connected = false;
                for url in &endpoints {
                    if let Ok(res) = client.post(url).json(&body).send().await {
                        if res.status().is_success() {
                            connected = true;
                            break;
                        } else {
                            let status = res.status();
                            let err_text = res.text().await.unwrap_or_default();
                            last_err = format_api_error("Gemini", status, &err_text);
                        }
                    }
                }

                if connected {
                    Ok(format!("Successfully connected to Google Gemini API ({})!", config.model))
                } else if !last_err.is_empty() {
                    Err(AppError::Internal(last_err))
                } else {
                    Ok("Successfully connected to Google Gemini API!".into())
                }
            }
            "anthropic" => {
                let api_key = config.api_key.as_deref().unwrap_or_default();
                if api_key.is_empty() {
                    return Err(AppError::Internal("Anthropic API Key is required".into()));
                }
                // Test lightweight message
                let body = json!({
                    "model": config.model,
                    "max_tokens": 5,
                    "messages": [{"role": "user", "content": "ping"}]
                });
                let res = client.post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::Internal(format!("Anthropic connection failed: {}", e)))?;
                if res.status().is_success() {
                    Ok("Successfully connected to Anthropic Claude API!".into())
                } else {
                    let status = res.status();
                    let err_text = res.text().await.unwrap_or_default();
                    Err(AppError::Internal(format_api_error("Anthropic", status, &err_text)))
                }
            }
            _ => Err(AppError::Internal(format!("Unsupported provider: {}", config.provider))),
        }
    }

    pub async fn generate_sql(
        config: &AiProviderConfig,
        system_prompt: &str,
        user_prompt: &str,
        dialect: &str,
    ) -> Result<AiSqlResponse, AppError> {
        let start_time = Instant::now();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::Internal(format!("HTTP client init failed: {}", e)))?;

        let raw_text = match config.provider.to_lowercase().as_str() {
            "ollama" => {
                let base_url = config.endpoint.as_deref().unwrap_or("http://localhost:11434").trim_end_matches('/');
                let url = format!("{}/api/generate", base_url);
                let body = json!({
                    "model": config.model,
                    "prompt": user_prompt,
                    "system": system_prompt,
                    "stream": false,
                    "options": {
                        "temperature": config.temperature.unwrap_or(0.2)
                    }
                });

                let resp = client.post(&url)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::Internal(format!("Ollama generate error: {}", e)))?;

                if !resp.status().is_success() {
                    let status = resp.status();
                    let err_msg = resp.text().await.unwrap_or_default();
                    return Err(AppError::Internal(format!("Ollama error (HTTP {}): {}", status, err_msg)));
                }

                let res_json: Value = resp.json().await
                    .map_err(|e| AppError::Internal(format!("Failed to parse Ollama response: {}", e)))?;

                res_json.get("response")
                    .and_then(|r| r.as_str())
                    .unwrap_or_default()
                    .to_string()
            }
            "openai" | "deepseek" | "custom" => {
                let api_key = config.api_key.as_deref().unwrap_or_default();
                let base_url = config.endpoint.as_deref().unwrap_or_else(|| {
                    if config.provider == "deepseek" {
                        "https://api.deepseek.com/v1"
                    } else {
                        "https://api.openai.com/v1"
                    }
                }).trim_end_matches('/');

                let url = format!("{}/chat/completions", base_url);
                let body = json!({
                    "model": config.model,
                    "messages": [
                        { "role": "system", "content": system_prompt },
                        { "role": "user", "content": user_prompt }
                    ],
                    "temperature": config.temperature.unwrap_or(0.2)
                });

                let mut req = client.post(&url).json(&body);
                if !api_key.is_empty() {
                    req = req.header("Authorization", format!("Bearer {}", api_key));
                }

                let resp = req.send().await
                    .map_err(|e| AppError::Internal(format!("{} request error: {}", config.provider, e)))?;

                if !resp.status().is_success() {
                    let status = resp.status();
                    let err_msg = resp.text().await.unwrap_or_default();
                    return Err(AppError::Internal(format_api_error(&config.provider, status, &err_msg)));
                }

                let res_json: Value = resp.json().await
                    .map_err(|e| AppError::Internal(format!("Failed to parse JSON response: {}", e)))?;

                res_json.get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|f| f.get("message"))
                    .and_then(|m| m.get("content"))
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string()
            }
            "gemini" => {
                let api_key = config.api_key.as_deref().unwrap_or_default();
                if api_key.is_empty() {
                    return Err(AppError::Internal("Gemini API Key is required".into()));
                }

                let base_model = config.model.trim_start_matches("models/");
                let candidate_urls = [
                    format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", base_model, api_key),
                    format!("https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}", base_model, api_key),
                    format!("https://generativelanguage.googleapis.com/v1beta/models/{}-latest:generateContent?key={}", base_model, api_key),
                    format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash-latest:generateContent?key={}", api_key),
                    format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", api_key),
                ];

                let body = json!({
                    "system_instruction": {
                        "parts": [{ "text": system_prompt }]
                    },
                    "contents": [{
                        "parts": [{ "text": user_prompt }]
                    }],
                    "generationConfig": {
                        "temperature": config.temperature.unwrap_or(0.2)
                    }
                });

                let mut last_err = String::new();
                let mut chosen_res_json: Option<Value> = None;

                for url in &candidate_urls {
                    let resp = client.post(url)
                        .json(&body)
                        .send()
                        .await
                        .map_err(|e| AppError::Internal(format!("Gemini request error: {}", e)))?;

                    if resp.status().is_success() {
                        let res_json: Value = resp.json().await
                            .map_err(|e| AppError::Internal(format!("Failed to parse Gemini response: {}", e)))?;
                        chosen_res_json = Some(res_json);
                        break;
                    } else {
                        let status = resp.status();
                        let err_msg = resp.text().await.unwrap_or_default();
                        last_err = format_api_error("Gemini", status, &err_msg);
                        if status == reqwest::StatusCode::NOT_FOUND {
                            continue;
                        } else {
                            return Err(AppError::Internal(last_err));
                        }
                    }
                }

                let res_json = match chosen_res_json {
                    Some(j) => j,
                    None => return Err(AppError::Internal(if !last_err.is_empty() { last_err } else { "Gemini model not found".into() })),
                };

                res_json.get("candidates")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|f| f.get("content"))
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array())
                    .and_then(|pa| pa.first())
                    .and_then(|t| t.get("text"))
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string()
            }
            "anthropic" => {
                let api_key = config.api_key.as_deref().unwrap_or_default();
                if api_key.is_empty() {
                    return Err(AppError::Internal("Anthropic API Key is required".into()));
                }

                let body = json!({
                    "model": config.model,
                    "system": system_prompt,
                    "max_tokens": 4096,
                    "messages": [{ "role": "user", "content": user_prompt }],
                    "temperature": config.temperature.unwrap_or(0.2)
                });

                let resp = client.post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::Internal(format!("Anthropic request error: {}", e)))?;

                if !resp.status().is_success() {
                    let status = resp.status();
                    let err_msg = resp.text().await.unwrap_or_default();
                    return Err(AppError::Internal(format_api_error("Anthropic", status, &err_msg)));
                }

                let res_json: Value = resp.json().await
                    .map_err(|e| AppError::Internal(format!("Failed to parse Anthropic response: {}", e)))?;

                res_json.get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|f| f.get("text"))
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string()
            }
            _ => return Err(AppError::Internal(format!("Unsupported provider: {}", config.provider))),
        };

        let (sql, explanation) = extract_sql_and_explanation(&raw_text);
        let tables_used = extract_referenced_tables(&sql);
        let execution_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(AiSqlResponse {
            sql,
            explanation,
            tables_used,
            dialect: dialect.to_string(),
            model_used: config.model.clone(),
            execution_time_ms,
        })
    }
}

fn format_api_error(provider: &str, status: reqwest::StatusCode, raw_body: &str) -> String {
    let clean_body = raw_body.trim();
    if let Ok(v) = serde_json::from_str::<Value>(clean_body) {
        if let Some(msg) = v.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
            let clean_msg = if let Some(idx) = msg.find(" (request_id:") {
                &msg[..idx]
            } else {
                msg
            };
            return format!("{} error (HTTP {}): {}", provider, status, clean_msg);
        }
        if let Some(msg) = v.get("error").and_then(|e| e.as_str()) {
            return format!("{} error (HTTP {}): {}", provider, status, msg);
        }
        if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
            return format!("{} error (HTTP {}): {}", provider, status, msg);
        }
    }
    format!("{} error (HTTP {}): {}", provider, status, clean_body)
}

fn extract_sql_and_explanation(raw: &str) -> (String, String) {
    let raw = raw.trim();

    // Check for ```sql ... ``` code fence
    if let Some(start_idx) = raw.find("```sql") {
        let after_start = &raw[start_idx + 6..];
        if let Some(end_idx) = after_start.find("```") {
            let sql = after_start[..end_idx].trim().to_string();
            let mut explanation = String::new();
            if start_idx > 0 {
                explanation.push_str(raw[..start_idx].trim());
            }
            let after_fence = after_start[end_idx + 3..].trim();
            if !after_fence.is_empty() {
                if !explanation.is_empty() {
                    explanation.push_str("\n\n");
                }
                explanation.push_str(after_fence);
            }
            return (sql, explanation);
        }
    }

    // Check for generic ``` ... ``` code fence
    if let Some(start_idx) = raw.find("```") {
        let after_start = &raw[start_idx + 3..];
        if let Some(end_idx) = after_start.find("```") {
            let sql = after_start[..end_idx].trim().to_string();
            let mut explanation = String::new();
            if start_idx > 0 {
                explanation.push_str(raw[..start_idx].trim());
            }
            let after_fence = after_start[end_idx + 3..].trim();
            if !after_fence.is_empty() {
                if !explanation.is_empty() {
                    explanation.push_str("\n\n");
                }
                explanation.push_str(after_fence);
            }
            return (sql, explanation);
        }
    }

    // Fallback if no fence: check if starts with common SQL keywords
    let upper = raw.to_uppercase();
    if upper.starts_with("SELECT") || upper.starts_with("WITH") || upper.starts_with("INSERT") || upper.starts_with("UPDATE") || upper.starts_with("DELETE") {
        return (raw.to_string(), "Generated SQL query ready for execution.".into());
    }

    (raw.to_string(), String::new())
}

fn extract_referenced_tables(sql: &str) -> Vec<String> {
    let mut tables = Vec::new();
    let tokens: Vec<&str> = sql.split_whitespace().collect();
    for i in 0..tokens.len() {
        let tok = tokens[i].to_uppercase();
        if (tok == "FROM" || tok == "JOIN" || tok == "INTO" || tok == "UPDATE") && i + 1 < tokens.len() {
            let next_tok = tokens[i + 1]
                .trim_matches(|c| c == '(' || c == ')' || c == ',' || c == ';' || c == '"' || c == '`' || c == '\'');
            if !next_tok.is_empty() && !next_tok.to_uppercase().starts_with("SELECT") && !tables.contains(&next_tok.to_string()) {
                tables.push(next_tok.to_string());
            }
        }
    }
    tables
}
