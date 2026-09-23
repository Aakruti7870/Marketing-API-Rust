use crate::config::Config;
use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppSendResult {
    pub message_id: String,
    pub status: String,
    pub simulated: bool,
}

pub async fn send_whatsapp_message(
    http_client: &Client,
    config: &Config,
    to_phone: &str,
    content: &str,
    template_name: Option<&str>,
) -> Result<WhatsAppSendResult, AppError> {
    // 1. Check if Simulation Mode is enabled or Meta credentials are absent
    let has_credentials = config.whatsapp_phone_number_id.is_some() && config.whatsapp_access_token.is_some();
    if config.whatsapp_simulation_mode || !has_credentials {
        let simulated_wamid = format!(
            "wamid.HBgL{}FQIAERgS{}",
            to_phone.replace('+', ""),
            &Uuid::new_v4().to_string().replace('-', "")[..16]
        );

        info!(
            "🟢 [WHATSAPP SIMULATION] Message dispatched to {}: '{}' (Simulated ID: {})",
            to_phone, content, simulated_wamid
        );

        return Ok(WhatsAppSendResult {
            message_id: simulated_wamid,
            status: "SENT".to_string(),
            simulated: true,
        });
    }

    // 2. Real WhatsApp Cloud API Call via Meta Graph API
    let phone_number_id = config.whatsapp_phone_number_id.as_ref().unwrap();
    let access_token = config.whatsapp_access_token.as_ref().unwrap();
    let url = format!(
        "https://graph.facebook.com/{}/{}/messages",
        config.whatsapp_api_version, phone_number_id
    );

    let payload = if let Some(template) = template_name {
        json!({
            "messaging_product": "whatsapp",
            "to": to_phone,
            "type": "template",
            "template": {
                "name": template,
                "language": { "code": "en_US" }
            }
        })
    } else {
        json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to_phone,
            "type": "text",
            "text": { "preview_url": false, "body": content }
        })
    };

    let response = http_client
        .post(&url)
        .bearer_auth(access_token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("Meta Graph API request failed: {}", e)))?;

    if !response.status().is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(AppError::ExternalService(format!("Meta WhatsApp error: {}", err_text)));
    }

    let resp_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::ExternalService(format!("Failed to parse Meta response: {}", e)))?;

    let wamid = resp_json["messages"][0]["id"]
        .as_str()
        .unwrap_or("wamid.unknown")
        .to_string();

    Ok(WhatsAppSendResult {
        message_id: wamid,
        status: "SENT".to_string(),
        simulated: false,
    })
}
