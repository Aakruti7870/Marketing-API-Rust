use crate::error::AppError;
use crate::state::AppState;
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use tracing::info;

type HmacSha256 = Hmac<Sha256>;

fn verify_signature(secret: &str, body: &[u8], signature: Option<&str>) -> bool {
    let Some(signature) = signature.and_then(|s| s.strip_prefix("sha256=")) else {
        return false;
    };
    let Ok(signature) = hex::decode(signature) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    mac.verify_slice(&signature).is_ok()
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/whatsapp",
            get(verify_whatsapp_webhook).post(handle_whatsapp_webhook),
        )
        .route("/meta", get(verify_meta_webhook).post(handle_meta_webhook))
        .with_state(state)
}

#[derive(Deserialize)]
struct WebhookVerificationQuery {
    #[serde(rename = "hub.mode")]
    hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    hub_challenge: Option<String>,
}

async fn verify_whatsapp_webhook(
    State(state): State<AppState>,
    Query(query): Query<WebhookVerificationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let mode = query.hub_mode.as_deref();
    let token = query.hub_verify_token.as_deref();
    let challenge = query.hub_challenge.unwrap_or_default();

    if mode == Some("subscribe") && token == Some(&state.config.whatsapp_webhook_verify_token) {
        info!(" WhatsApp Cloud API webhook verified successfully");
        Ok((StatusCode::OK, challenge))
    } else {
        Err(AppError::Forbidden(
            "Webhook verification token mismatch".to_string(),
        ))
    }
}

async fn handle_whatsapp_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let Some(secret) = state
        .config
        .whatsapp_app_secret
        .as_deref()
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error":"Webhook signature verification is not configured"})),
        );
    };
    if !verify_signature(
        secret,
        &body,
        headers
            .get("x-hub-signature-256")
            .and_then(|v| v.to_str().ok()),
    ) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"Invalid webhook signature"})),
        );
    }
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error":"Invalid JSON"})),
            )
        }
    };
    info!("Received verified WhatsApp webhook notification");

    // Process WhatsApp status updates and inbound messages.
    if let Some(entries) = payload.get("entry").and_then(|v| v.as_array()) {
        for entry in entries {
            if let Some(changes) = entry.get("changes").and_then(|v| v.as_array()) {
                for change in changes {
                    let value = &change["value"];

                    if let Some(statuses) = value.get("statuses").and_then(|v| v.as_array()) {
                        for s in statuses {
                            if let (Some(id), Some(status)) =
                                (s["id"].as_str(), s["status"].as_str())
                            {
                                let normalized_status = status.to_uppercase();
                                let _ = sqlx::query!(
                                    "UPDATE messages SET status = $1, updated_at = NOW() WHERE external_message_id = $2",
                                    normalized_status,
                                    id
                                )
                                .execute(&state.pool)
                                .await;
                            }
                        }
                    }

                    let phone_number_id = value
                        .get("metadata")
                        .and_then(|v| v.get("phone_number_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();

                    if let Some(messages) = value.get("messages").and_then(|v| v.as_array()) {
                        for message in messages {
                            let sender = message
                                .get("from")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();
                            let text = message
                                .get("text")
                                .and_then(|v| v.get("body"))
                                .and_then(|v| v.as_str());

                            if !sender.is_empty() {
                                if let Some(body) = text {
                                    if let Err(err) =
                                        crate::services::agent_service::handle_meta_inbound(
                                            &state.pool,
                                            &state.config,
                                            "WHATSAPP",
                                            phone_number_id,
                                            sender,
                                            body,
                                        )
                                        .await
                                    {
                                        tracing::error!(
                                            "AI WhatsApp inbound handling failed: {}",
                                            err
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "received" })),
    )
}

async fn verify_meta_webhook(
    State(state): State<AppState>,
    Query(query): Query<WebhookVerificationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let mode = query.hub_mode.as_deref();
    let token = query.hub_verify_token.as_deref();
    let challenge = query.hub_challenge.unwrap_or_default();
    if mode == Some("subscribe") && token == Some(&state.config.whatsapp_webhook_verify_token) {
        Ok((StatusCode::OK, challenge))
    } else {
        Err(AppError::Forbidden(
            "Webhook verification token mismatch".into(),
        ))
    }
}

async fn handle_meta_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let Some(secret) = state
        .config
        .whatsapp_app_secret
        .as_deref()
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error":"Webhook signature verification is not configured"})),
        );
    };
    if !verify_signature(
        secret,
        &body,
        headers
            .get("x-hub-signature-256")
            .and_then(|v| v.to_str().ok()),
    ) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"Invalid webhook signature"})),
        );
    }
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error":"Invalid JSON"})),
            )
        }
    };
    if let Some(entries) = payload.get("entry").and_then(|v| v.as_array()) {
        for entry in entries {
            let account_id = entry.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if let Some(changes) = entry.get("changes").and_then(|v| v.as_array()) {
                for change in changes {
                    let value = &change["value"];
                    if let Some(messages) = value.get("messages").and_then(|v| v.as_array()) {
                        for message in messages {
                            let sender = message
                                .get("from")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();
                            let text = message
                                .get("text")
                                .and_then(|v| v.get("body"))
                                .and_then(|v| v.as_str());
                            let button_id = message
                                .get("interactive")
                                .and_then(|v| v.get("button_reply"))
                                .and_then(|v| v.get("id"))
                                .and_then(|v| v.as_str());
                            if !sender.is_empty() {
                                let account = value
                                    .get("metadata")
                                    .and_then(|v| v.get("phone_number_id"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(account_id);
                                if let Some(button_id) = button_id {
                                    match crate::services::playground_service::handle_button_action(
                                        &state.pool,
                                        &state.config,
                                        button_id,
                                        sender,
                                    )
                                    .await
                                    {
                                        Ok(true) => {}
                                        Ok(false) => {
                                            tracing::warn!(
                                                "Unknown GOLD-e Smart WhatsApp button id: {}",
                                                button_id
                                            );
                                        }
                                        Err(err) => tracing::error!(
                                            "Smart WhatsApp button handling failed: {}",
                                            err
                                        ),
                                    }
                                } else if let Some(body) = text {
                                    if let Err(err) =
                                        crate::services::agent_service::handle_meta_inbound(
                                            &state.pool,
                                            &state.config,
                                            "WHATSAPP",
                                            account,
                                            sender,
                                            body,
                                        )
                                        .await
                                    {
                                        tracing::error!(
                                            "AI WhatsApp inbound handling failed: {}",
                                            err
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(messages) = entry.get("messaging").and_then(|v| v.as_array()) {
                for event in messages {
                    let sender = event
                        .get("sender")
                        .and_then(|v| v.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let text = event
                        .get("message")
                        .and_then(|v| v.get("text"))
                        .and_then(|v| v.as_str());
                    if sender.is_empty() {
                        continue;
                    }
                    if let Some(body) = text {
                        let channel = if payload.get("object").and_then(|v| v.as_str())
                            == Some("instagram")
                        {
                            "INSTAGRAM"
                        } else {
                            "FACEBOOK"
                        };
                        if let Err(err) = crate::services::agent_service::handle_meta_inbound(
                            &state.pool,
                            &state.config,
                            channel,
                            account_id,
                            sender,
                            body,
                        )
                        .await
                        {
                            tracing::error!("AI {} inbound handling failed: {}", channel, err);
                        }
                    }
                }
            }
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"status":"received"})),
    )
}

#[cfg(test)]
mod signature_tests {
    use super::{verify_signature, HmacSha256};
    use hmac::Mac;

    #[test]
    fn accepts_valid_signature() {
        let secret = "test_app_secret";
        let body = br#"{"entry":[]}"#;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        assert!(verify_signature(secret, body, Some(&signature)));
    }

    #[test]
    fn rejects_missing_malformed_and_invalid_signatures() {
        let body = br#"{"entry":[]}"#;
        assert!(!verify_signature("secret", body, None));
        assert!(!verify_signature("secret", body, Some("invalid")));
        assert!(!verify_signature("secret", body, Some("sha256=xyz")));
        assert!(!verify_signature("secret", body, Some("sha256=deadbeef")));
    }

    #[test]
    fn rejects_signature_for_different_body_or_secret() {
        let secret = "test_app_secret";
        let body = br#"{"entry":[]}"#;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        assert!(!verify_signature(
            secret,
            br#"{"entry":[1]}"#,
            Some(&signature)
        ));
        assert!(!verify_signature("wrong_secret", body, Some(&signature)));
    }
}
