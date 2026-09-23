use golde_marketing_api::models::{DashboardSummary, ChannelMetric};

#[test]
fn test_dashboard_metrics_calculation() {
    let sent = 100;
    let delivered = 95;
    let read = 65;

    let delivery_rate = ((delivered as f64 / sent as f64) * 100.0).round();
    let read_rate = ((read as f64 / delivered as f64) * 100.0).round();

    let summary = DashboardSummary {
        total_contacts: 250,
        active_campaigns: 3,
        messages_sent: sent,
        messages_delivered: delivered,
        messages_read: read,
        delivery_rate_percent: delivery_rate,
        read_rate_percent: read_rate,
        agent_runs_total: 12,
        pending_approvals_count: 1,
    };

    assert_eq!(summary.delivery_rate_percent, 95.0);
    assert_eq!(summary.read_rate_percent, 68.0);
    assert_eq!(summary.pending_approvals_count, 1);
}
