use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::models::{CreateContactDto, UpdateContactDto};
use crate::services::contact_service;
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
        .route("/", get(list_contacts).post(create_contact))
        .route("/:id", get(get_contact).put(update_contact).delete(delete_contact))
        .with_state(state)
}

#[derive(Deserialize)]
struct ContactQuery {
    #[serde(flatten)]
    pagination: PaginationQuery,
    search: Option<String>,
}

async fn list_contacts(
    State(state): State<AppState>,
    tenant: TenantContext,
    Query(query): Query<ContactQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (contacts, meta) = contact_service::list_contacts(
        &state.pool,
        tenant.workspace_id,
        query.pagination,
        query.search,
    )
    .await?;

    Ok(json_paginated(contacts, meta, "Contacts retrieved"))
}

async fn get_contact(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(contact_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let contact = contact_service::get_contact(&state.pool, tenant.workspace_id, contact_id).await?;
    Ok(json_success(contact, "Contact retrieved"))
}

async fn create_contact(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<CreateContactDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    let contact = contact_service::create_contact(&state.pool, tenant.workspace_id, dto).await?;
    Ok(json_success(contact, "Contact created"))
}

async fn update_contact(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(contact_id): Path<Uuid>,
    Json(dto): Json<UpdateContactDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    let contact = contact_service::update_contact(&state.pool, tenant.workspace_id, contact_id, dto).await?;
    Ok(json_success(contact, "Contact updated"))
}

async fn delete_contact(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(contact_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    contact_service::delete_contact(&state.pool, tenant.workspace_id, contact_id).await?;
    Ok(json_success(serde_json::json!({ "deleted": true }), "Contact deleted"))
}
