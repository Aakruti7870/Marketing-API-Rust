use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::models::SendMessageDto;
use crate::services::message_service;
use crate::state::AppState;
use crate::utils::pagination::PaginationQuery;
use crate::utils::response::{json_paginated, json_success};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_messages).post(send_message))
        .route("/:id", get(get_message))
        .with_state(state)
}

#[derive(Deserialize)]
struct MessageQuery {
    #[serde(flatten)]
    pagination: PaginationQuery,
    status: Option<String>,
}

async fn list_messages(
    State(state): State<AppState>,
    tenant: TenantContext,
    Query(query): Query<MessageQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (messages, meta) = message_service::list_messages(
        &state.pool,
        tenant.workspace_id,
        query.pagination,
        query.status,
    )
    .await?;

    Ok(json_paginated(messages, meta, "Messages retrieved"))
}

async fn send_message(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<SendMessageDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    let message = message_service::send_message(
        &state.pool,
        &state.http_client,
        &state.config,
        tenant.workspace_id,
        dto,
    )
    .await?;

    Ok(json_success(message, "Message dispatched"))
}

async fn get_message(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let msg = message_service::get_message(&state.pool, tenant.workspace_id, message_id).await?;
    Ok(json_success(msg, "Message retrieved"))
}
