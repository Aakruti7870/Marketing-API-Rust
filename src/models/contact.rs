use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contact {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: String,
    pub company: Option<String>,
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub custom_fields: serde_json::Value,
    pub last_contacted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactDto {
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: String,
    pub company: Option<String>,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub custom_fields: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactDto {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub custom_fields: Option<serde_json::Value>,
}
