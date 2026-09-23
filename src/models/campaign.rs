use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub channel: String,
    pub status: String,
    pub schedule_time: Option<DateTime<Utc>>,
    pub target_tags: Vec<String>,
    pub template_id: Option<String>,
    pub template_params: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCampaignDto {
    pub name: String,
    pub description: Option<String>,
    pub channel: Option<String>,
    pub target_tags: Option<Vec<String>>,
    pub template_id: Option<String>,
    pub template_params: Option<serde_json::Value>,
    pub schedule_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub channel: Option<String>,
    pub status: Option<String>,
    pub target_tags: Option<Vec<String>>,
    pub template_id: Option<String>,
    pub template_params: Option<serde_json::Value>,
}
