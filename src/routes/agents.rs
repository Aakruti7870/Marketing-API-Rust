use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::models::{
    AiAgentChatDto, CreateAgentRunDto, CreateAiAgentDto, CreateChannelConnectionDto, PublicAgentChatDto, StepApprovalDto, StepRejectionDto,
    UpdateAiAgentDto,
};
use crate::services::agent_service;
use crate::state::AppState;
use crate::utils::pagination::PaginationQuery;
use crate::utils::response::{json_paginated, json_success};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_agents).post(create_agent))
        .route("/templates", get(list_templates))
        .route("/:id", get(get_agent).put(update_agent).delete(delete_agent))
        .route("/:id/chat", post(chat))
        .route("/:id/channels", get(list_channels).post(create_channel))
        .route("/channels/:channel_id", delete(delete_channel))
        .route("/runs", get(list_runs).post(create_run))
        .route("/runs/:id", get(get_run))
        .route("/runs/:id/steps/:stepId/approve", post(approve_step))
        .route("/runs/:id/steps/:stepId/reject", post(reject_step))
        .with_state(state)
}

#[derive(Deserialize)]
struct AgentRunQuery {
    #[serde(flatten)]
    pagination: PaginationQuery,
    status: Option<String>,
}

async fn list_agents(
    State(state): State<AppState>,
    tenant: TenantContext,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_success(
        agent_service::list_agents(&state.pool, tenant.workspace_id).await?,
        "AI agents retrieved",
    ))
}

async fn list_templates() -> Result<impl IntoResponse, AppError> {
    Ok(json_success(agent_service::templates(), "AI agent templates retrieved"))
}

async fn get_agent(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_success(
        agent_service::get_agent(&state.pool, tenant.workspace_id, id).await?,
        "AI agent retrieved",
    ))
}

async fn create_agent(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<CreateAiAgentDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    Ok(json_success(
        agent_service::create_agent(&state.pool, tenant.workspace_id, tenant.user_id, dto).await?,
        "AI agent created",
    ))
}

async fn update_agent(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateAiAgentDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    Ok(json_success(
        agent_service::update_agent(&state.pool, tenant.workspace_id, id, dto).await?,
        "AI agent updated",
    ))
}

async fn delete_agent(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    agent_service::delete_agent(&state.pool, tenant.workspace_id, id).await?;
    Ok(json_success(serde_json::json!({"id": id, "deleted": true}), "AI agent deleted"))
}

async fn chat(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<Uuid>,
    Json(dto): Json<AiAgentChatDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    Ok(json_success(
        agent_service::chat(&state.pool, &state.config, tenant.workspace_id, id, dto).await?,
        "AI agent response generated",
    ))
}

async fn list_channels(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_success(
        agent_service::list_channel_connections(&state.pool, tenant.workspace_id, id).await?,
        "Agent channels retrieved",
    ))
}

async fn create_channel(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<Uuid>,
    Json(dto): Json<CreateChannelConnectionDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    Ok(json_success(
        agent_service::create_channel_connection(&state.pool, &state.config, tenant.workspace_id, id, dto).await?,
        "Agent channel deployed",
    ))
}

async fn delete_channel(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    agent_service::delete_channel_connection(&state.pool, tenant.workspace_id, channel_id).await?;
    Ok(json_success(serde_json::json!({"id": channel_id, "deleted": true}), "Agent channel removed"))
}

async fn list_runs(
    State(state): State<AppState>,
    tenant: TenantContext,
    Query(query): Query<AgentRunQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (runs, meta) = agent_service::list_runs(
        &state.pool,
        tenant.workspace_id,
        query.pagination,
        query.status,
    )
    .await?;
    Ok(json_paginated(runs, meta, "Agent runs retrieved"))
}

async fn get_run(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(run_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_success(
        agent_service::get_run(&state.pool, tenant.workspace_id, run_id).await?,
        "Agent run retrieved",
    ))
}

async fn create_run(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<CreateAgentRunDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    Ok(json_success(
        agent_service::create_run(&state.pool, tenant.workspace_id, tenant.user_id, dto).await?,
        "Agent run triggered",
    ))
}

async fn approve_step(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path((run_id, step_id)): Path<(Uuid, Uuid)>,
    body: Option<Json<StepApprovalDto>>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    let dto = body.map(|b| b.0).unwrap_or(StepApprovalDto { comment: None });
    Ok(json_success(
        agent_service::approve_step(
            &state.pool,
            tenant.workspace_id,
            tenant.user_id,
            run_id,
            step_id,
            dto,
        )
        .await?,
        "Step approved and execution resumed",
    ))
}

async fn reject_step(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path((run_id, step_id)): Path<(Uuid, Uuid)>,
    Json(dto): Json<StepRejectionDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    Ok(json_success(
        agent_service::reject_step(
            &state.pool,
            tenant.workspace_id,
            tenant.user_id,
            run_id,
            step_id,
            dto,
        )
        .await?,
        "Step rejected and run cancelled",
    ))
}
