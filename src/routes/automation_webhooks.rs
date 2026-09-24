use crate::error::AppError;
use crate::services::automation_service;
use crate::state::AppState;
use axum::{extract::{Path,State}, Json};
use serde_json::Value;

pub async fn trigger(State(state):State<AppState>,Path(token):Path<String>,Json(payload):Json<Value>)->Result<Json<Value>,AppError>{
    let automation=sqlx::query_as::<_,crate::models::Automation>("SELECT * FROM automations WHERE webhook_token=$1 AND status='ACTIVE'")
        .bind(&token).fetch_optional(&state.pool).await.map_err(AppError::Database)?
        .ok_or_else(||AppError::NotFound("Automation webhook not found or inactive".into()))?;
    let run=automation_service::run(&state.pool,&state.http_client,automation.workspace_id,automation.id,payload).await?;
    Ok(Json(serde_json::json!({"data":run,"message":"Webhook automation executed"})))
}
