use crate::{error::AppError,state::AppState,services::automation_service};
use axum::{extract::{Path,State},http::StatusCode,response::IntoResponse,Json};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

pub async fn handle(State(s):State<AppState>,Path(id):Path<Uuid>,Json(payload):Json<Value>)->Result<impl IntoResponse,AppError>{
    let row=sqlx::query("SELECT workspace_id FROM automations a WHERE a.id=$1 AND a.status='PUBLISHED' AND EXISTS(SELECT 1 FROM automation_triggers t WHERE t.automation_id=a.id AND t.trigger_type='WEBHOOK' AND t.enabled)")
        .bind(id).fetch_optional(&s.pool).await?.ok_or_else(||AppError::NotFound("Webhook automation not found".into()))?;
    let wid:Uuid=row.try_get("workspace_id")?;
    let run_id=automation_service::trigger(&s,id,wid,"WEBHOOK",payload).await?;
    Ok((StatusCode::ACCEPTED,Json(serde_json::json!({"accepted":true,"run_id":run_id}))))
}
