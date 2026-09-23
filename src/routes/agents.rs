use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::models::{CreateAgentRunDto, StepApprovalDto, StepRejectionDto};
use crate::services::agent_service;
use crate::state::AppState;
use crate::utils::pagination::PaginationQuery;
use crate::utils::response::{json_paginated, json_success};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

pub fn routes(state: AppState) -> Router {
    Router::new()
        // Frontend contract: GET /api/v1/agents/runs
        .route("/runs", get(list_runs).post(create_run))
        .route("/runs/:id", get(get_run))
        // Frontend contract: POST /api/v1/agents/runs/:id/steps/:stepId/approve
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
    let run = agent_service::get_run(&state.pool, tenant.workspace_id, run_id).await?;
    Ok(json_success(run, "Agent run retrieved"))
}

async fn create_run(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<CreateAgentRunDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    let run = agent_service::create_run(&state.pool, tenant.workspace_id, tenant.user_id, dto).await?;
    Ok(json_success(run, "Agent run triggered"))
}

async fn approve_step(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path((run_id, step_id)): Path<(Uuid, Uuid)>,
    body: Option<Json<StepApprovalDto>>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    let dto = body.map(|b| b.0).unwrap_or_else(|| StepApprovalDto { comment: None });
    let run = agent_service::approve_step(
        &state.pool,
        tenant.workspace_id,
        tenant.user_id,
        run_id,
        step_id,
        dto,
    )
    .await?;

    Ok(json_success(run, "Step approved and execution resumed"))
}

async fn reject_step(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path((run_id, step_id)): Path<(Uuid, Uuid)>,
    Json(dto): Json<StepRejectionDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    let run = agent_service::reject_step(
        &state.pool,
        tenant.workspace_id,
        tenant.user_id,
        run_id,
        step_id,
        dto,
    )
    .await?;

    Ok(json_success(run, "Step rejected and run cancelled"))
}
