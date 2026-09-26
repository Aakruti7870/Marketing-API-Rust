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
    .route("/runs/:run_id/steps/:step_id/decline",post(decline))
    .route("/:id",axum::routing::delete(remove))
    .with_state(state)
}
async fn list(State(s):State<AppState>,t:TenantContext)->Result<impl IntoResponse,AppError>{Ok(json_success(automation_service::list_automations(&s.pool,t.workspace_id).await?,"Automations retrieved"))}
async fn create(State(s):State<AppState>,t:TenantContext,Json(d):Json<AutomationDefinition>)->Result<impl IntoResponse,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN"])?;Ok(json_success(automation_service::create_automation(&s,t.workspace_id,t.user_id,d).await?,"Automation created"))}
async fn publish(State(s):State<AppState>,t:TenantContext,Path(id):Path<Uuid>)->Result<impl IntoResponse,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN"])?;Ok(json_success(automation_service::publish(&s,t.workspace_id,id).await?,"Automation published"))}
async fn run(State(s):State<AppState>,t:TenantContext,Path(id):Path<Uuid>,Json(d):Json<TriggerDto>)->Result<impl IntoResponse,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN","MEMBER"])?;let rid=automation_service::trigger(&s,id,t.workspace_id,"MANUAL",d.payload).await?;Ok(json_success(json!({"run_id":rid}),"Automation executed"))}
async fn get_run(State(s):State<AppState>,t:TenantContext,Path(rid):Path<Uuid>)->Result<impl IntoResponse,AppError>{Ok(json_success(automation_service::get_run(&s.pool,t.workspace_id,rid).await?,"Automation run retrieved"))}

async fn approve(State(s):State<AppState>,t:TenantContext,Path((rid,sid)):Path<(Uuid,Uuid)>)->Result<impl IntoResponse,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN"])?;automation_service::approve_run(&s,t.workspace_id,rid,sid).await?;Ok(json_success(json!({"run_id":rid,"approved":true,"outcome":"approved"}),"Automation approval accepted and execution resumed"))}
async fn decline(State(s):State<AppState>,t:TenantContext,Path((rid,sid)):Path<(Uuid,Uuid)>)->Result<impl IntoResponse,AppError>{require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN"])?;automation_service::decline_run(&s,t.workspace_id,rid,sid).await?;Ok(json_success(json!({"run_id":rid,"approved":false,"outcome":"declined"}),"Automation approval declined and execution resumed"))}
async fn remove(State(s):State<AppState>,t:TenantContext,Path(id):Path<Uuid>)->Result<impl IntoResponse,AppError>{\n    require_workspace_roles(&t.workspace_role,&["OWNER","ADMIN"])?;\n    let mut tx=s.pool.begin().await?;\n    let exists=sqlx::query("SELECT 1 FROM automations WHERE id=$1 AND workspace_id=$2").bind(id).fetch_optional(&mut *tx).await?;\n    if exists.is_none(){return Err(AppError::NotFound("Automation not found".into()));}\n    // Explicit child cleanup keeps deletion compatible with databases created before the\n    // automation foreign keys were hardened with ON DELETE CASCADE.\n    sqlx::query("DELETE FROM automation_logs WHERE run_id IN (SELECT id FROM automation_runs WHERE automation_id=$1)").bind(id).execute(&mut *tx).await?;\n    sqlx::query("DELETE FROM automation_run_steps WHERE run_id IN (SELECT id FROM automation_runs WHERE automation_id=$1)").bind(id).execute(&mut *tx).await?;\n    sqlx::query("DELETE FROM automation_runs WHERE automation_id=$1").bind(id).execute(&mut *tx).await?;\n    sqlx::query("DELETE FROM automation_schedules WHERE automation_id=$1").bind(id).execute(&mut *tx).await?;\n    sqlx::query("DELETE FROM automation_variables WHERE automation_id=$1").bind(id).execute(&mut *tx).await?;\n    sqlx::query("DELETE FROM automation_edges WHERE automation_id=$1").bind(id).execute(&mut *tx).await?;\n    sqlx::query("DELETE FROM automation_nodes WHERE automation_id=$1").bind(id).execute(&mut *tx).await?;\n    sqlx::query("DELETE FROM automation_triggers WHERE automation_id=$1").bind(id).execute(&mut *tx).await?;\n    sqlx::query("DELETE FROM automations WHERE id=$1 AND workspace_id=$2").bind(id).bind(t.workspace_id).execute(&mut *tx).await?;\n    tx.commit().await?;\n    Ok(json_success(json!({"id":id,"deleted":true}),"Automation deleted"))\n}
