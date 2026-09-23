use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::models::{CreateCampaignDto, UpdateCampaignDto};
use crate::services::campaign_service;
use crate::state::AppState;
use crate::utils::pagination::PaginationQuery;
use crate::utils::response::{json_paginated, json_success};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_campaigns).post(create_campaign))
        .route("/:id", get(get_campaign).put(update_campaign))
        .route("/:id/launch", post(launch_campaign))
        .route("/:id/pause", post(pause_campaign))
        .with_state(state)
}

async fn list_campaigns(
    State(state): State<AppState>,
    tenant: TenantContext,
    Query(query): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (campaigns, meta) = campaign_service::list_campaigns(&state.pool, tenant.workspace_id, query).await?;
    Ok(json_paginated(campaigns, meta, "Campaigns retrieved"))
}

async fn create_campaign(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<CreateCampaignDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    let campaign = campaign_service::create_campaign(&state.pool, tenant.workspace_id, tenant.user_id, dto).await?;
    Ok(json_success(campaign, "Campaign created"))
}

async fn get_campaign(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(campaign_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let campaign = campaign_service::get_campaign(&state.pool, tenant.workspace_id, campaign_id).await?;
    Ok(json_success(campaign, "Campaign retrieved"))
}

async fn update_campaign(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(campaign_id): Path<Uuid>,
    Json(dto): Json<UpdateCampaignDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    let campaign = campaign_service::update_campaign(&state.pool, tenant.workspace_id, campaign_id, dto).await?;
    Ok(json_success(campaign, "Campaign updated"))
}

async fn launch_campaign(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(campaign_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    let campaign = campaign_service::launch_campaign(
        &state.pool,
        &state.http_client,
        &state.config,
        tenant.workspace_id,
        campaign_id,
    )
    .await?;
    Ok(json_success(campaign, "Campaign launched"))
}

async fn pause_campaign(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(campaign_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    let campaign = campaign_service::pause_campaign(&state.pool, tenant.workspace_id, campaign_id).await?;
    Ok(json_success(campaign, "Campaign paused"))
}
