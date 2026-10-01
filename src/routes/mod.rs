pub mod agents;
pub mod analytics;
pub mod auth;
pub mod automation_webhooks;
pub mod automations;
pub mod campaigns;
pub mod contacts;
pub mod custom_domains;
pub mod messages;
pub mod playground;
pub mod public_agents;
pub mod webhooks;
pub mod workspaces;

use crate::state::AppState;
use axum::{extract::Extension, http::StatusCode, routing::get, Json, Router};
use serde_json::json;

pub fn create_api_router(state: AppState) -> Router {
    Router::new()
        .layer(axum::Extension(state.clone()))
        .route("/", get(custom_domains::landing))
        .route("/health", get(health_check))
        .nest("/api/v1/auth", auth::routes(state.clone()))
        .nest("/api/v1/workspaces", workspaces::routes(state.clone()))
        .nest(
            "/api/v1/custom-domains",
            custom_domains::routes(state.clone()),
        )
        .nest("/api/v1/contacts", contacts::routes(state.clone()))
        .nest("/api/v1/campaigns", campaigns::routes(state.clone()))
        .nest("/api/v1/messages", messages::routes(state.clone()))
        .nest("/api/v1/agents", agents::routes(state.clone()))
        .nest("/api/public/agents", public_agents::routes(state.clone()))
        .nest("/api/v1/analytics", analytics::routes(state.clone()))
        .nest("/api/v1/webhooks", webhooks::routes(state.clone()))
        .nest("/api/v1/automations", automations::routes(state.clone()))
        .nest("/api/v1/playground", playground::routes(state.clone()))
        .nest(
            "/api/public/playground",
            playground::public_routes(state.clone()),
        )
        .nest(
            "/api/v1/automation-webhooks",
            axum::Router::new()
                .route("/:id", axum::routing::post(automation_webhooks::handle))
                .with_state(state.clone()),
        )
}

async fn health_check(
    Extension(state): Extension<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    let redis_status = match state.redis.as_ref() {
        Some(connection) => {
            let mut connection = connection.clone();
            match redis::cmd("PING").query_async::<String>(&mut connection).await {
                Ok(response) if response == "PONG" => "healthy",
                _ => "unhealthy",
            }
        }
        None => "disabled",
    };
    let healthy = redis_status != "unhealthy";
    let status_code = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status_code,
        Json(json!({
            "status": if healthy { "healthy" } else { "degraded" },
            "service": "GOLD-e GrowthOS Marketing API",
            "runtime": "Rust / Axum / SQLx",
            "version": "1.0.0",
            "dependencies": { "redis": redis_status },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}
