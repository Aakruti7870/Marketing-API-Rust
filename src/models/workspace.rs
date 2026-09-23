use chrono::{DateTime, Utc};
use std::fmt;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkspaceRole {
    OWNER,
    ADMIN,
    MEMBER,
    VIEWER,
}

impl fmt::Display for WorkspaceRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceRole::OWNER => write!(f, "OWNER"),
            WorkspaceRole::ADMIN => write!(f, "ADMIN"),
            WorkspaceRole::MEMBER => write!(f, "MEMBER"),
            WorkspaceRole::VIEWER => write!(f, "VIEWER"),
        }
    }
}

impl From<&str> for WorkspaceRole {
    fn from(s: &str) -> Self {
        match s {
            "OWNER" => WorkspaceRole::OWNER,
            "ADMIN" => WorkspaceRole::ADMIN,
            "VIEWER" => WorkspaceRole::VIEWER,
            _ => WorkspaceRole::MEMBER,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkspaceMember {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMemberDetail {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}
