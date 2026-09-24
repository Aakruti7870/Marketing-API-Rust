use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::models::{CreateAutomationDto, RunAutomationDto, UpdateAutomationDto};
use crate::services::automation_service;
use crate::state::AppState;
use axum::{extract::{Path, State}, routing::{get,post,put}, Json, Router};
use serde_json::json;
use uuid::Uuid;

pub fn routes(state: AppState)->Router{
    Router::new()
      .route("/",get(list).post(create))
      .route("/:id",get(get_one).put(update))
      .route("/:id/activate",post(activate))
      .route("/:id/pause",post(pause))
      .route("/:id/run",post(run))
      .route("/:id/runs",get(runs))
      .route("/:id/runs/:run_id/steps",get(run_steps))
      .with_state(state)
}

async fn list(State(state):State<AppState>,tenant:TenantContext)->Result<Json<serde_json::Value>,AppError>{
    Ok(Json(json!({"data":automation_service::list(&state.pool,tenant.workspace_id).await?,"message":"Automations retrieved"})))
}
async fn get_one(State(state):State<AppState>,tenant:TenantContext,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>,AppError>{
    Ok(Json(json!({"data":automation_service::get(&state.pool,tenant.workspace_id,id).await?,"message":"Automation retrieved"})))
}
async fn create(State(state):State<AppState>,tenant:TenantContext,Json(dto):Json<CreateAutomationDto>)->Result<Json<serde_json::Value>,AppError>{
    require_workspace_roles(&tenant.workspace_role,&["OWNER","ADMIN","MEMBER"])?;
    Ok(Json(json!({"data":automation_service::create(&state.pool,tenant.workspace_id,tenant.user_id,dto).await?,"message":"Automation created"})))
}
async fn update(State(state):State<AppState>,tenant:TenantContext,Path(id):Path<Uuid>,Json(dto):Json<UpdateAutomationDto>)->Result<Json<serde_json::Value>,AppError>{
    require_workspace_roles(&tenant.workspace_role,&["OWNER","ADMIN","MEMBER"])?;
    Ok(Json(json!({"data":automation_service::update(&state.pool,tenant.workspace_id,id,dto).await?,"message":"Automation updated"})))
}
async fn activate(State(state):State<AppState>,tenant:TenantContext,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>,AppError>{
    require_workspace_roles(&tenant.workspace_role,&["OWNER","ADMIN"])?;
    Ok(Json(json!({"data":automation_service::set_status(&state.pool,tenant.workspace_id,id,"ACTIVE").await?,"message":"Automation activated"})))
}
async fn pause(State(state):State<AppState>,tenant:TenantContext,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>,AppError>{
    require_workspace_roles(&tenant.workspace_role,&["OWNER","ADMIN"])?;
    Ok(Json(json!({"data":automation_service::set_status(&state.pool,tenant.workspace_id,id,"PAUSED").await?,"message":"Automation paused"})))
}
async fn run(State(state):State<AppState>,tenant:TenantContext,Path(id):Path<Uuid>,Json(dto):Json<RunAutomationDto>)->Result<Json<serde_json::Value>,AppError>{
    require_workspace_roles(&tenant.workspace_role,&["OWNER","ADMIN","MEMBER"])?;
    Ok(Json(json!({"data":automation_service::run(&state.pool,&state.http_client,tenant.workspace_id,id,dto.payload.unwrap_or_else(||json!({}))).await?,"message":"Automation executed"})))
}
async fn run_steps(State(state):State<AppState>,tenant:TenantContext,Path((id,run_id)):Path<(Uuid,Uuid)>)->Result<Json<serde_json::Value>,AppError>{
    Ok(Json(json!({"data":automation_service::run_steps(&state.pool,tenant.workspace_id,id,run_id).await?,"message":"Automation run steps retrieved"})))
}
async fn runs(State(state):State<AppState>,tenant:TenantContext,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>,AppError>{
    Ok(Json(json!({"data":automation_service::runs(&state.pool,tenant.workspace_id,id).await?,"message":"Automation runs retrieved"})))
}
