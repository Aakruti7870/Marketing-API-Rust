use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use tracing::info;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/whatsapp", get(verify_whatsapp_webhook).post(handle_whatsapp_webhook))
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
        Err(AppError::Forbidden("Webhook verification token mismatch".to_string()))
    }
}

async fn handle_whatsapp_webhook(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("📩 Received WhatsApp webhook notification: {:?}", payload);

    // Update message delivery statuses if status array exists
    if let Some(entry) = payload["entry"].as_array().and_then(|a| a.first()) {
        if let Some(changes) = entry["changes"].as_array().and_then(|a| a.first()) {
            if let Some(statuses) = changes["value"]["statuses"].as_array() {
                for s in statuses {
                    if let (Some(id), Some(status)) = (s["id"].as_str(), s["status"].as_str()) {
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
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "status": "received" })))
}
