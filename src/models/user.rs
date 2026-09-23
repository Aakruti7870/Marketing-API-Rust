use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GlobalRole {
    SYSTEM_ADMIN,
    USER,
}

impl ToString for GlobalRole {
    fn to_string(&self) -> String {
        match self {
            GlobalRole::SYSTEM_ADMIN => "SYSTEM_ADMIN".to_string(),
            GlobalRole::USER => "USER".to_string(),
        }
    }
}

impl From<&str> for GlobalRole {
    fn from(s: &str) -> Self {
        match s {
            "SYSTEM_ADMIN" => GlobalRole::SYSTEM_ADMIN,
            _ => GlobalRole::USER,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            phone: u.phone,
            role: u.role,
            is_active: u.is_active,
            created_at: u.created_at,
        }
    }
}
