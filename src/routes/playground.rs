use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::services::playground_service;
use crate::state::AppState;
use crate::utils::response::json_success;
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/generate", post(generate))
        .route("/image", post(generate_image))
        .route("/assets", get(list_assets))
        .route("/groups", get(list_groups))
        .route("/button-actions/:asset_id", get(list_button_actions))
        .route("/button-actions", post(save_button_action))
        .route("/share-whatsapp", post(share_whatsapp))
        .route("/channels/:channel_id/import-contacts", post(import_contacts))
        .with_state(state)
}

pub fn public_routes(state: AppState) -> Router {
    Router::new()
        .route("/assets/:public_key", get(public_asset))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct GenerateRequest {
    kind: String,
    prompt: String,
    business_context: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageRequest {
    kind: String,
    prompt: String,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ShareWhatsAppRequest {
    channel_id: Uuid,
    group_id: Uuid,
    asset_id: Uuid,
    caption: Option<String>,
}

async fn generate(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<GenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    Ok(json_success(
        playground_service::generate_text(
            &state.config,
            &state.pool,
            tenant.workspace_id,
            tenant.user_id,
            &dto.kind,
            &dto.prompt,
            dto.business_context.as_deref().unwrap_or(""),
        ).await?,
        "Playground content generated",
    ))
}

async fn generate_image(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<ImageRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    Ok(json_success(
        playground_service::generate_image(
            &state.config,
            &state.pool,
            tenant.workspace_id,
            tenant.user_id,
            &dto.kind,
            &dto.prompt,
            dto.title.as_deref().unwrap_or("GOLD-e Playground image"),
        ).await?,
        "Playground image generated",
    ))
}


async fn list_button_actions(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_success(
        playground_service::list_button_actions(&state.pool, tenant.workspace_id, asset_id).await?,
        "Playground button actions retrieved",
    ))
}

async fn save_button_action(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<crate::models::PlaygroundButtonActionDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    Ok(json_success(
        playground_service::save_button_action(&state.pool, tenant.workspace_id, dto).await?,
        "Playground button action saved",
    ))
}

async fn list_groups(
    State(state): State<AppState>,
    tenant: TenantContext,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_success(
        playground_service::list_groups(&state.pool, tenant.workspace_id).await?,
        "Contact groups retrieved",
    ))
}

async fn list_assets(
    State(state): State<AppState>,
    tenant: TenantContext,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_success(
        playground_service::list_assets(&state.pool, tenant.workspace_id).await?,
        "Playground assets retrieved",
    ))
}

async fn share_whatsapp(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<ShareWhatsAppRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN", "MEMBER"])?;
    Ok(json_success(
        playground_service::share_whatsapp(
            &state.config,
            &state.pool,
            tenant.workspace_id,
            dto.channel_id,
            dto.group_id,
            dto.asset_id,
            dto.caption.as_deref(),
        ).await?,
        "WhatsApp distribution completed",
    ))
}

async fn import_contacts(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(channel_id): Path<Uuid>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;
    Ok(json_success(
        playground_service::import_contacts(&state.pool, tenant.workspace_id, tenant.user_id, channel_id, multipart).await?,
        "Contacts imported and grouped",
    ))
}

async fn public_asset(
    State(state): State<AppState>,
    Path(public_key): Path<String>,
) -> Result<Response<Body>, AppError> {
    let row = sqlx::query_as::<_, (Option<Vec<u8>>, Option<String>)>(
        "SELECT media_data, mime_type FROM playground_assets WHERE public_key=$1"
    )
    .bind(&public_key)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Playground asset not found".into()))?;

    let data = row.0.ok_or_else(|| AppError::NotFound("This playground asset has no media payload".into()))?;
    let mime = row.1.unwrap_or_else(|| "application/octet-stream".into());
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(data))
        .map_err(|e| AppError::ExternalService(format!("Unable to build asset response: {}", e)))?;
    Ok(response)
}
