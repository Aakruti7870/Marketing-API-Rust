use golde_marketing_api::{app, config::Config, state::AppState};
use axum::Router;
use sqlx::PgPool;

pub async fn setup_test_app(pool: PgPool) -> Router {
    let mut config = Config::from_env().unwrap();
    config.whatsapp_simulation_mode = true;

    let state = AppState::new(pool, config);
    app(state)
}
