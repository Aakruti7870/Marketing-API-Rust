use crate::auth::AuthenticatedUser;
use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::services::workspace_service::{
    self, AddMemberDto, CreateWorkspaceDto, UpdateWorkspaceDto,
};
use crate::state::AppState;
use crate::utils::response::json_success;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use uuid::Uuid;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_workspaces).post(create_workspace))
        .route("/current", get(get_current_workspace).put(update_current_workspace).delete(delete_current_workspace))
        .route("/current/members", get(list_workspace_members).post(add_workspace_member))
        .route("/current/members/:id", delete(remove_workspace_member))
        .with_state(state)
}

async fn list_workspaces(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let workspaces = workspace_service::list_user_workspaces(&state.pool, claims.sub).await?;
    Ok(json_success(workspaces, "Workspaces retrieved"))
}

async fn create_workspace(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(dto): Json<CreateWorkspaceDto>,
) -> Result<impl IntoResponse, AppError> {
    let workspace = workspace_service::create_workspace(&state.pool, claims.sub, dto).await?;
    Ok(json_success(workspace, "Workspace created"))
}

async fn get_current_workspace(
    State(state): State<AppState>,
    tenant: TenantContext,
) -> Result<impl IntoResponse, AppError> {
    let workspace = workspace_service::get_workspace(&state.pool, tenant.workspace_id).await?;
    Ok(json_success(workspace, "Workspace fetched"))
}

async fn update_current_workspace(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<UpdateWorkspaceDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    let workspace = workspace_service::update_workspace(&state.pool, tenant.workspace_id, dto).await?;
    Ok(json_success(workspace, "Workspace updated"))
}

async fn delete_current_workspace(
    State(state): State<AppState>,
    tenant: TenantContext,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER"])?;
    workspace_service::delete_workspace(&state.pool, tenant.workspace_id).await?;
    Ok(json_success(serde_json::json!({ "deleted": true }), "Workspace deleted"))
}

async fn list_workspace_members(
    State(state): State<AppState>,
    tenant: TenantContext,
) -> Result<impl IntoResponse, AppError> {
    let members = workspace_service::list_members(&state.pool, tenant.workspace_id).await?;
    Ok(json_success(members, "Members retrieved"))
}

async fn add_workspace_member(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<AddMemberDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    let member = workspace_service::add_member(&state.pool, tenant.workspace_id, dto).await?;
    Ok(json_success(member, "Member invited"))
}

async fn remove_workspace_member(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(member_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    workspace_service::remove_member(&state.pool, tenant.workspace_id, member_id).await?;
    Ok(json_success(serde_json::json!({ "removed": true }), "Member removed"))
}
