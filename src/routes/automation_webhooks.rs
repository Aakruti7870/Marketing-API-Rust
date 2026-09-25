use crate::{error::AppError,state::AppState,services::automation_service};
use axum::{extract::{Path,State},http::{HeaderMap,StatusCode},response::IntoResponse,Json};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

pub async fn handle(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query("SELECT workspace_id, COALESCE(config->>'secret','') AS secret FROM automations a JOIN automation_triggers t ON t.automation_id=a.id WHERE a.id=$1 AND a.status='PUBLISHED' AND t.trigger_type='WEBHOOK' AND t.enabled LIMIT 1")
        .bind(id).fetch_optional(&s.pool).await?
        .ok_or_else(|| AppError::NotFound("Webhook automation not found".into()))?;
    let wid: Uuid = row.try_get("workspace_id")?;
    let secret: String = row.try_get("secret")?;
    if !secret.is_empty() && headers.get("x-automation-secret").and_then(|v| v.to_str().ok()) != Some(secret.as_str()) {
        return Err(AppError::Unauthorized("Invalid automation webhook secret".into()));
    }
    let run_id = automation_service::trigger(&s, id, wid, "WEBHOOK", payload).await?;
    Ok((StatusCode::ACCEPTED, Json(serde_json::json!({"accepted":true,"run_id":run_id}))))
}
