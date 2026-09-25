use crate::{error::AppError, models::*, services::whatsapp_service, state::AppState};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::Row;
use std::{collections::{HashMap, HashSet}, time::Duration};
use uuid::Uuid;

const MAX_NODES: usize = 100;
const MAX_RUN_SECONDS: u64 = 300;

pub async fn create_automation(state: &AppState, workspace_id: Uuid, user_id: Uuid, dto: AutomationDefinition) -> Result<AutomationSummary, AppError> {
    if dto.name.trim().is_empty() || dto.nodes.is_empty() || dto.nodes.len() > MAX_NODES {
        return Err(AppError::BadRequest("Automation name and 1-100 nodes are required".into()));
    }
    let mut tx = state.pool.begin().await?;
    let row = sqlx::query("INSERT INTO automations(workspace_id,name,description,created_by_id) VALUES($1,$2,$3,$4) RETURNING id,name,description,status,version,created_at,updated_at")
        .bind(workspace_id).bind(dto.name.trim()).bind(&dto.description).bind(user_id)
        .fetch_one(&mut *tx).await?;
    let id: Uuid = row.try_get("id")?;

    for n in &dto.nodes {
        if n.node_key.trim().is_empty() || n.node_type.trim().is_empty() {
            return Err(AppError::BadRequest("Every node needs node_key and node_type".into()));
        }
        sqlx::query("INSERT INTO automation_nodes(automation_id,node_key,node_type,name,config,position) VALUES($1,$2,$3,$4,$5,$6)")
            .bind(id).bind(&n.node_key).bind(&n.node_type).bind(&n.name).bind(&n.config).bind(&n.position)
            .execute(&mut *tx).await?;
    }
    for e in &dto.edges {
        sqlx::query("INSERT INTO automation_edges(automation_id,source_node_key,target_node_key,config) VALUES($1,$2,$3,$4)")
            .bind(id).bind(&e.source_node_key).bind(&e.target_node_key).bind(&e.config).execute(&mut *tx).await?;
    }
    for t in &dto.triggers {
        sqlx::query("INSERT INTO automation_triggers(automation_id,trigger_type,config,enabled) VALUES($1,$2,$3,$4)")
            .bind(id).bind(&t.trigger_type).bind(&t.config).bind(t.enabled).execute(&mut *tx).await?;
    }
    if let Some(s) = &dto.schedule {
        if s.interval_seconds < 15 { return Err(AppError::BadRequest("Schedule interval must be at least 15 seconds".into())); }
        sqlx::query("INSERT INTO automation_schedules(automation_id,interval_seconds,next_run_at,enabled) VALUES($1,$2,NOW()+($2 * INTERVAL '1 second'),$3)")
            .bind(id).bind(s.interval_seconds).bind(s.enabled).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(AutomationSummary {
        id, name: dto.name, description: dto.description, status: "DRAFT".into(), version: 1,
        created_at: row.try_get("created_at")?, updated_at: row.try_get("updated_at")?
    })
}

pub async fn list_automations(pool: &sqlx::PgPool, workspace_id: Uuid) -> Result<Vec<AutomationSummary>, AppError> {
    let rows = sqlx::query("SELECT id,name,description,status,version,created_at,updated_at FROM automations WHERE workspace_id=$1 ORDER BY created_at DESC")
        .bind(workspace_id).fetch_all(pool).await?;
    rows.into_iter().map(|r| Ok(AutomationSummary {
        id: r.try_get("id")?, name: r.try_get("name")?, description: r.try_get("description")?,
        status: r.try_get("status")?, version: r.try_get("version")?, created_at: r.try_get("created_at")?, updated_at: r.try_get("updated_at")?
    })).collect()
}

pub async fn publish(state: &AppState, workspace_id: Uuid, id: Uuid) -> Result<AutomationSummary, AppError> {
    let trigger_count: i64 = sqlx::query("SELECT COUNT(*) AS count FROM automation_triggers WHERE automation_id=$1 AND enabled")
        .bind(id).fetch_one(&state.pool).await?.try_get("count")?;
    if trigger_count == 0 { return Err(AppError::BadRequest("Automation needs at least one enabled trigger".into())); }

    let row = sqlx::query("UPDATE automations SET status='PUBLISHED',version=version+1,published_at=NOW(),updated_at=NOW() WHERE id=$1 AND workspace_id=$2 RETURNING id,name,description,status,version,created_at,updated_at")
        .bind(id).bind(workspace_id).fetch_optional(&state.pool).await?
        .ok_or_else(|| AppError::NotFound("Automation not found".into()))?;
    if let Some(s) = sqlx::query("SELECT interval_seconds FROM automation_schedules WHERE automation_id=$1").bind(id).fetch_optional(&state.pool).await? {
        let secs:i64=s.try_get("interval_seconds")?;
        sqlx::query("UPDATE automation_schedules SET enabled=true,next_run_at=COALESCE(next_run_at,NOW()+($1 * INTERVAL '1 second')),updated_at=NOW() WHERE automation_id=$2")
            .bind(secs).bind(id).execute(&state.pool).await?;
    }
    Ok(AutomationSummary {
        id:row.try_get("id")?,name:row.try_get("name")?,description:row.try_get("description")?,
        status:row.try_get("status")?,version:row.try_get("version")?,created_at:row.try_get("created_at")?,updated_at:row.try_get("updated_at")?
    })
}

pub async fn trigger(state: &AppState, automation_id: Uuid, workspace_id: Uuid, trigger_type: &str, payload: Value) -> Result<Uuid, AppError> {
    let ok = sqlx::query("SELECT 1 FROM automations WHERE id=$1 AND workspace_id=$2 AND status='PUBLISHED' AND EXISTS(SELECT 1 FROM automation_triggers t WHERE t.automation_id=$1 AND t.trigger_type=$3 AND t.enabled)")
        .bind(automation_id).bind(workspace_id).bind(trigger_type).fetch_optional(&state.pool).await?;
    if ok.is_none() { return Err(AppError::NotFound("Published automation trigger not found".into())); }

    let row=sqlx::query("INSERT INTO automation_runs(automation_id,workspace_id,trigger_type,trigger_payload,variables) VALUES($1,$2,$3,$4,$4) RETURNING id")
        .bind(automation_id).bind(workspace_id).bind(trigger_type).bind(&payload).fetch_one(&state.pool).await?;
    let run_id:Uuid=row.try_get("id")?;
    if let Err(e)=execute_run(state,run_id,automation_id,payload).await {
        let msg=e.to_string();
        let _=sqlx::query("UPDATE automation_runs SET status='FAILED',error=$1,completed_at=NOW() WHERE id=$2").bind(msg).bind(run_id).execute(&state.pool).await;
    }
    Ok(run_id)
}

async fn execute_run(state:&AppState, run_id:Uuid, automation_id:Uuid, payload:Value)->Result<(),AppError>{
    let node_rows=sqlx::query("SELECT node_key,node_type,config FROM automation_nodes WHERE automation_id=$1").bind(automation_id).fetch_all(&state.pool).await?;
    let edge_rows=sqlx::query("SELECT source_node_key,target_node_key,config FROM automation_edges WHERE automation_id=$1").bind(automation_id).fetch_all(&state.pool).await?;
    let mut nodes=HashMap::new();
    for r in node_rows { nodes.insert(r.try_get::<String,_>("node_key")?, (r.try_get::<String,_>("node_type")?,r.try_get::<Value,_>("config")?)); }
    let mut edges:HashMap<String,Vec<(String,Value)>>=HashMap::new();
    for r in edge_rows { edges.entry(r.try_get("source_node_key")?).or_default().push((r.try_get("target_node_key")?,r.try_get("config")?)); }

    let start=nodes.iter().find(|(_, (t,_))| t.ends_with("TRIGGER")).map(|(k,_)|k.clone())
        .or_else(||nodes.keys().next().cloned()).ok_or_else(||AppError::BadRequest("Automation has no nodes".into()))?;
    let mut current=Some(start);
    let mut vars=json!({"trigger":payload});
    let mut visited=HashSet::new();
    let started=Utc::now();

    while let Some(key)=current {
        if !visited.insert(key.clone()) { return Err(AppError::BadRequest("Automation contains a cycle".into())); }
        if Utc::now().signed_duration_since(started).num_seconds() > MAX_RUN_SECONDS as i64 { return Err(AppError::Conflict("Automation exceeded its 5 minute safety limit".into())); }
        let (typ,cfg)=nodes.get(&key).ok_or_else(||AppError::BadRequest("Broken workflow edge".into()))?.clone();
        let step=sqlx::query("INSERT INTO automation_run_steps(run_id,node_key,node_type,input) VALUES($1,$2,$3,$4) RETURNING id")
            .bind(run_id).bind(&key).bind(&typ).bind(&vars).fetch_one(&state.pool).await?;
        let step_id:Uuid=step.try_get("id")?;
        if typ == "APPROVAL" {
            sqlx::query("UPDATE automation_run_steps SET status='WAITING_APPROVAL',completed_at=NULL WHERE id=$1").bind(step_id).execute(&state.pool).await?;
            sqlx::query("UPDATE automation_runs SET status='WAITING_APPROVAL',variables=$1 WHERE id=$2").bind(&vars).bind(run_id).execute(&state.pool).await?;
            return Ok(());
        }
        match execute_node(state,&typ,&cfg,&vars).await {
            Ok(out)=>{
                vars=merge(vars,out.clone());
                sqlx::query("UPDATE automation_run_steps SET status='COMPLETED',output=$1,completed_at=NOW() WHERE id=$2").bind(out).bind(step_id).execute(&state.pool).await?;
                sqlx::query("INSERT INTO automation_logs(run_id,node_key,message,metadata) VALUES($1,$2,$3,$4)").bind(run_id).bind(&key).bind("Node completed").bind(json!({"node_type":typ})).execute(&state.pool).await?;
            }
            Err(e)=>{
                let msg=e.to_string();
                sqlx::query("UPDATE automation_run_steps SET status='FAILED',error=$1,completed_at=NOW() WHERE id=$2").bind(&msg).bind(step_id).execute(&state.pool).await?;
                return Err(e);
            }
        }
        current=edges.get(&key).and_then(|outs|outs.iter().find(|(_,c)|condition_matches(c,&vars)).map(|(target,_)|target.clone()));
    }
    sqlx::query("UPDATE automation_runs SET status='COMPLETED',variables=$1,completed_at=NOW() WHERE id=$2").bind(vars).bind(run_id).execute(&state.pool).await?;
    Ok(())
}

async fn execute_node(state:&AppState,typ:&str,cfg:&Value,vars:&Value)->Result<Value,AppError>{
    match typ {
        "WEBHOOK_TRIGGER"|"SCHEDULE_TRIGGER"|"MANUAL_TRIGGER" => Ok(json!({"triggered":true})),
        "SET"|"TRANSFORM"|"VARIABLES" => {
            let mut out=json!({});
            if let Some(values)=cfg.get("values").and_then(Value::as_object) { for (k,v) in values { out[k]=interpolate(v,vars); } }
            Ok(out)
        },
        "IF"|"CONDITION" => Ok(json!({"condition":condition_matches(cfg,vars)})),
        "DELAY" => { let secs=cfg.get("seconds").and_then(Value::as_u64).unwrap_or(0).min(300); tokio::time::sleep(Duration::from_secs(secs)).await; Ok(json!({"delayed_seconds":secs})) },
        "HTTP_REQUEST" => {
            let url=cfg.get("url").and_then(Value::as_str).ok_or_else(||AppError::BadRequest("HTTP_REQUEST requires config.url".into()))?;
            let url=interpolate_str(url,vars);
            let mut req=match cfg.get("method").and_then(Value::as_str).unwrap_or("GET").to_uppercase().as_str() {
                "POST"=>state.http_client.post(&url),"PUT"=>state.http_client.put(&url),"PATCH"=>state.http_client.patch(&url),
                "DELETE"=>state.http_client.delete(&url),_=>state.http_client.get(&url)
            };
            if let Some(headers)=cfg.get("headers").and_then(Value::as_object) { for (k,v) in headers { if let Some(s)=v.as_str(){req=req.header(k,interpolate_str(s,vars));} } }
            if let Some(body)=cfg.get("body") { req=req.json(&interpolate(body,vars)); }
            let resp=req.send().await.map_err(|e|AppError::ExternalService(e.to_string()))?;
            let status=resp.status().as_u16(); let body=resp.json::<Value>().await.unwrap_or(Value::Null);
            Ok(json!({"status":status,"body":body}))
        },
        "WHATSAPP"|"WHATSAPP_MESSAGE" => {
            let to=cfg.get("to").and_then(Value::as_str).ok_or_else(||AppError::BadRequest("WHATSAPP requires config.to".into()))?;
            let content=cfg.get("content").and_then(Value::as_str).unwrap_or("");
            let result=whatsapp_service::send_whatsapp_message(&state.http_client,&state.config,&interpolate_str(to,vars),&interpolate_str(content,vars),cfg.get("template_name").and_then(Value::as_str)).await?;
            Ok(json!({"message_id":result.message_id,"status":result.status,"simulated":result.simulated}))
        },
        "APPROVAL" => Err(AppError::Conflict("Approval node paused execution; approval API is required".into())),
        _=>Err(AppError::BadRequest(format!("Unsupported node type: {}",typ)))
    }
}

fn condition_matches(c:&Value,vars:&Value)->bool {
    let Some(path)=c.get("path").and_then(Value::as_str) else { return true; };
    let mut cur=vars;
    for p in path.split('.') { cur=match cur.get(p){Some(v)=>v,None=>return false}; }
    match c.get("operator").and_then(Value::as_str).unwrap_or("exists") {
        "eq"=>cur==c.get("value").unwrap_or(&Value::Null),
        "neq"=>cur!=c.get("value").unwrap_or(&Value::Null),
        "truthy"=>cur.as_bool().unwrap_or(false),
        "exists"=>!cur.is_null(),
        _=>false
    }
}
fn merge(mut a:Value,b:Value)->Value { if let (Some(am),Some(bm))=(a.as_object_mut(),b.as_object()){for(k,v) in bm{am.insert(k.clone(),v.clone());}} a }
fn interpolate(v:&Value,vars:&Value)->Value { match v {Value::String(s)=>Value::String(interpolate_str(s,vars)),Value::Array(a)=>Value::Array(a.iter().map(|x|interpolate(x,vars)).collect()),Value::Object(m)=>Value::Object(m.iter().map(|(k,v)|(k.clone(),interpolate(v,vars))).collect()),x=>x.clone()} }
fn interpolate_str(s:&str,vars:&Value)->String { let mut out=s.to_string(); for _ in 0..20 { let Some(a)=out.find("{{") else{break}; let Some(rel)=out[a+2..].find("}}") else{break}; let b=a+2+rel; let mut v=vars; for p in out[a+2..b].trim().split('.') {v=match v.get(p){Some(x)=>x,None=>&Value::Null};} let rep=v.as_str().map(str::to_owned).unwrap_or_else(||v.to_string()); out.replace_range(a..b+2,&rep); } out }

pub async fn approve_run(state:&AppState, workspace_id:Uuid, run_id:Uuid, step_id:Uuid)->Result<(),AppError>{
    let run=sqlx::query("SELECT automation_id,variables,status FROM automation_runs WHERE id=$1 AND workspace_id=$2 FOR UPDATE")
        .bind(run_id).bind(workspace_id).fetch_optional(&state.pool).await?
        .ok_or_else(||AppError::NotFound("Automation run not found".into()))?;
    let status:String=run.try_get("status")?;
    if status!="WAITING_APPROVAL" { return Err(AppError::Conflict("Run is not waiting for approval".into())); }
    let automation_id:Uuid=run.try_get("automation_id")?;
    let mut vars:Value=run.try_get("variables")?;
    let step=sqlx::query("SELECT node_key,status FROM automation_run_steps WHERE id=$1 AND run_id=$2")
        .bind(step_id).bind(run_id).fetch_optional(&state.pool).await?
        .ok_or_else(||AppError::NotFound("Approval step not found".into()))?;
    let key:String=step.try_get("node_key")?;
    let step_status:String=step.try_get("status")?;
    if step_status!="WAITING_APPROVAL" { return Err(AppError::Conflict("Step is not waiting for approval".into())); }

    sqlx::query("UPDATE automation_run_steps SET status='COMPLETED',completed_at=NOW(),output=$1 WHERE id=$2")
        .bind(json!({"approved":true})).bind(step_id).execute(&state.pool).await?;

    let edges=sqlx::query("SELECT target_node_key,config FROM automation_edges WHERE automation_id=$1 AND source_node_key=$2")
        .bind(automation_id).bind(&key).fetch_all(&state.pool).await?;
    let next=edges.into_iter().find(|r|condition_matches(&r.try_get::<Value,_>("config").unwrap_or(json!({})),&vars))
        .map(|r|r.try_get::<String,_>("target_node_key").unwrap());

    let nodes=sqlx::query("SELECT node_key,node_type,config FROM automation_nodes WHERE automation_id=$1").bind(automation_id).fetch_all(&state.pool).await?;
    let map:HashMap<String,(String,Value)>=nodes.into_iter().map(|r|Ok::<_,sqlx::Error>((r.try_get("node_key")?,(r.try_get("node_type")?,r.try_get("config")?)))).collect::<Result<_,_>>()?;

    let mut current=next;
    let mut visited=HashSet::new();
    while let Some(k)=current {
        if !visited.insert(k.clone()) { return Err(AppError::BadRequest("Automation contains a cycle".into())); }
        let (typ,cfg)=map.get(&k).ok_or_else(||AppError::BadRequest("Broken workflow edge".into()))?.clone();
        let step=sqlx::query("INSERT INTO automation_run_steps(run_id,node_key,node_type,input) VALUES($1,$2,$3,$4) RETURNING id")
            .bind(run_id).bind(&k).bind(&typ).bind(&vars).fetch_one(&state.pool).await?;
        let sid:Uuid=step.try_get("id")?;
        match execute_node(state,&typ,&cfg,&vars).await {
            Ok(out)=>{
                vars=merge(vars,out.clone());
                sqlx::query("UPDATE automation_run_steps SET status='COMPLETED',output=$1,completed_at=NOW() WHERE id=$2").bind(out).bind(sid).execute(&state.pool).await?;
            }
            Err(e)=>{
                let msg=e.to_string();
                sqlx::query("UPDATE automation_run_steps SET status='FAILED',error=$1,completed_at=NOW() WHERE id=$2").bind(&msg).bind(sid).execute(&state.pool).await?;
                sqlx::query("UPDATE automation_runs SET status='FAILED',error=$1,variables=$2,completed_at=NOW() WHERE id=$3").bind(&msg).bind(&vars).bind(run_id).execute(&state.pool).await?;
                return Err(e);
            }
        }
        let next_edges=sqlx::query("SELECT target_node_key,config FROM automation_edges WHERE automation_id=$1 AND source_node_key=$2").bind(automation_id).bind(&k).fetch_all(&state.pool).await?;
        current=next_edges.into_iter().find(|r|condition_matches(&r.try_get::<Value,_>("config").unwrap_or(json!({})),&vars)).map(|r|r.try_get::<String,_>("target_node_key").unwrap());
    }
    sqlx::query("UPDATE automation_runs SET status='COMPLETED',variables=$1,completed_at=NOW() WHERE id=$2").bind(vars).bind(run_id).execute(&state.pool).await?;
    Ok(())
}

pub async fn scheduler_tick(state:&AppState)->Result<(),AppError>{
    let rows=sqlx::query("SELECT s.id,s.automation_id,a.workspace_id,s.interval_seconds FROM automation_schedules s JOIN automations a ON a.id=s.automation_id WHERE s.enabled AND a.status='PUBLISHED' AND s.next_run_at<=NOW() LIMIT 25")
        .fetch_all(&state.pool).await?;
    for r in rows {
        let sid:Uuid=r.try_get("id")?; let aid:Uuid=r.try_get("automation_id")?; let wid:Uuid=r.try_get("workspace_id")?; let secs:i64=r.try_get("interval_seconds")?;
        let _=trigger(state,aid,wid,"SCHEDULE",json!({"scheduled_at":Utc::now().to_rfc3339()})).await;
        sqlx::query("UPDATE automation_schedules SET next_run_at=NOW()+($1 * INTERVAL '1 second'),updated_at=NOW() WHERE id=$2").bind(secs).bind(sid).execute(&state.pool).await?;
    }
    Ok(())
}

pub async fn get_run(pool:&sqlx::PgPool,workspace_id:Uuid,run_id:Uuid)->Result<Value,AppError>{
    let run=sqlx::query("SELECT id,automation_id,status,trigger_type,trigger_payload,variables,error,started_at,completed_at FROM automation_runs WHERE id=$1 AND workspace_id=$2").bind(run_id).bind(workspace_id).fetch_optional(pool).await?.ok_or_else(||AppError::NotFound("Automation run not found".into()))?;
    let steps=sqlx::query("SELECT node_key,node_type,status,input,output,error,attempt,started_at,completed_at FROM automation_run_steps WHERE run_id=$1 ORDER BY started_at").bind(run_id).fetch_all(pool).await?;
    let arr:Vec<Value>=steps.into_iter().map(|r|json!({"id":r.try_get::<Uuid,_>("id").unwrap_or_default(),"node_key":r.try_get::<String,_>("node_key").unwrap_or_default(),"node_type":r.try_get::<String,_>("node_type").unwrap_or_default(),"status":r.try_get::<String,_>("status").unwrap_or_default(),"input":r.try_get::<Value,_>("input").unwrap_or(json!({})),"output":r.try_get::<Value,_>("output").unwrap_or(json!({})),"error":r.try_get::<Option<String>,_>("error").unwrap_or(None)})).collect();
    Ok(json!({"id":run.try_get::<Uuid,_>("id")?,"automation_id":run.try_get::<Uuid,_>("automation_id")?,"status":run.try_get::<String,_>("status")?,"trigger_type":run.try_get::<Option<String>,_>("trigger_type")?,"trigger_payload":run.try_get::<Value,_>("trigger_payload")?,"variables":run.try_get::<Value,_>("variables")?,"error":run.try_get::<Option<String>,_>("error")?,"steps":arr}))
}
