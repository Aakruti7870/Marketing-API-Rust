use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::agent::AgentRunWithSteps;

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_contacts: i64,
    pub active_campaigns: i64,
    pub messages_sent: i64,
    pub messages_delivered: i64,
    pub messages_read: i64,
    pub delivery_rate_percent: f64,
    pub read_rate_percent: f64,
    pub agent_runs_total: i64,
    pub pending_approvals_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelMetric {
    pub channel: String,
    pub sent: i64,
    pub delivered: i64,
    pub read: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: Uuid,
    pub title: String,
    pub entity_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub summary: DashboardSummary,
    pub channel_breakdown: Vec<ChannelMetric>,
    pub recent_agent_runs: Vec<AgentRunWithSteps>,
    pub recent_activities: Vec<ActivityItem>,
}
