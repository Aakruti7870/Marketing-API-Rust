use crate::error::AppError;
use crate::models::{
    ActivityItem, ChannelMetric, DashboardData, DashboardSummary,
};
use crate::services::agent_service::list_runs;
use crate::utils::pagination::PaginationQuery;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_dashboard(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<DashboardData, AppError> {
    // 1. Total contacts
    let contacts_count = sqlx::query!(
        "SELECT COUNT(*) as total FROM contacts WHERE workspace_id = $1",
        workspace_id
    )
    .fetch_one(pool)
    .await?
    .total
    .unwrap_or(0);

    // 2. Active campaigns
    let campaigns_count = sqlx::query!(
        "SELECT COUNT(*) as total FROM campaigns WHERE workspace_id = $1 AND status = 'RUNNING'",
        workspace_id
    )
    .fetch_one(pool)
    .await?
    .total
    .unwrap_or(0);

    // 3. Messages metrics
    let msg_stats = sqlx::query!(
        "SELECT
            COUNT(*) FILTER (WHERE status IN ('SENT', 'DELIVERED', 'READ')) as sent,
            COUNT(*) FILTER (WHERE status IN ('DELIVERED', 'READ')) as delivered,
            COUNT(*) FILTER (WHERE status = 'READ') as read,
            COUNT(*) FILTER (WHERE status = 'FAILED') as failed
         FROM messages
         WHERE workspace_id = $1",
        workspace_id
    )
    .fetch_one(pool)
    .await?;

    let sent = msg_stats.sent.unwrap_or(0);
    let delivered = msg_stats.delivered.unwrap_or(0);
    let read = msg_stats.read.unwrap_or(0);
    let failed = msg_stats.failed.unwrap_or(0);

    let delivery_rate = if sent > 0 {
        ((delivered as f64 / sent as f64) * 100.0).round()
    } else {
        100.0
    };

    let read_rate = if delivered > 0 {
        ((read as f64 / delivered as f64) * 100.0).round()
    } else {
        0.0
    };

    // 4. Agent runs metrics
    let agent_stats = sqlx::query!(
        "SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'WAITING_APPROVAL') as pending_approvals
         FROM agent_runs
         WHERE workspace_id = $1",
        workspace_id
    )
    .fetch_one(pool)
    .await?;

    let agent_runs_total = agent_stats.total.unwrap_or(0);
    let pending_approvals_count = agent_stats.pending_approvals.unwrap_or(0);

    let summary = DashboardSummary {
        total_contacts: contacts_count,
        active_campaigns: campaigns_count,
        messages_sent: sent,
        messages_delivered: delivered,
        messages_read: read,
        delivery_rate_percent: delivery_rate,
        read_rate_percent: read_rate,
        agent_runs_total,
        pending_approvals_count,
    };

    let channel_breakdown = vec![
        ChannelMetric {
            channel: "WHATSAPP".to_string(),
            sent,
            delivered,
            read,
            failed,
        },
        ChannelMetric {
            channel: "EMAIL".to_string(),
            sent: 0,
            delivered: 0,
            read: 0,
            failed: 0,
        },
        ChannelMetric {
            channel: "SMS".to_string(),
            sent: 0,
            delivered: 0,
            read: 0,
            failed: 0,
        },
    ];

    // 5. Recent agent runs with steps
    let (recent_runs, _) = list_runs(
        pool,
        workspace_id,
        PaginationQuery { page: Some(1), limit: Some(5) },
        None,
    )
    .await?;

    // 6. Recent activities
    let activity_rows = sqlx::query!(
        "SELECT id, action, entity_type, created_at FROM audit_logs
         WHERE workspace_id = $1
         ORDER BY created_at DESC
         LIMIT 10",
        workspace_id
    )
    .fetch_all(pool)
    .await?;

    let recent_activities = activity_rows
        .into_iter()
        .map(|r| ActivityItem {
            id: r.id,
            title: format!("{}: {}", r.action, r.entity_type),
            entity_type: r.entity_type,
            timestamp: r.created_at,
        })
        .collect();

    Ok(DashboardData {
        summary,
        channel_breakdown,
        recent_agent_runs: recent_runs,
        recent_activities,
    })
}
