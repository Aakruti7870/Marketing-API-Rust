use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Automation {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub trigger_type: String,
    pub trigger_config: serde_json::Value,
    pub webhook_token: Option<String>,
    pub graph: serde_json::Value,
    pub version: i32,
    pub created_by_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schedule_interval_seconds: Option<i64>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AutomationRun {
    pub id: Uuid,
    pub automation_id: Uuid,
    pub workspace_id: Uuid,
    pub status: String,
    pub trigger_payload: serde_json::Value,
    pub output_data: serde_json::Value,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AutomationRunStep {
    pub id: Uuid,
    pub run_id: Uuid,
    pub node_id: String,
    pub node_type: String,
    pub status: String,
    pub input_data: serde_json::Value,
    pub output_data: serde_json::Value,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationGraph {
    pub nodes: Vec<AutomationNode>,
    pub edges: Vec<AutomationEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub position: [f64; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationEdge {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAutomationDto {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: Option<serde_json::Value>,
    pub graph: AutomationGraph,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAutomationDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_config: Option<serde_json::Value>,
    pub graph: Option<AutomationGraph>,
}

#[derive(Debug, Deserialize)]
pub struct RunAutomationDto {
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AutomationWithRuns {
    pub automation: Automation,
    pub runs: Vec<AutomationRun>,
}