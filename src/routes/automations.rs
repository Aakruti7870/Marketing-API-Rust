use crate::{error::AppError,middleware::{require_workspace_roles,TenantContext},models::AutomationDefinition,services::automation_service,state::AppState,utils::response::json_success};
use axum::{extract::{Path,State},response::IntoResponse,routing::{get,post},Json,Router};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use uuid::Uuid;

#[derive(Debug,Serialize,Deserialize)]
struct TriggerDto { #[serde(default)] payload: Value }

pub fn routes(state:AppState)->Router{Router::new()
    .route("/",get(list).post(create))
    .route("/:id/publish",post(publish))
    .route("/:id/run",post(run))
    .route("/runs/:run_id",get(get_run))
    .route("/runs/:run_id/steps/:step_id/approve",post(approve))
    .with_state(state)
}
async fn list(State(s):State<AppState>,t:TenantContext)->Result<impl IntoResponse,AppError>{Ok(json_success(automation_service::list_automations(&s.pool,t.workspace_id).await?,"Automations retrieved"))}
async fn create(State(s):State<AppState>,t:TenantContext,Json(d):Json<AutomationDefinition>)->Result<Json<Value>,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN"])?;Ok(json_success(automation_service::create_automation(&s,t.workspace_id,t.user_id,d).await?,"Automation created"))}
async fn publish(State(s):State<AppState>,t:TenantContext,Path(id):Path<Uuid>)->Result<Json<Value>,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN"])?;Ok(json_success(automation_service::publish(&s,t.workspace_id,id).await?,"Automation published"))}
async fn run(State(s):State<AppState>,t:TenantContext,Path(id):Path<Uuid>,Json(d):Json<TriggerDto>)->Result<Json<Value>,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN","MEMBER"])?;let rid=automation_service::trigger(&s,id,t.workspace_id,"MANUAL",d.payload).await?;Ok(json_success(json!({"run_id":rid}),"Automation executed"))}
async fn get_run(State(s):State<AppState>,t:TenantContext,Path(rid):Path<Uuid>)->Result<Json<Value>,AppError>{Ok(json_success(automation_service::get_run(&s.pool,t.workspace_id,rid).await?,"Automation run retrieved"))}

async fn approve(State(s):State<AppState>,t:TenantContext,Path((rid,sid)):Path<(Uuid,Uuid)>)->Result<Json<Value>,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN"])?;automation_service::approve_run(&s,t.workspace_id,rid,sid).await?;Ok(json_success(json!({"run_id":rid,"approved":true}),"Automation approval accepted and execution resumed"))}
