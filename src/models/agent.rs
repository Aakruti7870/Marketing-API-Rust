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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAgent {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub template_key: String,
    pub industry: String,
    pub role: String,
    pub description: Option<String>,
    pub status: String,
    pub system_prompt: String,
    pub capabilities: serde_json::Value,
    pub tools: serde_json::Value,
    pub channels: serde_json::Value,
    pub settings: serde_json::Value,
    pub created_by_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub public_key: Option<String>,
    pub welcome_message: Option<String>,
    pub handoff_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAgentConversation {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub agent_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub channel: String,
    pub external_user_id: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAgentMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub tool_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAiAgentDto {
    pub name: String,
    pub template_key: String,
    pub industry: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub tools: Option<serde_json::Value>,
    pub channels: Option<serde_json::Value>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAiAgentDto {
    pub name: Option<String>,
    pub industry: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub system_prompt: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub tools: Option<serde_json::Value>,
    pub channels: Option<serde_json::Value>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiAgentChatDto {
    pub conversation_id: Option<Uuid>,
    pub external_user_id: Option<String>,
    pub channel: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AiAgentChatResponse {
    pub conversation_id: Uuid,
    pub agent_id: Uuid,
    pub reply: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAgentChannelConnection {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub agent_id: Uuid,
    pub channel: String,
    pub provider: String,
    pub status: String,
    pub external_account_id: Option<String>,
    pub external_sender_id: Option<String>,
    pub display_name: Option<String>,
    pub config: serde_json::Value,
    pub secret_ciphertext: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChannelConnectionDto {
    pub channel: String,
    pub provider: Option<String>,
    pub external_account_id: Option<String>,
    pub external_sender_id: Option<String>,
    pub display_name: Option<String>,
    pub secret: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ChannelConnectionResponse {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub channel: String,
    pub provider: String,
    pub status: String,
    pub external_account_id: Option<String>,
    pub external_sender_id: Option<String>,
    pub display_name: Option<String>,
    pub config: serde_json::Value,
    pub secret_configured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicAgentChatDto {
    pub conversation_id: Option<Uuid>,
    pub visitor_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PublicAgentChatResponse {
    pub conversation_id: Uuid,
    pub reply: String,
    pub agent_name: String,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentConversationSummary {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub channel: String,
    pub external_user_id: Option<String>,
    pub status: String,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub last_message: Option<String>,
    pub last_message_role: Option<String>,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct OwnerReplyDto {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaygroundButtonActionDto {
    pub button_id: String,
    pub asset_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub title: String,
    pub action_type: String,
    #[serde(default)]
    pub action_payload: serde_json::Value,
}
