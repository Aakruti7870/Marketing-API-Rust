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
        Err(AppError::Forbidden("Webhook verification token mismatch".to_string()))
    }
}

async fn handle_whatsapp_webhook(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("📩 Received WhatsApp webhook notification: {:?}", payload);

    // Process WhatsApp status updates and inbound messages.
    if let Some(entries) = payload.get("entry").and_then(|v| v.as_array()) {
        for entry in entries {
            if let Some(changes) = entry.get("changes").and_then(|v| v.as_array()) {
                for change in changes {
                    let value = &change["value"];

                    if let Some(statuses) = value.get("statuses").and_then(|v| v.as_array()) {
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

                    let phone_number_id = value
                        .get("metadata")
                        .and_then(|v| v.get("phone_number_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();

                    if let Some(messages) = value.get("messages").and_then(|v| v.as_array()) {
                        for message in messages {
                            let sender = message.get("from").and_then(|v| v.as_str()).unwrap_or_default();
                            let text = message
                                .get("text")
                                .and_then(|v| v.get("body"))
                                .and_then(|v| v.as_str());

                            if !sender.is_empty() {
                                if let Some(body) = text {
                                    if let Err(err) = crate::services::agent_service::handle_meta_inbound(
                                        &state.pool,
                                        &state.config,
                                        "WHATSAPP",
                                        phone_number_id,
                                        sender,
                                        body,
                                    )
                                    .await
                                    {
                                        tracing::error!("AI WhatsApp inbound handling failed: {}", err);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "status": "received" })))
}


async fn verify_meta_webhook(
    State(state): State<AppState>,
    Query(query): Query<WebhookVerificationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let mode=query.hub_mode.as_deref();
    let token=query.hub_verify_token.as_deref();
    let challenge=query.hub_challenge.unwrap_or_default();
    if mode==Some("subscribe") && token==Some(&state.config.whatsapp_webhook_verify_token) {
        Ok((StatusCode::OK,challenge))
    } else {
        Err(AppError::Forbidden("Webhook verification token mismatch".into()))
    }
}

async fn handle_meta_webhook(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(entries)=payload.get("entry").and_then(|v|v.as_array()) {
        for entry in entries {
            let account_id=entry.get("id").and_then(|v|v.as_str()).unwrap_or_default();
            if let Some(changes)=entry.get("changes").and_then(|v|v.as_array()) {
                for change in changes {
                    let value=&change["value"];
                    if let Some(messages)=value.get("messages").and_then(|v|v.as_array()) {
                        for message in messages {
                            let sender=message.get("from").and_then(|v|v.as_str()).unwrap_or_default();
                            let text=message.get("text").and_then(|v|v.get("body")).and_then(|v|v.as_str());
                            let button_id=message.get("interactive")
                                .and_then(|v|v.get("button_reply"))
                                .and_then(|v|v.get("id"))
                                .and_then(|v|v.as_str());
                            if !sender.is_empty() {
                                let account=value.get("metadata").and_then(|v|v.get("phone_number_id")).and_then(|v|v.as_str()).unwrap_or(account_id);
                                if let Some(button_id)=button_id {
                                    match crate::services::playground_service::handle_button_action(&state.pool,&state.config,button_id,sender).await {
                                        Ok(true)=>{},
                                        Ok(false)=>{
                                            tracing::warn!("Unknown GOLD-e Smart WhatsApp button id: {}",button_id);
                                        },
                                        Err(err)=>tracing::error!("Smart WhatsApp button handling failed: {}",err),
                                    }
                                } else if let Some(body)=text {
                                    if let Err(err)=crate::services::agent_service::handle_meta_inbound(&state.pool,&state.config,"WHATSAPP",account,sender,body).await {
                                        tracing::error!("AI WhatsApp inbound handling failed: {}",err);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(messages)=entry.get("messaging").and_then(|v|v.as_array()) {
                for event in messages {
                    let sender=event.get("sender").and_then(|v|v.get("id")).and_then(|v|v.as_str()).unwrap_or_default();
                    let text=event.get("message").and_then(|v|v.get("text")).and_then(|v|v.as_str());
                    if sender.is_empty() { continue; }
                    if let Some(body)=text {
                        let channel=if payload.get("object").and_then(|v|v.as_str())==Some("instagram") { "INSTAGRAM" } else { "FACEBOOK" };
                        if let Err(err)=crate::services::agent_service::handle_meta_inbound(&state.pool,&state.config,channel,account_id,sender,body).await {
                            tracing::error!("AI {} inbound handling failed: {}",channel,err);
                        }
                    }
                }
            }
        }
    }
    (StatusCode::OK, Json(serde_json::json!({"status":"received"})))
}
