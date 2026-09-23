use crate::auth::AuthenticatedUser;
use crate::error::AppError;
use crate::services::auth_service::{self, LoginDto, RefreshTokenDto, RegisterDto};
use crate::state::AppState;
use crate::utils::response::json_success;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(get_me))
        .with_state(state)
}

async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> Result<impl IntoResponse, AppError> {
    let res = auth_service::register(&state.pool, &state.config, dto).await?;
    Ok(json_success(res, "User registered successfully"))
}

async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<impl IntoResponse, AppError> {
    let res = auth_service::login(&state.pool, &state.config, dto).await?;
    Ok(json_success(res, "Login successful"))
}

async fn refresh(
    State(state): State<AppState>,
    Json(dto): Json<RefreshTokenDto>,
) -> Result<impl IntoResponse, AppError> {
    let tokens = auth_service::refresh(&state.pool, &state.config, dto).await?;
    Ok(json_success(tokens, "Tokens refreshed"))
}

#[derive(Deserialize)]
struct LogoutBody {
    refresh_token: Option<String>,
}

async fn logout(
    State(state): State<AppState>,
    body: Option<Json<LogoutBody>>,
) -> Result<impl IntoResponse, AppError> {
    let token = body.and_then(|b| b.refresh_token.clone());
    auth_service::logout(&state.pool, token).await?;
    Ok(json_success(serde_json::json!({ "logged_out": true }), "Logged out"))
}

async fn get_me(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_service::get_me(&state.pool, claims.sub).await?;
    Ok(json_success(user, "User profile fetched"))
}
