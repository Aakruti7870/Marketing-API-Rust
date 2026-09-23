use crate::error::AppError;
use crate::middleware::TenantContext;
use crate::services::analytics_service;
use crate::state::AppState;
use crate::utils::response::json_success;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        // Frontend contract: GET /api/v1/analytics/dashboard
        .route("/dashboard", get(get_dashboard))
        .with_state(state)
}

async fn get_dashboard(
    State(state): State<AppState>,
    tenant: TenantContext,
) -> Result<impl IntoResponse, AppError> {
    let dashboard = analytics_service::get_dashboard(&state.pool, tenant.workspace_id).await?;
    Ok(json_success(dashboard, "Command Center dashboard data retrieved"))
}
