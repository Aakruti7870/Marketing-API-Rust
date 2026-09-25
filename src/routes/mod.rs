pub mod auth;
pub mod workspaces;
pub mod contacts;
pub mod campaigns;
pub mod messages;
pub mod agents;
pub mod analytics;
pub mod webhooks;
pub mod automations;
pub mod automation_webhooks;

use crate::state::AppState;
use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn create_api_router(state: AppState) -> Router {
    let automation_state = state.clone();
    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1/auth", auth::routes(state.clone()))
        .nest("/api/v1/workspaces", workspaces::routes(state.clone()))
        .nest("/api/v1/contacts", contacts::routes(state.clone()))
        .nest("/api/v1/campaigns", campaigns::routes(state.clone()))
        .nest("/api/v1/messages", messages::routes(state.clone()))
        .nest("/api/v1/agents", agents::routes(state.clone()))
        .nest("/api/v1/analytics", analytics::routes(state.clone()))
        .nest("/api/v1/webhooks", webhooks::routes(state.clone()))
        .nest("/api/v1/automations", automations::routes(state.clone()))
        .route("/api/v1/automation-webhooks/:id", axum::routing::post(automation_webhooks::handle))
        .with_state(automation_state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "GOLD-e GrowthOS Marketing API",
        "runtime": "Rust / Axum / SQLx",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
