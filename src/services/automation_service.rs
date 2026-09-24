use crate::error::AppError;
use crate::models::{Automation, AutomationGraph, AutomationNode, AutomationRun, CreateAutomationDto, UpdateAutomationDto};
use reqwest::{Method, Url};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::{collections::{HashMap, HashSet, VecDeque}, net::IpAddr, time::Duration};
use uuid::Uuid;

const MAX_BODY_BYTES: usize = 1_048_576;
const MAX_NODES: usize = 100;
const MAX_RUN_STEPS: usize = 500;

pub async fn list(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<Automation>, AppError> {
    sqlx::query_as::<_, Automation>("SELECT * FROM automations WHERE workspace_id=$1 ORDER BY updated_at DESC")
        .bind(workspace_id).fetch_all(pool).await.map_err(AppError::Database)
}

pub async fn get(pool:&PgPool, workspace_id:Uuid, id:Uuid)->Result<Automation,AppError>{
    sqlx::query_as::<_,Automation>("SELECT * FROM automations WHERE workspace_id=$1 AND id=$2")
        .bind(workspace_id).bind(id).fetch_optional(pool).await.map_err(AppError::Database)?
        .ok_or_else(||AppError::NotFound("Automation not found".into()))
}

pub async fn create(pool:&PgPool,workspace_id:Uuid,user_id:Uuid,dto:CreateAutomationDto)->Result<Automation,AppError>{
    validate_graph(&dto.graph)?;
    let webhook_token=if dto.trigger_type.eq_ignore_ascii_case("WEBHOOK"){Some(Uuid::new_v4().simple().to_string())}else{None};
    let graph=serde_json::to_value(dto.graph).map_err(|e|AppError::BadRequest(e.to_string()))?;
    sqlx::query_as::<_,Automation>("INSERT INTO automations
      (id,workspace_id,name,description,status,trigger_type,trigger_config,webhook_token,graph,created_by_id)
      VALUES ($1,$2,$3,$4,'DRAFT',$5,$6,$7,$8,$9) RETURNING *")
      .bind(Uuid::new_v4()).bind(workspace_id).bind(dto.name.trim()).bind(dto.description)
      .bind(dto.trigger_type.to_uppercase()).bind(dto.trigger_config.unwrap_or_else(||json!({})))
      .bind(webhook_token).bind(graph).bind(user_id).fetch_one(pool).await.map_err(AppError::Database)
}

pub async fn update(pool:&PgPool,workspace_id:Uuid,id:Uuid,dto:UpdateAutomationDto)->Result<Automation,AppError>{
    let current=get(pool,workspace_id,id).await?;
    let graph=match dto.graph{Some(g)=>{validate_graph(&g)?;Some(serde_json::to_value(g).map_err(|e|AppError::BadRequest(e.to_string()))?)}None=>None};
    let status=if current.status=="ACTIVE"{"DRAFT"}else{&current.status};
    sqlx::query_as::<_,Automation>("UPDATE automations SET name=COALESCE($3,name),description=COALESCE($4,description),
      trigger_type=COALESCE($5,trigger_type),trigger_config=COALESCE($6,trigger_config),graph=COALESCE($7,graph),
      status=$8,version=version+1,updated_at=NOW() WHERE workspace_id=$1 AND id=$2 RETURNING *")
      .bind(workspace_id).bind(id).bind(dto.name.map(|s|s.trim().to_string())).bind(dto.description)
      .bind(dto.trigger_type.map(|s|s.to_uppercase())).bind(dto.trigger_config).bind(graph).bind(status)
      .fetch_one(pool).await.map_err(AppError::Database)
}

pub async fn set_status(pool:&PgPool,workspace_id:Uuid,id:Uuid,status:&str)->Result<Automation,AppError>{
    if !matches!(status,"DRAFT"|"ACTIVE"|"PAUSED"){return Err(AppError::BadRequest("Invalid automation status".into()));}
    sqlx::query_as::<_,Automation>("UPDATE automations SET status=$3,updated_at=NOW() WHERE workspace_id=$1 AND id=$2 RETURNING *")
      .bind(workspace_id).bind(id).bind(status).fetch_optional(pool).await.map_err(AppError::Database)?
      .ok_or_else(||AppError::NotFound("Automation not found".into()))
}

pub async fn runs(pool:&PgPool,workspace_id:Uuid,id:Uuid)->Result<Vec<AutomationRun>,AppError>{
    sqlx::query_as::<_,AutomationRun>("SELECT * FROM automation_runs WHERE workspace_id=$1 AND automation_id=$2 ORDER BY started_at DESC LIMIT 100")
      .bind(workspace_id).bind(id).fetch_all(pool).await.map_err(AppError::Database)
}

pub async fn run(pool:&PgPool,http:&reqwest::Client,workspace_id:Uuid,id:Uuid,payload:Value)->Result<AutomationRun,AppError>{
    let automation=get(pool,workspace_id,id).await?;
    if automation.status=="PAUSED"{return Err(AppError::BadRequest("Automation is paused".into()));}
    let run_id=Uuid::new_v4();
    let mut execution=sqlx::query_as::<_,AutomationRun>("INSERT INTO automation_runs
      (id,automation_id,workspace_id,status,trigger_payload) VALUES($1,$2,$3,'RUNNING',$4) RETURNING *")
      .bind(run_id).bind(id).bind(workspace_id).bind(&payload).fetch_one(pool).await.map_err(AppError::Database)?;
    let graph:AutomationGraph=serde_json::from_value(automation.graph.clone()).map_err(|e|AppError::BadRequest(format!("Invalid workflow graph: {e}")))?;
    match execute_graph(pool,http,run_id,&graph,payload).await{
      Ok(output)=>execution=sqlx::query_as::<_,AutomationRun>("UPDATE automation_runs SET status='COMPLETED',output_data=$2,completed_at=NOW() WHERE id=$1 RETURNING *").bind(run_id).bind(output).fetch_one(pool).await.map_err(AppError::Database)?,
      Err(e)=>execution=sqlx::query_as::<_,AutomationRun>("UPDATE automation_runs SET status='FAILED',error_message=$2,completed_at=NOW() WHERE id=$1 RETURNING *").bind(run_id).bind(e.to_string()).fetch_one(pool).await.map_err(AppError::Database)?
    }
    Ok(execution)
}

pub async fn run_due_schedules(pool:&PgPool,http:&reqwest::Client)->Result<u64,AppError>{
    let rows=sqlx::query_as::<_,Automation>("SELECT * FROM automations WHERE status='ACTIVE' AND trigger_type='SCHEDULE' AND
      (next_run_at IS NULL OR next_run_at<=NOW()) ORDER BY next_run_at NULLS FIRST LIMIT 25")
      .fetch_all(pool).await.map_err(AppError::Database)?;
    let mut count=0;
    for a in rows{
      let seconds=a.trigger_config.get("interval_seconds").and_then(Value::as_u64).unwrap_or(3600).clamp(60,86400);
      let result=run(pool,http,a.workspace_id,a.id,a.trigger_config.get("payload").cloned().unwrap_or_else(||json!({}))).await;
      sqlx::query("UPDATE automations SET next_run_at=NOW()+make_interval(secs => $2::double precision),updated_at=NOW() WHERE id=$1")
        .bind(a.id).bind(seconds as f64).execute(pool).await.map_err(AppError::Database)?;
      if result.is_ok(){count+=1;}
    }
    Ok(count)
}

async fn execute_graph(pool:&PgPool,http:&reqwest::Client,run_id:Uuid,graph:&AutomationGraph,payload:Value)->Result<Value,AppError>{
    let nodes:HashMap<String,&AutomationNode>=graph.nodes.iter().map(|n|(n.id.clone(),n)).collect();
    let mut next:HashMap<String,Vec<(&String,Option<&String>)>>=HashMap::new();
    for e in &graph.edges{next.entry(e.source.clone()).or_default().push((&e.target,e.branch.as_ref()));}
    let roots=graph.nodes.iter().filter(|n|!graph.edges.iter().any(|e|e.target==n.id)).map(|n|n.id.clone()).collect::<Vec<_>>();
    if roots.len()!=1{return Err(AppError::BadRequest("Workflow must have exactly one trigger/root node".into()));}
    let mut queue:VecDeque<(String,Value)>=VecDeque::from([(roots[0].clone(),payload)]);
    let mut executions=0usize;
    let mut last=Value::Null;
    while let Some((id,input))=queue.pop_front(){
      executions+=1;if executions>MAX_RUN_STEPS{return Err(AppError::BadRequest("Workflow execution step limit exceeded".into()));}
      let node=nodes.get(&id).ok_or_else(||AppError::BadRequest(format!("Unknown node {id}")))?;
      let step_id=Uuid::new_v4();
      sqlx::query("INSERT INTO automation_run_steps(id,run_id,node_id,node_type,status,input_data) VALUES($1,$2,$3,$4,'RUNNING',$5)")
        .bind(step_id).bind(run_id).bind(&node.id).bind(&node.node_type).bind(&input).execute(pool).await.map_err(AppError::Database)?;
      let result=execute_node(http,node,&input).await;
      match result{
        Ok(out)=>{
          sqlx::query("UPDATE automation_run_steps SET status='COMPLETED',output_data=$2,completed_at=NOW() WHERE id=$1").bind(step_id).bind(&out).execute(pool).await.map_err(AppError::Database)?;
          last=out.clone();
          let branch=out.get("_branch").and_then(Value::as_str);
          if let Some(children)=next.get(&id){for (child,edge_branch) in children{
            if edge_branch.is_none() || edge_branch.and_then(|b|branch).is_some_and(|b|b==edge_branch.unwrap().as_str()){queue.push_back(((*child).clone(),out.clone()));}
          }}
        }
        Err(e)=>{
          sqlx::query("UPDATE automation_run_steps SET status='FAILED',error_message=$2,completed_at=NOW() WHERE id=$1").bind(step_id).bind(e.to_string()).execute(pool).await.map_err(AppError::Database)?;
          return Err(e)
        }
      }
    }
    Ok(last)
}

async fn execute_node(http:&reqwest::Client,node:&AutomationNode,input:&Value)->Result<Value,AppError>{
  match node.node_type.as_str(){
    "trigger.manual"|"trigger.webhook"|"trigger.schedule"=>Ok(input.clone()),
    "data.set"=>Ok(merge_set(input,node.config.get("data").unwrap_or(&json!({})))),
    "logic.condition"=>{
      let actual=get_path(input,node.config.get("path").and_then(Value::as_str).unwrap_or(""));
      let expected=node.config.get("equals").cloned().unwrap_or(Value::Null);
      let op=node.config.get("operator").and_then(Value::as_str).unwrap_or("equals");
      let matched=compare_values(actual,expected,op);
      Ok(json!({"_branch":if matched{"true"}else{"false"},"matched":matched,"input":input}))
    },
    "action.http"|"http.request"=>execute_http(http,node,input).await,
    "delay.wait"=>{let ms=node.config.get("milliseconds").and_then(Value::as_u64).unwrap_or(1000).min(30000);tokio::time::sleep(Duration::from_millis(ms)).await;Ok(input.clone())},
    "data.transform"=>Ok(apply_transform(input,node.config.get("operations").unwrap_or(&json!([])))),
    "logic.switch"=>execute_switch(node,input),
    "action.log"=>{tracing::info!(node=%node.name,payload=%input,"automation log node");Ok(input.clone())},
    _=>Err(AppError::BadRequest(format!("Unsupported automation node type: {}",node.node_type)))
  }
}

async fn execute_http(http:&reqwest::Client,node:&AutomationNode,input:&Value)->Result<Value,AppError>{
  let method=node.config.get("method").and_then(Value::as_str).unwrap_or("GET").parse::<Method>().map_err(|_|AppError::BadRequest("Invalid HTTP method".into()))?;
  let raw=node.config.get("url").and_then(Value::as_str).ok_or_else(||AppError::BadRequest("HTTP node requires url".into()))?;
  let url=render_template(raw,input)?;
  validate_url(&url)?;
  let mut req=http.request(method.clone(),Url::parse(&url).map_err(|_|AppError::BadRequest("Invalid URL".into()))?);
  if let Some(headers)=node.config.get("headers").and_then(Value::as_object){for(k,v)in headers{let value=render_template(v.as_str().unwrap_or(""),input)?;req=req.header(k,value);}}
  if method!=Method::GET && method!=Method::HEAD{
    let body=node.config.get("body").cloned().unwrap_or_else(||input.clone());
    req=req.json(&render_value(body,input)?);
  }
  let response=req.send().await.map_err(|e|AppError::BadRequest(format!("HTTP request failed: {e}")))?;
  let status=response.status().as_u16();
  let bytes=response.bytes().await.map_err(|e|AppError::BadRequest(format!("HTTP response failed: {e}")))?;
  if bytes.len()>MAX_BODY_BYTES{return Err(AppError::BadRequest("HTTP response exceeds 1 MB limit".into()));}
  let parsed=serde_json::from_slice::<Value>(&bytes).unwrap_or_else(|_|json!({"status":status,"body":String::from_utf8_lossy(&bytes)}));
  if status>=400{return Err(AppError::BadRequest(format!("HTTP node returned status {status}")));} 
  Ok(json!({"status":status,"response":parsed}))
}

fn validate_url(raw:&str)->Result<(),AppError>{
  let u=Url::parse(raw).map_err(|_|AppError::BadRequest("Invalid HTTP URL".into()))?;
  if u.scheme()!="http"&&u.scheme()!="https"{return Err(AppError::BadRequest("Only HTTP/HTTPS URLs are allowed".into()));}
  let host=u.host_str().ok_or_else(||AppError::BadRequest("HTTP URL has no host".into()))?;
  let blocked=matches!(host,"localhost"|"localhost.localdomain")||host.ends_with(".localhost")||host.ends_with(".internal");
  if blocked{return Err(AppError::BadRequest("Private/local HTTP targets are blocked".into()));}
  if let Ok(ip)=host.parse::<IpAddr>(){if is_private_ip(ip){return Err(AppError::BadRequest("Private HTTP targets are blocked".into()));}}
  Ok(())
}
fn is_private_ip(ip:IpAddr)->bool{match ip{IpAddr::V4(v)=>v.is_private()||v.is_loopback()||v.is_link_local()||v.is_unspecified()||v.octets()[0]==100&&v.octets()[1]>=64&&v.octets()[1]<=127,IpAddr::V6(v)=>v.is_loopback()||v.is_unspecified()||v.segments()[0]&0xfe00==0xfc00}}
fn get_path<'a>(v:&'a Value,path:&str)->&'a Value{path.split('.').filter(|s|!s.is_empty()).fold(v,|cur,key|cur.get(key).unwrap_or(&Value::Null))}
fn render_template(s:&str,input:&Value)->Result<String,AppError>{let mut out=String::with_capacity(s.len());let mut rest=s;while let Some(start)=rest.find("{{"){out.push_str(&rest[..start]);let tail=&rest[start+2..];let end=tail.find("}}").ok_or_else(||AppError::BadRequest("Unclosed template expression".into()))?;out.push_str(get_path(input,tail[..end].trim()).as_str().unwrap_or(&get_path(input,tail[..end].trim()).to_string()));rest=&tail[end+2..];}out.push_str(rest);Ok(out)}
fn render_value(v:Value,input:&Value)->Result<Value,AppError>{match v{Value::String(s)=>Ok(Value::String(render_template(&s,input)?)),Value::Array(a)=>Ok(Value::Array(a.into_iter().map(|x|render_value(x,input)).collect::<Result<_,_>>()?)),Value::Object(o)=>Ok(Value::Object(o.into_iter().map(|(k,v)|Ok((k,render_value(v,input)?))).collect::<Result<_,AppError>>()?)),x=>Ok(x)}}
fn merge_set(input:&Value,set:&Value)->Value{if let(Some(base),Some(add))=(input.as_object(),set.as_object()){let mut m=base.clone();for(k,v)in add{m.insert(k.clone(),v.clone());}Value::Object(m)}else{set.clone()}}
fn compare_values(a:&Value,b:Value,op:&str)->bool{match op{"not_equals"=>a!=&b,"contains"=>a.as_str().is_some_and(|s|b.as_str().is_some_and(|x|s.contains(x))),"greater_than"=>a.as_f64().zip(b.as_f64()).is_some_and(|(x,y)|x>y),"less_than"=>a.as_f64().zip(b.as_f64()).is_some_and(|(x,y)|x<y),_=>a==&b}}
fn apply_transform(input:&Value,ops:&Value)->Value{let mut out=input.clone();if let Some(arr)=ops.as_array(){for op in arr{if let(Some(target),Some(path))=(op.get("set"),op.get("path").and_then(Value::as_str)){if let Some(obj)=out.as_object_mut(){obj.insert(path.to_string(),target.clone());}}}}out}
fn execute_switch(node:&AutomationNode,input:&Value)->Result<Value,AppError>{let value=get_path(input,node.config.get("path").and_then(Value::as_str).unwrap_or(""));let branch=value.to_string().trim_matches('"').to_string();Ok(json!({"_branch":branch,"value":value,"input":input}))}

fn validate_graph(graph:&AutomationGraph)->Result<(),AppError>{
  if graph.nodes.is_empty()||graph.nodes.len()>MAX_NODES{return Err(AppError::BadRequest(format!("Workflow must contain 1-{MAX_NODES} nodes")));}
  let ids:HashSet<&str>=graph.nodes.iter().map(|n|n.id.as_str()).collect();
  if ids.len()!=graph.nodes.len(){return Err(AppError::BadRequest("Automation node IDs must be unique".into()));}
  let mut indegree=HashMap::<&str,usize>::new();
  let mut adjacency=HashMap::<&str,Vec<&str>>::new();
  for e in &graph.edges{if !ids.contains(e.source.as_str())||!ids.contains(e.target.as_str()){return Err(AppError::BadRequest("Automation edge references an unknown node".into()));}if e.source==e.target{return Err(AppError::BadRequest("Automation cannot contain self-loops".into()));}*indegree.entry(e.target.as_str()).or_default()+=1;adjacency.entry(e.source.as_str()).or_default().push(e.target.as_str());}
  let mut q=graph.nodes.iter().filter(|n|!indegree.contains_key(n.id.as_str())).map(|n|n.id.as_str()).collect::<VecDeque<_>>();
  let mut seen=0;while let Some(n)=q.pop_front(){seen+=1;if let Some(children)=adjacency.get(n){for c in children{let d=indegree.get_mut(c).unwrap();*d-=1;if *d==0{q.push_back(c);}}}}
  if seen!=graph.nodes.len(){return Err(AppError::BadRequest("Automation graph contains a cycle".into()));}
  let roots=graph.nodes.iter().filter(|n|!graph.edges.iter().any(|e|e.target==n.id)).count();
  if roots!=1{return Err(AppError::BadRequest("Automation must have exactly one root trigger".into()));}
  Ok(())
}

#[cfg(test)]
mod tests{
  use super::*;
  #[test]fn blocks_private_targets(){assert!(validate_url("http://127.0.0.1:4000").is_err());assert!(validate_url("http://10.0.0.1").is_err());}
  #[test]fn compares(){assert!(compare_values(&json!(10),json!(5),"greater_than"));assert!(!compare_values(&json!("a"),json!("b"),"equals"));}
  #[test]fn detects_cycles(){let g=AutomationGraph{nodes:vec![AutomationNode{id:"a".into(),name:"A".into(),node_type:"trigger.manual".into(),config:json!({}),position:[0.0,0.0]},AutomationNode{id:"b".into(),name:"B".into(),node_type:"action.log".into(),config:json!({}),position:[0.0,0.0]}],edges:vec![crate::models::AutomationEdge{source:"a".into(),target:"b".into(),branch:None},crate::models::AutomationEdge{source:"b".into(),target:"a".into(),branch:None}]};assert!(validate_graph(&g).is_err());}
}