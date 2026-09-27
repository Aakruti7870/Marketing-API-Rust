use crate::error::AppError;
use crate::models::PublicAgentChatDto;
use crate::services::agent_service;
use crate::state::AppState;
use axum::{extract::{Path, State}, response::IntoResponse, routing::post, Json, Router};
use crate::utils::response::json_success;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/:public_key/chat", post(chat))
        .with_state(state)
}

async fn chat(
    State(state): State<AppState>,
    Path(public_key): Path<String>,
    Json(dto): Json<PublicAgentChatDto>,
) -> Result<impl IntoResponse, AppError> {
    if dto.message.trim().is_empty() {
        return Err(AppError::Validation("message is required".into()));
    }
    if dto.message.len() > 4000 {
        return Err(AppError::Validation("message is too long".into()));
    }
    Ok(json_success(
        agent_service::public_chat(&state.pool, &state.config, &public_key, dto).await?,
        "AI agent response generated",
    ))
}
