pub mod config;
pub mod error;
pub mod state;
pub mod utils;
pub mod models;
pub mod auth;
pub mod middleware;
pub mod services;
pub mod routes;

use axum::Router;
use state::AppState;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    routes::create_api_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
