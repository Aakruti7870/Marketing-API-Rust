use crate::error::AppError;
use crate::models::{Automation, AutomationGraph, AutomationNode, AutomationRun, AutomationRunStep, CreateAutomationDto, UpdateAutomationDto};
use reqwest::Method;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

pub async fn list(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<Automation>, AppError> {
    Ok(sqlx::query_as::<_, Automation>(
        "SELECT * FROM automations WHERE workspace_id = $1 ORDER BY updated_at DESC"
    ).bind(workspace_id).fetch_all(pool).await.map_err(AppError::Database)?)
}

pub async fn get(pool: &PgPool, workspace_id: Uuid, id: Uuid) -> Result<Automation, AppError> {
    sqlx::query_as::<_, Automation>(
        "SELECT * FROM automations WHERE workspace_id = $1 AND id = $2"
    ).bind(workspace_id).bind(id).fetch_optional(pool).await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Automation not found".into()))
}

pub async fn create(pool: &PgPool, workspace_id: Uuid, user_id: Uuid, dto: CreateAutomationDto) -> Result<Automation, AppError> {
    validate_graph(&dto.graph)?;
    let id = Uuid::new_v4();
    let webhook_token = if dto.trigger_type.eq_ignore_ascii_case("WEBHOOK") { Some(Uuid::new_v4().to_string().replace("-","")) } else { None };
    let graph = serde_json::to_value(dto.graph).map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(sqlx::query_as::<_, Automation>(
        "INSERT INTO automations (id,workspace_id,name,description,status,trigger_type,trigger_config,webhook_token,graph,created_by_id)
         VALUES ($1,$2,$3,$4,'DRAFT',$5,$6,$7,$8,$9) RETURNING *"
    ).bind(id).bind(workspace_id).bind(dto.name).bind(dto.description).bind(dto.trigger_type)
     .bind(dto.trigger_config.unwrap_or_else(|| json!({}))).bind(webhook_token).bind(graph).bind(user_id)
     .fetch_one(pool).await.map_err(AppError::Database)?)
}

pub async fn update(pool: &PgPool, workspace_id: Uuid, id: Uuid, dto: UpdateAutomationDto) -> Result<Automation, AppError> {
    let current = get(pool, workspace_id, id).await?;
    let graph = if let Some(g)=dto.graph { validate_graph(&g)?; Some(serde_json::to_value(g).map_err(|e|AppError::BadRequest(e.to_string()))?) } else { None };
    let status = if current.status=="ACTIVE" { "DRAFT".to_string() } else { current.status };
    sqlx::query_as::<_, Automation>(
        "UPDATE automations SET name=COALESCE($3,name),description=COALESCE($4,description),trigger_type=COALESCE($5,trigger_type),
         trigger_config=COALESCE($6,trigger_config),graph=COALESCE($7,graph),status=$8,version=version+1,updated_at=NOW()
         WHERE workspace_id=$1 AND id=$2 RETURNING *"
    ).bind(workspace_id).bind(id).bind(dto.name).bind(dto.description).bind(dto.trigger_type).bind(dto.trigger_config).bind(graph).bind(status)
     .fetch_one(pool).await.map_err(AppError::Database)
}

pub async fn set_status(pool:&PgPool, workspace_id:Uuid,id:Uuid,status:&str)->Result<Automation,AppError>{
    if !["DRAFT","ACTIVE","PAUSED"].contains(&status){return Err(AppError::BadRequest("Invalid automation status".into()));}
    sqlx::query_as::<_,Automation>("UPDATE automations SET status=$3,updated_at=NOW() WHERE workspace_id=$1 AND id=$2 RETURNING *")
        .bind(workspace_id).bind(id).bind(status).fetch_optional(pool).await.map_err(AppError::Database)?
        .ok_or_else(||AppError::NotFound("Automation not found".into()))
}

pub async fn runs(pool:&PgPool,workspace_id:Uuid,id:Uuid)->Result<Vec<AutomationRun>,AppError>{
    Ok(sqlx::query_as::<_,AutomationRun>("SELECT * FROM automation_runs WHERE workspace_id=$1 AND automation_id=$2 ORDER BY started_at DESC LIMIT 50")
        .bind(workspace_id).bind(id).fetch_all(pool).await.map_err(AppError::Database)?)
}

pub async fn run(pool:&PgPool,http:&reqwest::Client,workspace_id:Uuid,id:Uuid,payload:Value)->Result<AutomationRun,AppError>{
    let automation=get(pool,workspace_id,id).await?;
    if automation.status!="ACTIVE" && automation.status!="DRAFT" { return Err(AppError::BadRequest("Automation is paused".into())); }
    let run_id=Uuid::new_v4();
    let mut execution=sqlx::query_as::<_,AutomationRun>(
        "INSERT INTO automation_runs (id,automation_id,workspace_id,status,trigger_payload) VALUES ($1,$2,$3,'RUNNING',$4) RETURNING *")
        .bind(run_id).bind(id).bind(workspace_id).bind(payload.clone()).fetch_one(pool).await.map_err(AppError::Database)?;
    let graph:AutomationGraph=serde_json::from_value(automation.graph.clone()).map_err(|e|AppError::BadRequest(format!("Invalid workflow graph: {e}")))?;
    let result=execute_graph(pool,http,run_id,&graph,payload).await;
    match result {
        Ok(output)=>{
            execution=sqlx::query_as::<_,AutomationRun>("UPDATE automation_runs SET status='COMPLETED',output_data=$2,completed_at=NOW() WHERE id=$1 RETURNING *")
                .bind(run_id).bind(output).fetch_one(pool).await.map_err(AppError::Database)?;
        }
        Err(e)=>{
            execution=sqlx::query_as::<_,AutomationRun>("UPDATE automation_runs SET status='FAILED',error_message=$2,completed_at=NOW() WHERE id=$1 RETURNING *")
                .bind(run_id).bind(e.to_string()).fetch_one(pool).await.map_err(AppError::Database)?;
        }
    }
    Ok(execution)
}

async fn execute_graph(pool:&PgPool,http:&reqwest::Client,run_id:Uuid,graph:&AutomationGraph,payload:Value)->Result<Value,AppError>{
    let mut nodes:HashMap<String,&AutomationNode>=HashMap::new();
    for n in &graph.nodes { nodes.insert(n.id.clone(),n); }
    let mut incoming=HashSet::new();
    let mut next:HashMap<String,Vec<String>>=HashMap::new();
    for e in &graph.edges { incoming.insert(e.target.clone()); next.entry(e.source.clone()).or_default().push(e.target.clone()); }
    let mut queue:VecDeque<String>=graph.nodes.iter().filter(|n|!incoming.contains(&n.id)).map(|n|n.id.clone()).collect();
    let mut data=payload;
    let mut visited=HashSet::new();
    while let Some(id)=queue.pop_front(){
        if !visited.insert(id.clone()){continue}
        let node=nodes.get(&id).ok_or_else(||AppError::BadRequest(format!("Unknown node {id}")))?;
        let step_id=Uuid::new_v4();
        sqlx::query("INSERT INTO automation_run_steps (id,run_id,node_id,node_type,status,input_data) VALUES ($1,$2,$3,$4,'RUNNING',$5)")
            .bind(step_id).bind(run_id).bind(&node.id).bind(&node.node_type).bind(&data).execute(pool).await.map_err(AppError::Database)?;
        let result=execute_node(http,node,&data).await;
        match result {
            Ok(out)=>{
                sqlx::query("UPDATE automation_run_steps SET status='COMPLETED',output_data=$2,completed_at=NOW() WHERE id=$1").bind(step_id).bind(&out).execute(pool).await.map_err(AppError::Database)?;
                data=out;
                if let Some(children)=next.get(&id){ for c in children {queue.push_back(c.clone());}}
            }
            Err(e)=>{
                sqlx::query("UPDATE automation_run_steps SET status='FAILED',error_message=$2,completed_at=NOW() WHERE id=$1").bind(step_id).bind(e.to_string()).execute(pool).await.map_err(AppError::Database)?;
                return Err(e);
            }
        }
    }
    Ok(data)
}

async fn execute_node(http:&reqwest::Client,node:&AutomationNode,input:&Value)->Result<Value,AppError>{
    match node.node_type.as_str() {
        "trigger.manual"|"trigger.webhook"|"trigger.schedule"|"data.set" => Ok(node.config.get("data").cloned().unwrap_or_else(||input.clone())),
        "logic.condition" => {
            let path=node.config.get("path").and_then(Value::as_str).unwrap_or("");
            let expected=node.config.get("equals").cloned().unwrap_or(Value::Null);
            let actual=path.split('.').filter(|x|!x.is_empty()).fold(input,|v,k|v.get(k).unwrap_or(&Value::Null));
            if actual==&expected { Ok(input.clone()) } else { Ok(json!({"condition":false,"input":input})) }
        }
        "action.http"|"http.request" => {
            let method=node.config.get("method").and_then(Value::as_str).unwrap_or("GET").parse::<Method>().map_err(|_|AppError::BadRequest("Invalid HTTP method".into()))?;
            let url=node.config.get("url").and_then(Value::as_str).ok_or_else(||AppError::BadRequest("HTTP node requires url".into()))?;
            let mut req=http.request(method,url);
            if let Some(headers)=node.config.get("headers").and_then(Value::as_object){for (k,v) in headers{if let Some(s)=v.as_str(){req=req.header(k,s);}}}
            let body=node.config.get("body").cloned().unwrap_or_else(||input.clone());
            let response=if method==Method::GET||method==Method::HEAD{req.send().await}else{req.json(&body).send().await};
            let response=response.map_err(|e|AppError::BadRequest(format!("HTTP request failed: {e}")))?;
            let status=response.status().as_u16();
            let text=response.text().await.map_err(|e|AppError::BadRequest(format!("HTTP response failed: {e}")))?;
            let parsed=serde_json::from_str::<Value>(&text).unwrap_or_else(|_|json!({"status":status,"body":text}));
            Ok(json!({"status":status,"response":parsed}))
        }
        "delay.wait" => {
            let ms=node.config.get("milliseconds").and_then(Value::as_u64).unwrap_or(1000).min(30000);
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            Ok(input.clone())
        }
        _ => Err(AppError::BadRequest(format!("Unsupported automation node type: {}",node.node_type)))
    }
}

fn validate_graph(graph:&AutomationGraph)->Result<(),AppError>{
    if graph.nodes.is_empty(){return Err(AppError::BadRequest("Automation needs at least one node".into()));}
    let ids:HashSet<_>=graph.nodes.iter().map(|n|n.id.as_str()).collect();
    if ids.len()!=graph.nodes.len(){return Err(AppError::BadRequest("Automation node IDs must be unique".into()));}
    for e in &graph.edges {if !ids.contains(e.source.as_str())||!ids.contains(e.target.as_str()){return Err(AppError::BadRequest("Automation edge references an unknown node".into()));}}
    Ok(())
}
