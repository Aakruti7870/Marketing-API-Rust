use golde_marketing_api::config::Config;
use golde_marketing_api::error::AppError;
use golde_marketing_api::services::ai_service::{
    build_anthropic_payload, build_ollama_payload, build_openai_payload, generate_completion,
    get_provider_catalog, parse_anthropic_response, parse_ollama_response, parse_openai_response,
    AiChatMessage, AiCompletionOptions, AiProvider, MAX_PROMPT_CHARS,
};
use reqwest::StatusCode;

#[test]
fn test_claude_messages_request_structure() {
    let messages = vec![
        AiChatMessage {
            role: "user".to_string(),
            content: "Draft a product launch announcement".to_string(),
        },
        AiChatMessage {
            role: "assistant".to_string(),
            content: "Here is draft 1".to_string(),
        },
        AiChatMessage {
            role: "user".to_string(),
            content: "Make it more punchy".to_string(),
        },
    ];

    let payload = build_anthropic_payload(
        "claude-sonnet-5-5",
        Some("You are an expert copywriter"),
        &messages,
        0.7,
        1500,
    );

    assert_eq!(payload["model"], "claude-sonnet-5-5");
    assert_eq!(payload["max_tokens"], 1500);
    assert_eq!(payload["system"], "You are an expert copywriter");
    let msgs = payload["messages"].as_array().expect("array of messages");
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[0]["role"], "user");
    assert_eq!(msgs[0]["content"], "Draft a product launch announcement");
    assert_eq!(msgs[1]["role"], "assistant");
    assert_eq!(msgs[2]["role"], "user");
}

#[test]
fn test_claude_messages_response_parsing_success() {
    let raw_response = r#"{
        "id": "msg_0123456789",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "Exclusive festive offer: Get 20% off all marketing automation tools!"
            }
        ],
        "model": "claude-sonnet-5-5",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 58,
            "output_tokens": 112
        }
    }"#;

    let res = parse_anthropic_response(StatusCode::OK, raw_response, "claude-sonnet-5-5")
        .expect("parsing should succeed");

    assert_eq!(res.provider, "anthropic");
    assert_eq!(res.model, "claude-sonnet-5-5");
    assert_eq!(
        res.text,
        "Exclusive festive offer: Get 20% off all marketing automation tools!"
    );
    assert_eq!(res.prompt_tokens, Some(58));
    assert_eq!(res.completion_tokens, Some(112));
}

#[test]
fn test_claude_api_error_response() {
    let error_response = r#"{
        "type": "error",
        "error": {
            "type": "authentication_error",
            "message": "x-api-key header is invalid or expired"
        }
    }"#;

    let err = parse_anthropic_response(
        StatusCode::UNAUTHORIZED,
        error_response,
        "claude-sonnet-5-5",
    )
    .expect_err("should return AppError");

    match err {
        AppError::ExternalService(msg) => {
            assert!(msg.contains("x-api-key header is invalid"));
            assert!(msg.contains("HTTP 401"));
        }
        _ => panic!("Expected ExternalService error variant"),
    }
}

#[test]
fn test_ollama_chat_request_structure() {
    let messages = vec![AiChatMessage {
        role: "user".to_string(),
        content: "Write a high-converting email subject line".to_string(),
    }];

    let payload = build_ollama_payload(
        "llama3.2",
        Some("You are an email marketing AI"),
        &messages,
        0.5,
    );

    assert_eq!(payload["model"], "llama3.2");
    assert_eq!(payload["stream"], false);
    assert_eq!(payload["options"]["temperature"], 0.5);

    let msgs = payload["messages"].as_array().expect("messages array");
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0]["role"], "system");
    assert_eq!(msgs[0]["content"], "You are an email marketing AI");
    assert_eq!(msgs[1]["role"], "user");
}

#[test]
fn test_ollama_chat_response_parsing_success() {
    let raw = r#"{
        "model": "llama3.2",
        "created_at": "2026-10-16T00:00:00Z",
        "message": {
            "role": "assistant",
            "content": "⚡ Last chance: Claim your exclusive marketing credit before midnight"
        },
        "done": true,
        "prompt_eval_count": 32,
        "eval_count": 14
    }"#;

    let res = parse_ollama_response(StatusCode::OK, raw, "llama3.2")
        .expect("Ollama parse should succeed");

    assert_eq!(res.provider, "ollama");
    assert_eq!(res.model, "llama3.2");
    assert_eq!(
        res.text,
        "⚡ Last chance: Claim your exclusive marketing credit before midnight"
    );
    assert_eq!(res.prompt_tokens, Some(32));
    assert_eq!(res.completion_tokens, Some(14));
}

#[test]
fn test_ollama_error_propagation() {
    let raw_err = r#"{
        "error": "model 'non-existent-model' not found, try pulling it first"
    }"#;

    let err = parse_ollama_response(StatusCode::NOT_FOUND, raw_err, "non-existent-model")
        .expect_err("should return error");

    match err {
        AppError::ExternalService(msg) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected ExternalService error variant"),
    }
}

#[test]
fn test_openai_payload_structure() {
    let messages = vec![AiChatMessage {
        role: "user".to_string(),
        content: "Generate product marketing headline".to_string(),
    }];

    let payload = build_openai_payload(
        "gpt-5.6-luna",
        Some("You are an expert copywriter"),
        &messages,
        0.5,
    );

    assert_eq!(payload["model"], "gpt-5.6-luna");
    assert_eq!(payload["temperature"], 0.5);
    let msgs = payload["messages"].as_array().expect("messages array");
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0]["role"], "system");
    assert_eq!(msgs[0]["content"], "You are an expert copywriter");
    assert_eq!(msgs[1]["role"], "user");
    assert_eq!(msgs[1]["content"], "Generate product marketing headline");
}

#[test]
fn test_openai_compatible_response_parsing() {
    let raw = r#"{
        "id": "chatcmpl-test-999",
        "object": "chat.completion",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Welcome to Kryso Marketing Engine"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 6,
            "total_tokens": 16
        }
    }"#;

    let res = parse_openai_response(StatusCode::OK, raw, "gpt-5.6-luna")
        .expect("OpenAI parse should succeed");

    assert_eq!(res.provider, "openai");
    assert_eq!(res.text, "Welcome to Kryso Marketing Engine");
    assert_eq!(res.prompt_tokens, Some(10));
    assert_eq!(res.completion_tokens, Some(6));
}

#[test]
fn test_provider_auto_inference() {
    assert_eq!(
        AiProvider::from_str_or_default(None, Some("claude-sonnet-5-5"), "openai"),
        AiProvider::Anthropic
    );
    assert_eq!(
        AiProvider::from_str_or_default(None, Some("claude-haiku-4-5-20251001"), "openai"),
        AiProvider::Anthropic
    );
    assert_eq!(
        AiProvider::from_str_or_default(None, Some("claude-opus-4-8"), "openai"),
        AiProvider::Anthropic
    );
    assert_eq!(
        AiProvider::from_str_or_default(None, Some("llama3"), "openai"),
        AiProvider::Ollama
    );
    assert_eq!(
        AiProvider::from_str_or_default(None, Some("phi3:mini"), "openai"),
        AiProvider::Ollama
    );
    assert_eq!(
        AiProvider::from_str_or_default(None, Some("qwen2.5:7b"), "openai"),
        AiProvider::Ollama
    );
    assert_eq!(
        AiProvider::from_str_or_default(None, Some("gpt-4o"), "anthropic"),
        AiProvider::OpenAi
    );
    assert_eq!(
        AiProvider::from_str_or_default(Some("anthropic"), Some("custom-model"), "openai"),
        AiProvider::Anthropic
    );
    assert_eq!(
        AiProvider::from_str_or_default(Some("ollama"), Some("any-model"), "openai"),
        AiProvider::Ollama
    );
}

#[test]
fn test_provider_catalog_returns_all_providers() {
    let config = Config::from_env().unwrap();
    let catalog = get_provider_catalog(&config);

    assert_eq!(catalog.len(), 3);
    let ids: Vec<&str> = catalog.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"openai"));
    assert!(ids.contains(&"anthropic"));
    assert!(ids.contains(&"ollama"));

    let anthropic_info = catalog.iter().find(|p| p.id == "anthropic").unwrap();
    assert_eq!(anthropic_info.default_model, "claude-sonnet-5-5");
    assert!(anthropic_info
        .models
        .contains(&"claude-sonnet-5-5".to_string()));
    assert!(anthropic_info
        .models
        .contains(&"claude-haiku-4-5-20251001".to_string()));
    assert!(anthropic_info
        .models
        .contains(&"claude-opus-4-8".to_string()));

    let ollama_info = catalog.iter().find(|p| p.id == "ollama").unwrap();
    assert_eq!(ollama_info.configured, config.ollama_base_url.is_some());
    assert!(ollama_info.models.contains(&"llama3".to_string()));
}

#[test]
fn test_provider_catalog_gemini_models() {
    let mut config = Config::from_env().unwrap();
    config.ai_api_base_url = "https://generativelanguage.googleapis.com/v1beta/openai".to_string();
    config.ai_model = "gemini-3.5-flash".to_string();
    let catalog = get_provider_catalog(&config);

    let openai_info = catalog.iter().find(|p| p.id == "openai").unwrap();
    assert_eq!(openai_info.default_model, "gemini-3.5-flash");
    assert!(openai_info.models.contains(&"gemini-3.5-flash".to_string()));
    assert!(openai_info
        .models
        .contains(&"gemini-3.1-pro-preview".to_string()));
}

#[tokio::test]
async fn test_prompt_length_safeguard_rejects_oversized_input() {
    let config = Config::from_env().unwrap();
    let massive_string = "A".repeat(MAX_PROMPT_CHARS + 50);

    let options = AiCompletionOptions {
        provider: Some("openai".to_string()),
        model: Some("gpt-5.6-luna".to_string()),
        system_prompt: None,
        messages: vec![AiChatMessage {
            role: "user".to_string(),
            content: massive_string,
        }],
        temperature: Some(0.2),
        max_tokens: Some(100),
    };

    let err = generate_completion(&config, &options)
        .await
        .expect_err("Massive prompt should fail validation safeguard");

    match err {
        AppError::Validation(msg) => {
            assert!(msg.contains("exceeds maximum allowed safeguard"));
        }
        _ => panic!("Expected Validation error variant"),
    }
}

#[tokio::test]
async fn test_missing_anthropic_api_key_fails_closed_without_simulation() {
    let mut config = Config::from_env().unwrap();
    config.anthropic_api_key = None;

    let options = AiCompletionOptions {
        provider: Some("anthropic".to_string()),
        model: Some("claude-sonnet-5-5".to_string()),
        system_prompt: None,
        messages: vec![AiChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        temperature: Some(0.2),
        max_tokens: Some(100),
    };

    let err = generate_completion(&config, &options)
        .await
        .expect_err("Missing Anthropic key must fail immediately");

    match err {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("Anthropic provider is not configured"));
            assert!(msg.contains("ANTHROPIC_API_KEY"));
        }
        _ => panic!("Expected BadRequest error variant"),
    }
}

#[tokio::test]
async fn test_missing_ollama_base_url_fails_closed() {
    let mut config = Config::from_env().unwrap();
    config.ollama_base_url = None;

    let options = AiCompletionOptions {
        provider: Some("ollama".to_string()),
        model: Some("llama3".to_string()),
        system_prompt: None,
        messages: vec![AiChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        temperature: Some(0.2),
        max_tokens: Some(100),
    };

    let err = generate_completion(&config, &options)
        .await
        .expect_err("Missing Ollama endpoint must fail immediately");

    match err {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("Ollama provider is not configured"));
            assert!(msg.contains("OLLAMA_BASE_URL"));
        }
        _ => panic!("Expected BadRequest error variant"),
    }
}
