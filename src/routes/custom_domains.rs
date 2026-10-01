use crate::error::AppError;
use crate::middleware::{require_workspace_roles, TenantContext};
use crate::state::AppState;
use axum::{
    extract::{Extension, Host, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

const DNS_TARGET: &str = "customers.goldetech.com";

#[derive(Debug, Deserialize)]
struct ConnectDomainDto {
    agent_id: Uuid,
    hostname: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
struct CustomDomain {
    id: Uuid,
    workspace_id: Uuid,
    agent_id: Uuid,
    hostname: String,
    status: String,
    dns_target: String,
    cloudflare_hostname_id: Option<String>,
    last_dns_status: Option<String>,
    last_ssl_status: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(connect_domain))
        .route("/agent/:agent_id", get(list_agent_domains))
        .route("/:id/verify", post(verify_domain))
        .with_state(state)
}

async fn connect_domain(
    State(state): State<AppState>,
    tenant: TenantContext,
    Json(dto): Json<ConnectDomainDto>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_roles(&tenant.workspace_role, &["OWNER", "ADMIN"])?;

    let hostname = normalize_hostname(&dto.hostname)?;
    let agent = sqlx::query_as::<_, crate::models::AiAgent>(
        "SELECT * FROM ai_agents WHERE id=$1 AND workspace_id=$2",
    )
    .bind(dto.agent_id)
    .bind(tenant.workspace_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("AI agent not found".into()))?;

    if agent.status != "ACTIVE" {
        return Err(AppError::BadRequest(
            "Deploy the AI agent before connecting a domain.".into(),
        ));
    }
    if agent.public_key.is_none() {
        return Err(AppError::BadRequest(
            "This AI agent has no public widget key.".into(),
        ));
    }

    if let Some(existing) =
        sqlx::query_as::<_, CustomDomain>("SELECT * FROM ai_agent_custom_domains WHERE hostname=$1")
            .bind(&hostname)
            .fetch_optional(&state.pool)
            .await?
    {
        if existing.workspace_id != tenant.workspace_id || existing.agent_id != dto.agent_id {
            return Err(AppError::Conflict(
                "This hostname is already connected to another AI agent.".into(),
            ));
        }
        return Ok((
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": domain_payload(existing, &agent.name),
                "message": "Domain connection already exists"
            })),
        ));
    }

    let domain = sqlx::query_as::<_, CustomDomain>(
        "INSERT INTO ai_agent_custom_domains
         (id,workspace_id,agent_id,hostname,status,dns_target)
         VALUES ($1,$2,$3,$4,'PENDING_DNS',$5)
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(tenant.workspace_id)
    .bind(dto.agent_id)
    .bind(&hostname)
    .bind(DNS_TARGET)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": domain_payload(domain, &agent.name),
            "message": "Domain created. Add the CNAME record, then verify."
        })),
    ))
}

async fn list_agent_domains(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query_as::<_, CustomDomain>(
        "SELECT * FROM ai_agent_custom_domains
         WHERE workspace_id=$1 AND agent_id=$2
         ORDER BY created_at DESC",
    )
    .bind(tenant.workspace_id)
    .bind(agent_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({"success": true, "data": rows})))
}

async fn verify_domain(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let domain = sqlx::query_as::<_, CustomDomain>(
        "SELECT * FROM ai_agent_custom_domains WHERE id=$1 AND workspace_id=$2",
    )
    .bind(id)
    .bind(tenant.workspace_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Custom domain not found".into()))?;

    let agent = sqlx::query_as::<_, crate::models::AiAgent>(
        "SELECT * FROM ai_agents WHERE id=$1 AND workspace_id=$2",
    )
    .bind(domain.agent_id)
    .bind(tenant.workspace_id)
    .fetch_one(&state.pool)
    .await?;

    let dns = check_cname(&domain.hostname, DNS_TARGET).await?;
    let status = if dns { "DNS_CONNECTED" } else { "PENDING_DNS" };

    let updated = sqlx::query_as::<_, CustomDomain>(
        "UPDATE ai_agent_custom_domains
         SET status=$1,last_dns_status=$2,updated_at=NOW()
         WHERE id=$3
         RETURNING *",
    )
    .bind(status)
    .bind(if dns { "MATCH" } else { "NOT_MATCHED" })
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": domain_payload(updated, &agent.name),
        "message": if dns { "DNS verified. The custom domain can now serve the AI chat." } else { "DNS record is not pointing to the GOLD-e target yet." }
    })))
}

async fn check_cname(hostname: &str, expected: &str) -> Result<bool, AppError> {
    let url = format!("https://dns.google/resolve?name={}&type=CNAME", hostname);
    let body: serde_json::Value = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("DNS verification failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::ExternalService(format!("DNS response could not be read: {}", e)))?;

    let expected = expected.trim_end_matches('.').to_ascii_lowercase();
    Ok(body
        .get("Answer")
        .and_then(|v| v.as_array())
        .map(|answers| {
            answers.iter().any(|answer| {
                answer.get("type").and_then(|v| v.as_i64()) == Some(5)
                    && answer
                        .get("data")
                        .and_then(|v| v.as_str())
                        .map(|v| v.trim_end_matches('.').eq_ignore_ascii_case(&expected))
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false))
}

fn normalize_hostname(raw: &str) -> Result<String, AppError> {
    let value = raw.trim().trim_end_matches('.').to_ascii_lowercase();
    if value.is_empty() || value.len() > 255 || value.contains('/') || value.contains("://") {
        return Err(AppError::Validation(
            "Enter a valid hostname such as chat.example.com.".into(),
        ));
    }
    if value == "goldetech.com" || !value.contains('.') {
        return Err(AppError::Validation(
            "Use a subdomain such as chat.example.com. Apex domains require a different DNS setup."
                .into(),
        ));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err(AppError::Validation(
            "Hostname contains unsupported characters.".into(),
        ));
    }
    Ok(value)
}

fn domain_payload(domain: CustomDomain, agent_name: &str) -> serde_json::Value {
    json!({
        "id": domain.id,
        "agent_id": domain.agent_id,
        "hostname": domain.hostname,
        "status": domain.status,
        "dns_target": domain.dns_target,
        "cloudflare_hostname_id": domain.cloudflare_hostname_id,
        "last_dns_status": domain.last_dns_status,
        "last_ssl_status": domain.last_ssl_status,
        "agent_name": agent_name,
        "dns_record": {"type":"CNAME","name":domain.hostname,"target":domain.dns_target},
        "created_at": domain.created_at,
        "updated_at": domain.updated_at
    })
}

pub async fn landing(Extension(state): Extension<AppState>, Host(host): Host) -> Response {
    let hostname = host
        .split(':')
        .next()
        .unwrap_or(host.as_str())
        .to_ascii_lowercase();

    let domain = match sqlx::query_as::<_, CustomDomain>(
        "SELECT * FROM ai_agent_custom_domains
         WHERE hostname=$1 AND status='DNS_CONNECTED'
         LIMIT 1",
    )
    .bind(&hostname)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                "GOLD-e custom domain is not connected.",
            )
                .into_response()
        }
    };

    let agent = match sqlx::query_as::<_, crate::models::AiAgent>(
        "SELECT * FROM ai_agents WHERE id=$1 AND status='ACTIVE'",
    )
    .bind(domain.agent_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(agent)) => agent,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                "GOLD-e AI agent is not active.",
            )
                .into_response()
        }
    };

    let key = match agent.public_key {
        Some(key) => key,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let html = format!(
        r#"<!doctype html>
<html lang="en"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{}</title>
<style>
html,body{{margin:0;height:100%;background:#faf8fd;font-family:Inter,system-ui,sans-serif}}
.page{{min-height:100%;display:grid;place-items:center;padding:24px;box-sizing:border-box}}
.card{{width:min(900px,100%);min-height:680px;background:white;border:1px solid #eee6f5;border-radius:24px;box-shadow:0 20px 60px rgba(50,30,80,.12);display:grid;place-items:center}}
.content{{text-align:center;max-width:560px;padding:40px}}h1{{font-size:34px;margin:0 0 12px;color:#24192e}}p{{color:#766a80;line-height:1.6}}
</style></head>
<body><div class="page"><div class="card"><div class="content"><h1>{}</h1>
<p>This GOLD-e customer assistant is ready. Open the chat button to start a conversation.</p>
</div></div></div>
<script src="https://api.goldetech.com/api/public/agents/{}/widget.js" defer></script>
</body></html>"#,
        agent.name, agent.name, key
    );

    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        html,
    )
        .into_response()
}
