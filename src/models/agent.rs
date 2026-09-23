use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentRun {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub agent_type: String,
    pub name: String,
    pub status: String,
    pub triggered_by_id: Uuid,
    pub input_params: serde_json::Value,
    pub output_data: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentStep {
    pub id: Uuid,
    pub run_id: Uuid,
    pub step_number: i32,
    pub name: String,
    pub description: Option<String>,
    pub action_type: String,
    pub status: String,
    pub input_payload: serde_json::Value,
    pub output_payload: serde_json::Value,
    pub requires_approval: bool,
    pub approved_by_id: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRunWithSteps {
    #[serde(flatten)]
    pub run: AgentRun,
    pub steps: Vec<AgentStep>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRunDto {
    pub agent_type: String,
    pub name: String,
    pub input_params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StepApprovalDto {
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StepRejectionDto {
    pub reason: String,
}
