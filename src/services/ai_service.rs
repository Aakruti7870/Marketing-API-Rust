use crate::config::Config;
use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

pub const MAX_PROMPT_CHARS: usize = 64_000;
const HTTP_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    OpenAi,
    Anthropic,
    Ollama,
}

impl AiProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiProvider::OpenAi => "openai",
            AiProvider::Anthropic => "anthropic",
            AiProvider::Ollama => "ollama",
        }
    }

    pub fn from_str_or_default(
        provider: Option<&str>,
        model: Option<&str>,
        default_provider: &str,
    ) -> Self {
        if let Some(p) = provider {
            let p_lower = p.trim().to_lowercase();
            match p_lower.as_str() {
                "anthropic" | "claude" => return AiProvider::Anthropic,
                "ollama" | "local" => return AiProvider::Ollama,
                "openai" | "gemini" => return AiProvider::OpenAi,
                _ => {}
            }
        }

        if let Some(m) = model {
            let m_lower = m.trim().to_lowercase();
            if m_lower.contains("claude") {
                return AiProvider::Anthropic;
            }
            if m_lower.starts_with("llama")
                || m_lower.starts_with("mistral")
                || m_lower.starts_with("phi")
                || m_lower.starts_with("qwen")
                || m_lower.starts_with("gemma")
                || m_lower.starts_with("deepseek")
            {
                return AiProvider::Ollama;
            }
            if m_lower.starts_with("gpt")
                || m_lower.starts_with("o1")
                || m_lower.starts_with("o3")
                || m_lower.starts_with("gemini")
            {
                return AiProvider::OpenAi;
            }
        }

        match default_provider.trim().to_lowercase().as_str() {
            "anthropic" | "claude" => AiProvider::Anthropic,
            "ollama" | "local" => AiProvider::Ollama,
            _ => AiProvider::OpenAi,
        }
    }

    pub fn default_model(&self, config: &Config) -> String {
        match self {
            AiProvider::OpenAi => config.ai_model.clone(),
            AiProvider::Anthropic => "claude-3-5-sonnet-20241022".to_string(),
            AiProvider::Ollama => "llama3".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct AiCompletionOptions {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub messages: Vec<AiChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCompletionResult {
    pub text: String,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderInfo {
    pub id: String,
    pub name: String,
    pub configured: bool,
    pub default_model: String,
    pub models: Vec<String>,
}

pub fn get_provider_catalog(config: &Config) -> Vec<AiProviderInfo> {
    vec![
        AiProviderInfo {
            id: "openai".to_string(),
            name: "OpenAI / Compatible".to_string(),
            configured: config.ai_api_key.is_some(),
            default_model: config.ai_model.clone(),
            models: vec![
                config.ai_model.clone(),
                "gpt-5.6-luna".to_string(),
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
            ],
        },
        AiProviderInfo {
            id: "anthropic".to_string(),
            name: "Anthropic Claude".to_string(),
            configured: config.anthropic_api_key.is_some(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
            models: vec![
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-haiku-20240307".to_string(),
                "claude-3-opus-20240229".to_string(),
            ],
        },
        AiProviderInfo {
            id: "ollama".to_string(),
            name: "Ollama (Self-hosted / Local)".to_string(),
            configured: config.ollama_base_url.is_some(),
            default_model: "llama3".to_string(),
            models: vec![
                "llama3".to_string(),
                "llama3.2".to_string(),
                "mistral".to_string(),
                "phi3".to_string(),
                "qwen2.5".to_string(),
            ],
        },
    ]
}

pub async fn generate_completion(
    config: &Config,
    options: &AiCompletionOptions,
) -> Result<AiCompletionResult, AppError> {
    // 1. Safeguard: Input length verification
    let mut total_chars = options.system_prompt.as_ref().map(|s| s.len()).unwrap_or(0);
    for m in &options.messages {
        total_chars += m.content.len();
    }
    if total_chars > MAX_PROMPT_CHARS {
        return Err(AppError::Validation(format!(
            "Prompt total length ({} chars) exceeds maximum allowed safeguard ({} chars)",
            total_chars, MAX_PROMPT_CHARS
        )));
    }

    // 2. Resolve provider and model
    let provider = AiProvider::from_str_or_default(
        options.provider.as_deref(),
        options.model.as_deref(),
        &config.ai_provider,
    );
    let model = options
        .model
        .clone()
        .unwrap_or_else(|| provider.default_model(config));

    let client = Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| {
            AppError::ExternalService(format!("Failed to initialize HTTP client: {}", e))
        })?;

    match provider {
        AiProvider::OpenAi => {
            call_openai(
                &client,
                config,
                &model,
                options.system_prompt.as_deref(),
                &options.messages,
                options.temperature.unwrap_or(config.ai_temperature),
            )
            .await
        }
        AiProvider::Anthropic => {
            call_anthropic(
                &client,
                config,
                &model,
                options.system_prompt.as_deref(),
                &options.messages,
                options.temperature.unwrap_or(0.35),
                options.max_tokens.unwrap_or(2048),
            )
            .await
        }
        AiProvider::Ollama => {
            call_ollama(
                &client,
                config,
                &model,
                options.system_prompt.as_deref(),
                &options.messages,
                options.temperature.unwrap_or(0.35),
            )
            .await
        }
    }
}

pub fn build_anthropic_payload(
    model: &str,
    system_prompt: Option<&str>,
    messages: &[AiChatMessage],
    temperature: f32,
    max_tokens: u32,
) -> Value {
    let mut anthropic_messages = Vec::new();
    for msg in messages {
        let role = match msg.role.as_str() {
            "assistant" => "assistant",
            _ => "user",
        };
        anthropic_messages.push(json!({
            "role": role,
            "content": msg.content
        }));
    }

    if anthropic_messages.is_empty() {
        anthropic_messages.push(json!({
            "role": "user",
            "content": "Hello"
        }));
    }

    let mut body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "temperature": temperature,
        "messages": anthropic_messages
    });

    if let Some(sys) = system_prompt {
        if !sys.trim().is_empty() {
            body["system"] = json!(sys);
        }
    }

    body
}

pub fn parse_anthropic_response(
    status: reqwest::StatusCode,
    raw: &str,
    model: &str,
) -> Result<AiCompletionResult, AppError> {
    let payload: Value = serde_json::from_str(raw).map_err(|e| {
        AppError::ExternalService(format!(
            "Invalid Anthropic JSON response (HTTP {}): {}",
            status, e
        ))
    })?;

    if !status.is_success() {
        let msg = payload
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(Value::as_str)
            .or_else(|| payload.get("message").and_then(Value::as_str))
            .unwrap_or("Anthropic API returned an error");
        return Err(AppError::ExternalService(format!(
            "Anthropic error (HTTP {}): {}",
            status, msg
        )));
    }

    let content_blocks = payload
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AppError::ExternalService("Anthropic response missing 'content' array".into())
        })?;

    let mut texts = Vec::new();
    for block in content_blocks {
        if block.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(t) = block.get("text").and_then(Value::as_str) {
                texts.push(t);
            }
        }
    }

    if texts.is_empty() {
        return Err(AppError::ExternalService(
            "Anthropic response returned no text blocks".into(),
        ));
    }

    let text = texts.join("\n");
    let prompt_tokens = payload
        .get("usage")
        .and_then(|u| u.get("input_tokens"))
        .and_then(Value::as_u64)
        .map(|v| v as u32);
    let completion_tokens = payload
        .get("usage")
        .and_then(|u| u.get("output_tokens"))
        .and_then(Value::as_u64)
        .map(|v| v as u32);

    Ok(AiCompletionResult {
        text,
        provider: "anthropic".to_string(),
        model: model.to_string(),
        prompt_tokens,
        completion_tokens,
    })
}

async fn call_anthropic(
    client: &Client,
    config: &Config,
    model: &str,
    system_prompt: Option<&str>,
    messages: &[AiChatMessage],
    temperature: f32,
    max_tokens: u32,
) -> Result<AiCompletionResult, AppError> {
    let api_key = config.anthropic_api_key.as_deref().ok_or_else(|| {
        AppError::BadRequest(
            "Anthropic provider is not configured. Set ANTHROPIC_API_KEY on the backend.".into(),
        )
    })?;

    let base_url = config.anthropic_base_url.trim_end_matches('/');
    let body = build_anthropic_payload(model, system_prompt, messages, temperature, max_tokens);

    let response = client
        .post(format!("{}/messages", base_url))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("Anthropic request failed: {}", e)))?;

    let status = response.status();
    let raw = response.text().await.map_err(|e| {
        AppError::ExternalService(format!("Failed reading Anthropic response: {}", e))
    })?;

    parse_anthropic_response(status, &raw, model)
}

pub fn build_ollama_payload(
    model: &str,
    system_prompt: Option<&str>,
    messages: &[AiChatMessage],
    temperature: f32,
) -> Value {
    let mut ollama_messages = Vec::new();
    if let Some(sys) = system_prompt {
        if !sys.trim().is_empty() {
            ollama_messages.push(json!({
                "role": "system",
                "content": sys
            }));
        }
    }

    for msg in messages {
        ollama_messages.push(json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    json!({
        "model": model,
        "messages": ollama_messages,
        "stream": false,
        "options": {
            "temperature": temperature
        }
    })
}

pub fn parse_ollama_response(
    status: reqwest::StatusCode,
    raw: &str,
    model: &str,
) -> Result<AiCompletionResult, AppError> {
    let payload: Value = serde_json::from_str(raw).map_err(|e| {
        AppError::ExternalService(format!(
            "Invalid Ollama JSON response (HTTP {}): {}",
            status, e
        ))
    })?;

    if !status.is_success() {
        let msg = payload
            .get("error")
            .and_then(Value::as_str)
            .or_else(|| payload.get("message").and_then(Value::as_str))
            .unwrap_or("Ollama API returned an error");
        return Err(AppError::ExternalService(format!(
            "Ollama error (HTTP {}): {}",
            status, msg
        )));
    }

    if let Some(err) = payload.get("error").and_then(Value::as_str) {
        return Err(AppError::ExternalService(format!("Ollama error: {}", err)));
    }

    let text = payload
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            AppError::ExternalService("Ollama response missing message content".into())
        })?;

    let prompt_tokens = payload
        .get("prompt_eval_count")
        .and_then(Value::as_u64)
        .map(|v| v as u32);
    let completion_tokens = payload
        .get("eval_count")
        .and_then(Value::as_u64)
        .map(|v| v as u32);

    Ok(AiCompletionResult {
        text,
        provider: "ollama".to_string(),
        model: model.to_string(),
        prompt_tokens,
        completion_tokens,
    })
}

async fn call_ollama(
    client: &Client,
    config: &Config,
    model: &str,
    system_prompt: Option<&str>,
    messages: &[AiChatMessage],
    temperature: f32,
) -> Result<AiCompletionResult, AppError> {
    let base_url = config
        .ollama_base_url
        .as_deref()
        .ok_or_else(|| {
            AppError::BadRequest(
                "Ollama provider is not configured. Set OLLAMA_BASE_URL on the backend.".into(),
            )
        })?
        .trim_end_matches('/');
    let body = build_ollama_payload(model, system_prompt, messages, temperature);

    let response = client
        .post(format!("{}/api/chat", base_url))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("Ollama request failed: {}", e)))?;

    let status = response.status();
    let raw = response
        .text()
        .await
        .map_err(|e| AppError::ExternalService(format!("Failed reading Ollama response: {}", e)))?;

    parse_ollama_response(status, &raw, model)
}

pub fn build_openai_payload(
    model: &str,
    system_prompt: Option<&str>,
    messages: &[AiChatMessage],
    temperature: f32,
) -> Value {
    let mut openai_messages = Vec::new();
    if let Some(sys) = system_prompt {
        if !sys.trim().is_empty() {
            openai_messages.push(json!({
                "role": "system",
                "content": sys
            }));
        }
    }

    for msg in messages {
        openai_messages.push(json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    json!({
        "model": model,
        "messages": openai_messages,
        "temperature": temperature
    })
}

pub fn parse_openai_response(
    status: reqwest::StatusCode,
    raw: &str,
    model: &str,
) -> Result<AiCompletionResult, AppError> {
    let payload: Value = serde_json::from_str(raw).map_err(|e| {
        AppError::ExternalService(format!(
            "Invalid OpenAI JSON response (HTTP {}): {}",
            status, e
        ))
    })?;

    if !status.is_success() {
        let msg = payload
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(Value::as_str)
            .or_else(|| payload.get("message").and_then(Value::as_str))
            .unwrap_or("OpenAI API returned an error");
        return Err(AppError::ExternalService(format!(
            "OpenAI error (HTTP {}): {}",
            status, msg
        )));
    }

    let text = payload
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AppError::ExternalService("OpenAI returned no text output".into()))?;

    let prompt_tokens = payload
        .get("usage")
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(Value::as_u64)
        .map(|v| v as u32);
    let completion_tokens = payload
        .get("usage")
        .and_then(|u| u.get("completion_tokens"))
        .and_then(Value::as_u64)
        .map(|v| v as u32);

    Ok(AiCompletionResult {
        text,
        provider: "openai".to_string(),
        model: model.to_string(),
        prompt_tokens,
        completion_tokens,
    })
}

async fn call_openai(
    client: &Client,
    config: &Config,
    model: &str,
    system_prompt: Option<&str>,
    messages: &[AiChatMessage],
    temperature: f32,
) -> Result<AiCompletionResult, AppError> {
    let api_key = config.ai_api_key.as_deref().ok_or_else(|| {
        AppError::BadRequest("AI provider is not configured. Set AI_API_KEY on the backend.".into())
    })?;

    let base_url = config.ai_api_base_url.trim_end_matches('/');
    let body = build_openai_payload(model, system_prompt, messages, temperature);

    let response = client
        .post(format!("{}/chat/completions", base_url))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("OpenAI request failed: {}", e)))?;

    let status = response.status();
    let raw = response
        .text()
        .await
        .map_err(|e| AppError::ExternalService(format!("Failed reading OpenAI response: {}", e)))?;

    parse_openai_response(status, &raw, model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_inference_from_model_name() {
        assert_eq!(
            AiProvider::from_str_or_default(None, Some("claude-3-5-sonnet-20241022"), "openai"),
            AiProvider::Anthropic
        );
        assert_eq!(
            AiProvider::from_str_or_default(None, Some("claude-3-haiku"), "openai"),
            AiProvider::Anthropic
        );
        assert_eq!(
            AiProvider::from_str_or_default(None, Some("llama3"), "openai"),
            AiProvider::Ollama
        );
        assert_eq!(
            AiProvider::from_str_or_default(None, Some("mistral-7b"), "openai"),
            AiProvider::Ollama
        );
        assert_eq!(
            AiProvider::from_str_or_default(None, Some("gpt-4o"), "anthropic"),
            AiProvider::OpenAi
        );
    }

    #[test]
    fn test_explicit_provider_takes_precedence() {
        assert_eq!(
            AiProvider::from_str_or_default(Some("anthropic"), Some("custom-model"), "openai"),
            AiProvider::Anthropic
        );
        assert_eq!(
            AiProvider::from_str_or_default(Some("ollama"), Some("claude-alias"), "openai"),
            AiProvider::Ollama
        );
    }

    #[test]
    fn test_anthropic_payload_structure() {
        let messages = vec![
            AiChatMessage {
                role: "user".into(),
                content: "Hello Claude".into(),
            },
            AiChatMessage {
                role: "assistant".into(),
                content: "Hi there!".into(),
            },
        ];

        let payload = build_anthropic_payload(
            "claude-3-5-sonnet-20241022",
            Some("You are a helpful marketing assistant"),
            &messages,
            0.5,
            1024,
        );

        assert_eq!(payload["model"], "claude-3-5-sonnet-20241022");
        assert_eq!(payload["max_tokens"], 1024);
        assert_eq!(payload["system"], "You are a helpful marketing assistant");
        assert_eq!(payload["messages"].as_array().unwrap().len(), 2);
        assert_eq!(payload["messages"][0]["role"], "user");
        assert_eq!(payload["messages"][1]["role"], "assistant");
    }

    #[test]
    fn test_anthropic_response_parsing_success() {
        let json = r#"{
            "id": "msg_01X9DPz3",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "Here is the festive marketing campaign copy."
                }
            ],
            "model": "claude-3-5-sonnet-20241022",
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 42,
                "output_tokens": 128
            }
        }"#;

        let result =
            parse_anthropic_response(reqwest::StatusCode::OK, json, "claude-3-5-sonnet-20241022")
                .unwrap();

        assert_eq!(result.provider, "anthropic");
        assert_eq!(result.text, "Here is the festive marketing campaign copy.");
        assert_eq!(result.prompt_tokens, Some(42));
        assert_eq!(result.completion_tokens, Some(128));
    }

    #[test]
    fn test_anthropic_error_propagation() {
        let error_json = r#"{
            "type": "error",
            "error": {
                "type": "authentication_error",
                "message": "invalid x-api-key"
            }
        }"#;

        let err = parse_anthropic_response(
            reqwest::StatusCode::UNAUTHORIZED,
            error_json,
            "claude-3-5-sonnet-20241022",
        )
        .unwrap_err();

        match err {
            AppError::ExternalService(msg) => {
                assert!(msg.contains("invalid x-api-key"));
                assert!(msg.contains("HTTP 401"));
            }
            _ => panic!("Expected ExternalService error"),
        }
    }

    #[test]
    fn test_ollama_payload_and_response() {
        let messages = vec![AiChatMessage {
            role: "user".into(),
            content: "Generate product tagline".into(),
        }];

        let payload = build_ollama_payload("llama3", Some("Be creative"), &messages, 0.4);
        assert_eq!(payload["model"], "llama3");
        assert_eq!(payload["messages"].as_array().unwrap().len(), 2);
        assert_eq!(payload["messages"][0]["role"], "system");

        let ollama_response = r#"{
            "model": "llama3",
            "created_at": "2026-10-16T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "Engineered for unstoppable speed."
            },
            "done": true,
            "prompt_eval_count": 25,
            "eval_count": 8
        }"#;

        let result =
            parse_ollama_response(reqwest::StatusCode::OK, ollama_response, "llama3").unwrap();

        assert_eq!(result.provider, "ollama");
        assert_eq!(result.text, "Engineered for unstoppable speed.");
        assert_eq!(result.prompt_tokens, Some(25));
        assert_eq!(result.completion_tokens, Some(8));
    }

    #[test]
    fn test_openai_response_parsing() {
        let openai_json = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Boost your conversions by 40% with automated flows."
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 15,
                "completion_tokens": 30,
                "total_tokens": 45
            }
        }"#;

        let result =
            parse_openai_response(reqwest::StatusCode::OK, openai_json, "gpt-5.6-luna").unwrap();

        assert_eq!(result.provider, "openai");
        assert_eq!(
            result.text,
            "Boost your conversions by 40% with automated flows."
        );
        assert_eq!(result.prompt_tokens, Some(15));
        assert_eq!(result.completion_tokens, Some(30));
    }
}
