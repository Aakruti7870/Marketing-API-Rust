use crate::error::AppError;
use crate::models::{Automation, AutomationGraph, AutomationNode, AutomationRun, CreateAutomationDto, UpdateAutomationDto};
use reqwest::{Client, Method, Url};
use serde_json::{json, Map, Value};
use sqlx::PgPool;
use std::{collections::{HashMap, HashSet, VecDeque}, env, net::IpAddr, time::Duration};
use uuid::Uuid;

const MAX_RESPONSE_BYTES: usize = 1_048_576;

pub async fn list(pool:&PgPool,w:Uuid)->Result<Vec<Automation>,AppError>{
 Ok(sqlx::query_as::<_,Automation>("SELECT * FROM automations WHERE workspace_id=$1 ORDER BY updated_at DESC").bind(w).fetch_all(pool).await.map_err(AppError::Database)?)
}
pub async fn get(pool:&PgPool,w:Uuid,id:Uuid)->Result<Automation,AppError>{
 sqlx::query_as::<_,Automation>("SELECT * FROM automations WHERE workspace_id=$1 AND id=$2").bind(w).bind(id).fetch_optional(pool).await.map_err(AppError::Database)?.ok_or_else(||AppError::NotFound("Automation not found".into()))
}
pub async fn create(pool:&PgPool,w:Uuid,u:Uuid,dto:CreateAutomationDto)->Result<Automation,AppError>{
 validate_graph(&dto.graph)?; let t=dto.trigger_type.to_uppercase(); let cfg=dto.trigger_config.unwrap_or_else(||json!({}));
 let interval=if t=="SCHEDULE"{schedule_interval(&cfg)?}else{None}; let next=interval.map(|s|chrono::Utc::now()+chrono::Duration::seconds(s));
 let token=if t=="WEBHOOK"{Some(Uuid::new_v4().simple().to_string())}else{None};
 let graph=serde_json::to_value(dto.graph).map_err(|e|AppError::BadRequest(e.to_string()))?;
 Ok(sqlx::query_as::<_,Automation>("INSERT INTO automations(id,workspace_id,name,description,status,trigger_type,trigger_config,webhook_token,graph,created_by_id,schedule_interval_seconds,next_run_at) VALUES($1,$2,$3,$4,'DRAFT',$5,$6,$7,$8,$9,$10,$11) RETURNING *")
 .bind(Uuid::new_v4()).bind(w).bind(dto.name.trim()).bind(dto.description).bind(t).bind(cfg).bind(token).bind(graph).bind(u).bind(interval).bind(next).fetch_one(pool).await.map_err(AppError::Database)?)
}
pub async fn update(pool:&PgPool,w:Uuid,id:Uuid,dto:UpdateAutomationDto)->Result<Automation,AppError>{
 let old=get(pool,w,id).await?; if let Some(g)=&dto.graph{validate_graph(g)?}
 let t=dto.trigger_type.as_ref().map(|x|x.to_uppercase()); let cfg=dto.trigger_config.clone();
 let interval=if t.as_deref()==Some("SCHEDULE")||(t.is_none()&&old.trigger_type=="SCHEDULE"){schedule_interval(cfg.as_ref().unwrap_or(&old.trigger_config))?}else{None};
 let next=interval.map(|s|chrono::Utc::now()+chrono::Duration::seconds(s)); let status=if old.status=="ACTIVE"{"DRAFT"}else{&old.status};
 let graph=dto.graph.map(|g|serde_json::to_value(g)).transpose().map_err(|e|AppError::BadRequest(e.to_string()))?;
 Ok(sqlx::query_as::<_,Automation>("UPDATE automations SET name=COALESCE($3,name),description=COALESCE($4,description),trigger_type=COALESCE($5,trigger_type),trigger_config=COALESCE($6,trigger_config),graph=COALESCE($7,graph),status=$8,version=version+1,schedule_interval_seconds=$9,next_run_at=$10,updated_at=NOW() WHERE workspace_id=$1 AND id=$2 RETURNING *")
 .bind(w).bind(id).bind(dto.name.map(|x|x.trim().to_string())).bind(dto.description).bind(t).bind(cfg).bind(graph).bind(status).bind(interval).bind(next).fetch_one(pool).await.map_err(AppError::Database)?)
}
pub async fn set_status(pool:&PgPool,w:Uuid,id:Uuid,status:&str)->Result<Automation,AppError>{
 if !["DRAFT","ACTIVE","PAUSED"].contains(&status){return Err(AppError::BadRequest("Invalid automation status".into()))}
 let old=get(pool,w,id).await?; let next=if status=="ACTIVE"&&old.trigger_type=="SCHEDULE"{old.schedule_interval_seconds.map(|s|chrono::Utc::now()+chrono::Duration::seconds(s))}else{None};
 sqlx::query_as::<_,Automation>("UPDATE automations SET status=$3,next_run_at=$4,updated_at=NOW() WHERE workspace_id=$1 AND id=$2 RETURNING *").bind(w).bind(id).bind(status).bind(next).fetch_optional(pool).await.map_err(AppError::Database)?.ok_or_else(||AppError::NotFound("Automation not found".into()))
}
pub async fn run_steps(pool:&PgPool,w:Uuid,id:Uuid,run_id:Uuid)->Result<Vec<crate::models::AutomationRunStep>,AppError>{
 let owns=sqlx::query_scalar::<_,Uuid>("SELECT id FROM automation_runs WHERE id=$1 AND automation_id=$2 AND workspace_id=$3").bind(run_id).bind(id).bind(w).fetch_optional(pool).await.map_err(AppError::Database)?;
 if owns.is_none(){return Err(AppError::NotFound("Automation run not found".into()))}
 Ok(sqlx::query_as::<_,crate::models::AutomationRunStep>("SELECT * FROM automation_run_steps WHERE run_id=$1 ORDER BY started_at ASC").bind(run_id).fetch_all(pool).await.map_err(AppError::Database)?)
}
pub async fn runs(pool:&PgPool,w:Uuid,id:Uuid)->Result<Vec<AutomationRun>,AppError>{
 Ok(sqlx::query_as::<_,AutomationRun>("SELECT * FROM automation_runs WHERE workspace_id=$1 AND automation_id=$2 ORDER BY started_at DESC LIMIT 100").bind(w).bind(id).fetch_all(pool).await.map_err(AppError::Database)?)
}
pub async fn run(pool:&PgPool,http:&Client,w:Uuid,id:Uuid,payload:Value)->Result<AutomationRun,AppError>{
 let a=get(pool,w,id).await?; if a.status=="PAUSED"{return Err(AppError::BadRequest("Automation is paused".into()))}
 let rid=Uuid::new_v4(); sqlx::query("INSERT INTO automation_runs(id,automation_id,workspace_id,status,trigger_payload) VALUES($1,$2,$3,'RUNNING',$4)").bind(rid).bind(id).bind(w).bind(payload.clone()).execute(pool).await.map_err(AppError::Database)?;
 let graph:AutomationGraph=serde_json::from_value(a.graph.clone()).map_err(|e|AppError::BadRequest(format!("Invalid workflow graph: {e}")))?;
 match execute_graph(pool,http,rid,&graph,payload).await{
  Ok(v)=>{sqlx::query("UPDATE automation_runs SET status='COMPLETED',output_data=$2,completed_at=NOW() WHERE id=$1").bind(rid).bind(v).execute(pool).await.map_err(AppError::Database)?;},
  Err(e)=>{sqlx::query("UPDATE automation_runs SET status='FAILED',error_message=$2,completed_at=NOW() WHERE id=$1").bind(rid).bind(e.to_string()).execute(pool).await.map_err(AppError::Database)?;}
 }
 Ok(sqlx::query_as::<_,AutomationRun>("SELECT * FROM automation_runs WHERE id=$1").bind(rid).fetch_one(pool).await.map_err(AppError::Database)?)
}
pub async fn run_due_schedules(pool:&PgPool,http:&Client)->Result<usize,AppError>{
 let rows=sqlx::query_as::<_,Automation>("UPDATE automations SET next_run_at=NOW()+make_interval(secs=>schedule_interval_seconds),last_run_at=NOW() WHERE id IN(SELECT id FROM automations WHERE status='ACTIVE' AND trigger_type='SCHEDULE' AND next_run_at<=NOW() AND schedule_interval_seconds>0 ORDER BY next_run_at FOR UPDATE SKIP LOCKED LIMIT 20) RETURNING *").fetch_all(pool).await.map_err(AppError::Database)?;
 let n=rows.len(); for a in rows{if let Err(e)=run(pool,http,a.workspace_id,a.id,json!({"source":"schedule","scheduled_at":chrono::Utc::now()})).await{tracing::error!(automation_id=%a.id,error=%e,"scheduled automation failed")}} Ok(n)
}

struct NodeResult{data:Value,branch:Option<String>}
async fn execute_graph(pool:&PgPool,http:&Client,rid:Uuid,g:&AutomationGraph,payload:Value)->Result<Value,AppError>{
 validate_graph(g)?; let mut nodes=HashMap::new(); let mut incoming=HashSet::new(); let mut out:HashMap<String,Vec<(&str,Option<&str>)>>=HashMap::new();
 for n in &g.nodes{nodes.insert(n.id.clone(),n);} for e in &g.edges{incoming.insert(e.target.clone());out.entry(e.source.clone()).or_default().push((e.target.as_str(),e.branch.as_deref()))}
 let starts=g.nodes.iter().filter(|n|!incoming.contains(&n.id)).map(|n|n.id.clone()).collect::<Vec<_>>(); if starts.is_empty(){return Err(AppError::BadRequest("Automation graph has no start node".into()))}
 let mut q=VecDeque::new(); for s in starts{q.push_back((s,payload.clone()))} let mut last=payload; let mut count=0;
 while let Some((id,input))=q.pop_front(){count+=1;if count>1000{return Err(AppError::BadRequest("Automation execution limit exceeded".into()))}
  let n=*nodes.get(&id).ok_or_else(||AppError::BadRequest(format!("Unknown node {id}")))?; let sid=Uuid::new_v4();
  sqlx::query("INSERT INTO automation_run_steps(id,run_id,node_id,node_type,status,input_data) VALUES($1,$2,$3,$4,'RUNNING',$5)").bind(sid).bind(rid).bind(&n.id).bind(&n.node_type).bind(&input).execute(pool).await.map_err(AppError::Database)?;
  match execute_node(http,n,&input).await{
   Ok(r)=>{last=r.data.clone();sqlx::query("UPDATE automation_run_steps SET status='COMPLETED',output_data=$2,completed_at=NOW() WHERE id=$1").bind(sid).bind(&r.data).execute(pool).await.map_err(AppError::Database)?;
    if let Some(edges)=out.get(&id){for(t,b)in edges{if b.map(|x|r.branch.as_deref()==Some(x)).unwrap_or(r.branch.is_none()){q.push_back(((*t).into(),r.data.clone()))}}}},
   Err(e)=>{sqlx::query("UPDATE automation_run_steps SET status='FAILED',error_message=$2,completed_at=NOW() WHERE id=$1").bind(sid).bind(e.to_string()).execute(pool).await.map_err(AppError::Database)?;return Err(e)}
  }
 } Ok(last)
}
async fn execute_node(http:&Client,n:&AutomationNode,input:&Value)->Result<NodeResult,AppError>{
 let c=render(&n.config,input)?;
 match n.node_type.as_str(){
  "trigger.manual"|"trigger.webhook"|"trigger.schedule"=>Ok(NodeResult{data:input.clone(),branch:None}),
  "data.set"=>{let mut v=input.clone();merge(&mut v,c.get("data").cloned().unwrap_or_else(||json!({})));Ok(NodeResult{data:v,branch:None})},
  "logic.condition"|"logic.if"|"n8n.if"|"logic.filter"=>{let ok=lookup(input,c.get("path").and_then(Value::as_str).unwrap_or(""))==c.get("equals").unwrap_or(&Value::Null);Ok(NodeResult{data:input.clone(),branch:Some(if ok{"true".into()}else{"false".into()})})},
  "logic.switch"|"n8n.switch"=>{let a=lookup(input,c.get("path").and_then(Value::as_str).unwrap_or(""));let mut b=c.get("default_branch").and_then(Value::as_str).unwrap_or("default").to_string();if let Some(rs)=c.get("rules").and_then(Value::as_array){for r in rs{if r.get("equals")==Some(a){if let Some(x)=r.get("branch").and_then(Value::as_str){b=x.into();break}}}}Ok(NodeResult{data:input.clone(),branch:Some(b)})},
  "transform.split"=>{let a=lookup(input,c.get("path").and_then(Value::as_str).unwrap_or("")).as_array().cloned().unwrap_or_default();Ok(NodeResult{data:Value::Array(a),branch:None})},
  "transform.aggregate"=>{let a=lookup(input,c.get("path").and_then(Value::as_str).unwrap_or("")).as_array().cloned().unwrap_or_default();Ok(NodeResult{data:json!({"count":a.len(),"items":a}),branch:None})},
  "transform.merge"|"logic.merge"|"data.noop"=>Ok(NodeResult{data:input.clone(),branch:None}),
  "http.request"|"action.http"=>http_node(http,&c,input).await,
  "ai.generate"|"ai.agent"=>ai_node(http,&c,input).await,
  "delay.wait"=>{let ms=c.get("milliseconds").and_then(Value::as_u64).unwrap_or(1000).min(30000);tokio::time::sleep(Duration::from_millis(ms)).await;Ok(NodeResult{data:input.clone(),branch:None})},
  x=>Err(AppError::BadRequest(format!("Unsupported automation node type: {x}")))
 }
}
async fn http_node(http:&Client,c:&Value,input:&Value)->Result<NodeResult,AppError>{
 let raw=c.get("url").and_then(Value::as_str).ok_or_else(||AppError::BadRequest("HTTP node requires url".into()))?;validate_url(raw)?;
 let method=c.get("method").and_then(Value::as_str).unwrap_or("GET").parse::<Method>().map_err(|_|AppError::BadRequest("Invalid HTTP method".into()))?;let timeout=c.get("timeout_seconds").and_then(Value::as_u64).unwrap_or(30).clamp(1,60);
 let mut req=http.request(method.clone(),raw).timeout(Duration::from_secs(timeout));if let Some(h)=c.get("headers").and_then(Value::as_object){for(k,v)in h{if let Some(s)=v.as_str(){req=req.header(k,s)}}}
 let body=c.get("body").cloned().unwrap_or_else(||input.clone());let resp=if method==Method::GET||method==Method::HEAD||method==Method::DELETE{req.send().await}else{req.json(&body).send().await}.map_err(|e|AppError::ExternalService(e.to_string()))?;
 let status=resp.status().as_u16();if resp.content_length().unwrap_or(0)>MAX_RESPONSE_BYTES as u64{return Err(AppError::ExternalService("Response exceeds 1 MB".into()))}let bytes=resp.bytes().await.map_err(|e|AppError::ExternalService(e.to_string()))?;if bytes.len()>MAX_RESPONSE_BYTES{return Err(AppError::ExternalService("Response exceeds 1 MB".into()))}
 let v=serde_json::from_slice::<Value>(&bytes).unwrap_or_else(|_|json!({"body":String::from_utf8_lossy(&bytes)}));Ok(NodeResult{data:json!({"status":status,"response":v}),branch:None})
}
async fn ai_node(http:&Client,c:&Value,input:&Value)->Result<NodeResult,AppError>{
 let endpoint=c.get("endpoint").and_then(Value::as_str).ok_or_else(||AppError::BadRequest("AI node requires endpoint".into()))?;validate_url(endpoint)?;let provider=c.get("provider").and_then(Value::as_str).unwrap_or("openai-compatible").to_lowercase();let prompt=c.get("prompt").and_then(Value::as_str).unwrap_or("Process this input");let model=c.get("model").and_then(Value::as_str).unwrap_or("default");let key=secret(c.get("api_key_env"))?;
 let mut req=http.post(endpoint).timeout(Duration::from_secs(120)).header("content-type","application/json");if let Some(k)=key{req=req.bearer_auth(k)}
 let body=if provider=="gemini"{json!({"contents":[{"parts":[{"text":format!("{}\nINPUT:\n{}",prompt,input)}]}]})}else{json!({"model":model,"messages":[{"role":"user","content":format!("{}\nINPUT:\n{}",prompt,input)}]})};
 let resp=req.json(&body).send().await.map_err(|e|AppError::ExternalService(e.to_string()))?;if resp.content_length().unwrap_or(0)>MAX_RESPONSE_BYTES as u64{return Err(AppError::ExternalService("AI response exceeds 1 MB".into()))}let b=resp.bytes().await.map_err(|e|AppError::ExternalService(e.to_string()))?;Ok(NodeResult{data:serde_json::from_slice(&b).map_err(|_|AppError::ExternalService("AI returned invalid JSON".into()))?,branch:None})
}
fn validate_url(raw:&str)->Result<(),AppError>{let u=Url::parse(raw).map_err(|_|AppError::BadRequest("Invalid URL".into()))?;if u.scheme()!="https"&&env::var("AUTOMATION_ALLOW_HTTP").ok().as_deref()!=Some("true"){return Err(AppError::BadRequest("Automation HTTP requires HTTPS".into()))}let h=u.host_str().unwrap_or("").to_lowercase();if h=="localhost"||h.ends_with(".local")||h=="0.0.0.0"||h=="127.0.0.1"||h=="::1"{return Err(AppError::BadRequest("Private/local host blocked".into()))}if let Ok(ip)=h.parse::<IpAddr>(){if ip.is_private()||ip.is_loopback(){return Err(AppError::BadRequest("Private IP blocked".into()))}}Ok(())}
fn schedule_interval(c:&Value)->Result<Option<i64>,AppError>{let x=c.get("interval_seconds").and_then(Value::as_i64).or_else(||c.get("every_seconds").and_then(Value::as_i64));if let Some(v)=x{if !(1..=31536000).contains(&v){return Err(AppError::BadRequest("Schedule interval must be 1 second to 1 year".into()))}return Ok(Some(v))}Ok(None)}
fn lookup<'a>(v:&'a Value,p:&str)->&'a Value{p.split('.').filter(|x|!x.is_empty()).fold(v,|v,k|v.get(k).unwrap_or(&Value::Null))}
fn merge(v:&mut Value,e:Value){if let(Some(a),Some(b))=(v.as_object_mut(),e.as_object()){for(k,x)in b{a.insert(k.clone(),x.clone());}}else{*v=e}}
fn render(v:&Value,input:&Value)->Result<Value,AppError>{match v{Value::String(s)=>Ok(Value::String(render_string(s,input))),Value::Array(a)=>Ok(Value::Array(a.iter().map(|x|render(x,input)).collect::<Result<Vec<_>,_>>()?)),Value::Object(o)=>{let mut m=Map::new();for(k,x)in o{m.insert(k.clone(),render(x,input)?);}Ok(Value::Object(m))},_=>Ok(v.clone())}}
fn render_string(s:&str,input:&Value)->String{let mut out=s.to_string();let mut pos=0;while let Some(i)=out[pos..].find("{{$json."){let start=pos+i;if let Some(e)=out[start..].find("}}"){let end=start+e;let token=&out[start..end+2];let path=&out[start+8..end];out=out.replacen(token,&lookup(input,path).to_string(),1);pos=start+1}else{break}}out}
fn secret(v:Option<&Value>)->Result<Option<String>,AppError>{match v{None=>Ok(None),Some(Value::String(n))=>Ok(Some(env::var(n).map_err(|_|AppError::BadRequest(format!("Missing environment secret: {n}")))?)),Some(Value::Object(o))=>{let n=o.get("$env").and_then(Value::as_str).ok_or_else(||AppError::BadRequest("Invalid secret reference".into()))?;Ok(Some(env::var(n).map_err(|_|AppError::BadRequest(format!("Missing environment secret: {n}")))?))},_=>Err(AppError::BadRequest("Invalid secret reference".into()))}}
pub fn validate_graph(g:&AutomationGraph)->Result<(),AppError>{
 if g.nodes.is_empty(){return Err(AppError::BadRequest("Automation needs at least one node".into()))}let ids:HashSet<_>=g.nodes.iter().map(|n|n.id.as_str()).collect();if ids.len()!=g.nodes.len(){return Err(AppError::BadRequest("Automation node IDs must be unique".into()))}
 for n in &g.nodes{if !matches!(n.node_type.as_str(),"trigger.manual"|"trigger.webhook"|"trigger.schedule"|"data.set"|"logic.condition"|"logic.if"|"logic.switch"|"logic.filter"|"logic.merge"|"transform.merge"|"transform.split"|"transform.aggregate"|"http.request"|"action.http"|"ai.generate"|"ai.agent"|"delay.wait"|"data.noop"){return Err(AppError::BadRequest(format!("Unsupported automation node type: {}",n.node_type)))}}for e in &g.edges{if !ids.contains(e.source.as_str())||!ids.contains(e.target.as_str()){return Err(AppError::BadRequest("Automation edge references unknown node".into()))}if e.source==e.target{return Err(AppError::BadRequest("Automation cannot self-loop".into()))}}Ok(())
}