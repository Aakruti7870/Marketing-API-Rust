use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutomationDefinition {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub triggers: Vec<AutomationTriggerInput>,
    pub nodes: Vec<AutomationNodeInput>,
    pub edges: Vec<AutomationEdgeInput>,
    pub schedule: Option<AutomationScheduleInput>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutomationTriggerInput {
    pub trigger_type: String,
    #[serde(default)]
    pub config: Value,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutomationNodeInput {
    pub node_key: String,
    pub node_type: String,
    pub name: String,
    #[serde(default)]
    pub config: Value,
    #[serde(default)]
    pub position: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutomationEdgeInput {
    pub source_node_key: String,
    pub target_node_key: String,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutomationScheduleInput {
    pub interval_seconds: i64,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutomationSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutomationRunResponse {
    pub run_id: Uuid,
    pub status: String,
}

fn default_true() -> bool { true }
