use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub contact_id: Uuid,
    pub channel: String,
    pub direction: String,
    pub external_message_id: Option<String>,
    pub status: String,
    pub content: String,
    pub template_name: Option<String>,
    pub template_data: serde_json::Value,
    pub error: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageDto {
    pub contact_id: Uuid,
    pub content: String,
    pub template_name: Option<String>,
    pub template_data: Option<serde_json::Value>,
}
