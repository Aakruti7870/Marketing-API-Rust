pub mod auth;
pub mod config;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod state;
pub mod utils;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderName, HeaderValue, Method,
    },
    Router,
};
use state::AppState;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn app(state: AppState) -> Router {
    let configured_origins = state
        .config
        .cors_origin
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .filter_map(|origin| HeaderValue::try_from(origin).ok())
        .collect::<Vec<_>>();

    // Wildcard is retained only for local development. Config::from_env rejects
    // wildcard origins in production before the router is created.
    let allowed_origin = if state.config.cors_origin.trim() == "*" {
        AllowOrigin::from(Any)
    } else {
        AllowOrigin::list(configured_origins)
    };

    let cors = CorsLayer::new()
        .allow_origin(allowed_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("x-workspace-id"),
        ]);

    routes::create_api_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
