use crate::config::Config;
use crate::error::AppError;
use crate::models::{
    AgentRun, AgentRunWithSteps, AgentStep, AiAgent, AiAgentChatDto, AiAgentChatResponse, AiAgentChannelConnection, ChannelConnectionResponse, CreateChannelConnectionDto, PublicAgentChatDto, PublicAgentChatResponse,
    AiAgentConversation, AiAgentMessage, CreateAgentRunDto, CreateAiAgentDto, StepApprovalDto,
    StepRejectionDto, UpdateAiAgentDto,
};
use crate::utils::pagination::{PaginationMeta, PaginationQuery};
use reqwest::Client;
use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use sha2::{Digest, Sha256};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct AgentTemplate {
    pub key: &'static str,
    pub name: &'static str,
    pub industry: &'static str,
    pub role: &'static str,
    pub description: &'static str,
    pub capabilities: Value,
    pub tools: Value,
    pub channels: Value,
    pub system_prompt: &'static str,
}

pub fn templates() -> Vec<AgentTemplate> {
    vec![
        template(
            "PROPERTY",
            "Property Sales & Rental",
            "PROPERTY",
            "Property Sales & Rental Agent",
            "Qualify buyers and tenants, match inventory, share brochures and arrange site visits.",
            &["sales", "rental", "lead_qualification", "property_matching", "brochure_sharing", "site_visit"],
            &["search_properties", "get_property_details", "check_availability", "send_brochure", "schedule_site_visit", "create_lead", "handoff_to_sales"],
            "You are GOLD-e's property sales and rental agent. Understand the customer's budget, location, property type and intent. Never invent inventory, pricing or availability. Use connected business tools when available. Qualify serious leads and offer a human sales handoff when needed.",
        ),
        template(
            "HEALTHCARE",
            "Healthcare Receptionist",
            "HEALTHCARE",
            "Hospital / Clinic Receptionist",
            "Handle appointments, doctor availability, consultation fees, payment requests and receptionist handoff.",
            &["appointment_booking", "slot_information", "doctor_information", "fee_information", "payment_request", "rescheduling", "cancellation", "human_handoff"],
            &["get_doctors", "get_available_slots", "book_appointment", "reschedule_appointment", "cancel_appointment", "get_consultation_fee", "create_payment_request", "send_payment_qr", "handoff_to_receptionist"],
            "You are a healthcare receptionist assistant. Be precise and calm. Never diagnose, prescribe or invent clinical information. Appointment availability and fees must come from connected systems. Payment is considered successful only after a verified payment event.",
        ),
        template(
            "EDUCATION",
            "School & Coaching Admissions",
            "EDUCATION",
            "Admissions Counsellor",
            "Answer admission enquiries, explain courses and fees, share brochures, check seats and arrange counselling.",
            &["admission_enquiry", "course_information", "fee_information", "brochure_sharing", "seat_availability", "counselling_booking", "seat_booking"],
            &["get_courses", "get_course_details", "get_fee_structure", "get_batch_schedule", "check_seat_availability", "send_brochure", "create_admission_lead", "schedule_counselling", "reserve_seat", "handoff_to_admissions"],
            "You are an education admissions counsellor. Answer from the institution's configured information. Never invent fees, courses, eligibility or seat counts. Guide parents and students toward counselling or admission when appropriate.",
        ),
        template(
            "HOTEL",
            "Hotel Front Desk",
            "HOSPITALITY",
            "Hotel Receptionist",
            "Handle room enquiries, availability, rates, booking requests, amenities and guest handoff.",
            &["room_information", "availability", "rate_information", "booking", "cancellation", "amenities", "guest_handoff"],
            &["search_rooms", "get_room_details", "check_room_availability", "get_rate", "create_booking", "cancel_booking", "send_booking_confirmation", "handoff_to_front_desk"],
            "You are a hotel front-desk assistant. Never invent room availability or rates. Confirm dates, guests and room type before booking. Escalate special requests or exceptions to hotel staff.",
        ),
        template(
            "ACCOMMODATION",
            "Accommodation Booking",
            "ACCOMMODATION",
            "Accommodation Assistant",
            "Help customers compare accommodation options, availability, pricing and booking requirements.",
            &["availability", "pricing", "property_details", "booking", "move_in_information", "handoff"],
            &["search_accommodation", "get_accommodation_details", "check_availability", "get_pricing", "create_booking", "send_details", "handoff_to_manager"],
            "You are an accommodation booking assistant. Collect dates, occupancy, location and budget. Never invent availability or pricing. Present verified options and hand off complex requests to the accommodation team.",
        ),
        template(
            "PG_HOSTEL",
            "PG & Hostel Manager",
            "ACCOMMODATION",
            "PG / Hostel Front Desk",
            "Manage room/bed enquiries, availability, rent, amenities, visit requests and reservations.",
            &["room_or_bed_search", "availability", "rent_information", "amenities", "visit_booking", "bed_reservation", "handoff"],
            &["search_beds", "get_room_details", "check_bed_availability", "get_rent", "send_brochure", "schedule_property_visit", "reserve_bed", "handoff_to_manager"],
            "You are a PG and hostel front-desk assistant. Ask for move-in date, occupancy and budget. Never invent bed availability, rent or amenities. Confirm reservation conditions before taking a booking action.",
        ),
        template(
            "INFRASTRUCTURE",
            "Infrastructure Enquiry Agent",
            "INFRASTRUCTURE",
            "Infrastructure Sales & Enquiry Agent",
            "Handle project enquiries, capability questions, quotations, site requirements and sales-team handoff.",
            &["project_enquiry", "requirement_capture", "capability_information", "quotation_request", "site_visit", "lead_qualification"],
            &["get_capabilities", "capture_project_requirement", "create_lead", "request_quotation", "schedule_site_visit", "send_company_profile", "handoff_to_sales"],
            "You are an infrastructure business enquiry agent. Capture project type, location, scope, timeline and contact details. Never promise pricing, capacity or delivery dates without verified business data. Route qualified opportunities to the sales team.",
        ),
        template(
            "SALES",
            "General Sales Agent",
            "SALES",
            "AI Sales Representative",
            "Qualify inbound prospects, answer product questions, recommend configured offerings and create sales handoffs.",
            &["lead_qualification", "product_information", "pricing_information", "recommendation", "follow_up", "human_handoff"],
            &["search_products", "get_product_details", "get_pricing", "create_lead", "schedule_demo", "start_followup", "handoff_to_sales"],
            "You are GOLD-e's sales representative. Understand the customer's need before recommending an offering. Never invent product features, prices or commitments. Record qualified leads and offer a human sales handoff when appropriate.",
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn template(
    key: &'static str,
    name: &'static str,
    industry: &'static str,
    role: &'static str,
    description: &'static str,
    capabilities: &[&str],
    tools: &[&str],
    system_prompt: &'static str,
) -> AgentTemplate {
    AgentTemplate {
        key,
        name,
        industry,
        role,
        description,
        capabilities: json!(capabilities),
        tools: json!(tools),
        channels: json!(["WHATSAPP", "WEB_CHAT"]),
        system_prompt,
    }
}

pub async fn list_agents(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<AiAgent>, AppError> {
    Ok(sqlx::query_as::<_, AiAgent>(
        "SELECT * FROM ai_agents WHERE workspace_id=$1 ORDER BY created_at DESC"
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_agent(pool: &PgPool, workspace_id: Uuid, id: Uuid) -> Result<AiAgent, AppError> {
    sqlx::query_as::<_, AiAgent>(
        "SELECT * FROM ai_agents WHERE id=$1 AND workspace_id=$2"
    )
    .bind(id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("AI agent not found".into()))
}

pub async fn create_agent(
    pool: &PgPool,
    workspace_id: Uuid,
    user_id: Uuid,
    dto: CreateAiAgentDto,
) -> Result<AiAgent, AppError> {
    let tpl = templates()
        .into_iter()
        .find(|t| t.key.eq_ignore_ascii_case(&dto.template_key))
        .ok_or_else(|| AppError::Validation("Unknown AI agent template".into()))?;

    let id = Uuid::new_v4();
    let public_key = Uuid::new_v4().to_string().replace("-", "");
    let industry = dto.industry.unwrap_or_else(|| tpl.industry.to_string());
    let role = dto.role.unwrap_or_else(|| tpl.role.to_string());
    let description = dto.description.or_else(|| Some(tpl.description.to_string()));
    let system_prompt = dto.system_prompt.unwrap_or_else(|| tpl.system_prompt.to_string());
    let capabilities = dto.capabilities.unwrap_or(tpl.capabilities);
    let tools = dto.tools.unwrap_or(tpl.tools);
    let channels = dto.channels.unwrap_or(tpl.channels);

    sqlx::query_as::<_, AiAgent>(
        "INSERT INTO ai_agents
         (id,workspace_id,name,template_key,industry,role,description,status,system_prompt,capabilities,tools,channels,settings,created_by_id,public_key,welcome_message,handoff_message)
         VALUES ($1,$2,$3,$4,$5,$6,$7,'DRAFT',$8,$9,$10,$11,$12,$13,$14,$15,$16)
         RETURNING *"
    )
    .bind(id)
    .bind(workspace_id)
    .bind(dto.name)
    .bind(tpl.key)
    .bind(industry)
    .bind(role)
    .bind(description)
    .bind(system_prompt)
    .bind(capabilities)
    .bind(tools)
    .bind(channels)
    .bind(dto.settings.unwrap_or_else(|| json!({})))
    .bind(user_id)
    .bind(&public_key)
    .bind("Hi! I'm the AI assistant for this business. How can I help you today?")
    .bind("I'll connect you with a member of our team for that request.")
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn update_agent(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
    dto: UpdateAiAgentDto,
) -> Result<AiAgent, AppError> {
    let current = get_agent(pool, workspace_id, id).await?;
    let status = dto.status.unwrap_or(current.status);
    if !matches!(status.as_str(), "DRAFT" | "ACTIVE" | "PAUSED") {
        return Err(AppError::Validation("Agent status must be DRAFT, ACTIVE or PAUSED".into()));
    }

    sqlx::query_as::<_, AiAgent>(
        "UPDATE ai_agents SET
         name=COALESCE($3,name),
         industry=COALESCE($4,industry),
         role=COALESCE($5,role),
         description=COALESCE($6,description),
         status=$7,
         system_prompt=COALESCE($8,system_prompt),
         capabilities=COALESCE($9,capabilities),
         tools=COALESCE($10,tools),
         channels=COALESCE($11,channels),
         settings=COALESCE($12,settings),
         updated_at=NOW()
         WHERE id=$1 AND workspace_id=$2
         RETURNING *"
    )
    .bind(id)
    .bind(workspace_id)
    .bind(dto.name)
    .bind(dto.industry)
    .bind(dto.role)
    .bind(dto.description)
    .bind(status)
    .bind(dto.system_prompt)
    .bind(dto.capabilities)
    .bind(dto.tools)
    .bind(dto.channels)
    .bind(dto.settings)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn delete_agent(pool: &PgPool, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM ai_agents WHERE id=$1 AND workspace_id=$2")
        .bind(id).bind(workspace_id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("AI agent not found".into()));
    }
    Ok(())
}

pub async fn chat(
    pool: &PgPool,
    config: &Config,
    workspace_id: Uuid,
    agent_id: Uuid,
    dto: AiAgentChatDto,
) -> Result<AiAgentChatResponse, AppError> {
    let agent = get_agent(pool, workspace_id, agent_id).await?;
    if agent.status != "ACTIVE" {
        return Err(AppError::BadRequest("AI agent must be ACTIVE before chat can be used".into()));
    }
    if dto.message.trim().is_empty() {
        return Err(AppError::Validation("message is required".into()));
    }

    let conversation_id = if let Some(id) = dto.conversation_id {
        let exists = sqlx::query_as::<_, AiAgentConversation>(
            "SELECT * FROM ai_agent_conversations WHERE id=$1 AND workspace_id=$2 AND agent_id=$3"
        ).bind(id).bind(workspace_id).bind(agent_id).fetch_optional(pool).await?;
        exists.ok_or_else(|| AppError::NotFound("Conversation not found".into()))?.id
    } else {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO ai_agent_conversations
             (workspace_id,agent_id,channel,external_user_id)
             VALUES ($1,$2,$3,$4) RETURNING id"
        )
        .bind(workspace_id).bind(agent_id)
        .bind(dto.channel.unwrap_or_else(|| "WEB_CHAT".into()))
        .bind(dto.external_user_id)
        .fetch_one(pool).await?
    };

    sqlx::query(
        "INSERT INTO ai_agent_messages (conversation_id,role,content) VALUES ($1,'user',$2)"
    ).bind(conversation_id).bind(&dto.message).execute(pool).await?;

    let history = sqlx::query_as::<_, AiAgentMessage>(
        "SELECT * FROM ai_agent_messages WHERE conversation_id=$1 ORDER BY created_at DESC LIMIT 20"
    ).bind(conversation_id).fetch_all(pool).await?;

    let api_key = config.ai_api_key.clone().ok_or_else(|| AppError::BadRequest(
        "AI provider is not configured. Set AI_API_KEY on the backend.".into()
    ))?;
    let base_url = config.ai_api_base_url.trim_end_matches('/');
    let model = config.ai_model.clone();

    let mut input = Vec::new();
    for msg in history.iter().rev() {
        if msg.role == "user" || msg.role == "assistant" {
            input.push(json!({
                "role": msg.role,
                "content": msg.content
            }));
        }
    }

    let messages: Vec<Value> = input.to_vec();
    let mut messages = messages;
    messages.insert(0, json!({
        "role": "system",
        "content": agent.system_prompt
    }));

    let body = json!({
        "model": model,
        "messages": messages,
        "temperature": config.ai_temperature
    });

    let response = Client::new()
        .post(format!("{}/chat/completions", base_url))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("AI provider request failed: {}", e)))?;

    let status = response.status();
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let raw_body = response.text().await
        .map_err(|e| AppError::ExternalService(format!("Failed reading AI provider response (HTTP {}): {}", status, e)))?;

    let payload: Value = serde_json::from_str(&raw_body).map_err(|e| {
        let preview: String = raw_body.chars().take(1000).collect();
        AppError::ExternalService(format!(
            "Invalid AI provider response (HTTP {}, content-type {}): {} | body: {}",
            status, content_type, e, preview
        ))
    })?;

    if !status.is_success() {
        let message = payload.get("error").and_then(|e| e.get("message")).and_then(Value::as_str)
            .or_else(|| payload.get("message").and_then(Value::as_str))
            .unwrap_or("AI provider returned an error");
        return Err(AppError::ExternalService(format!("HTTP {}: {}", status, message)));
    }

    let reply = payload.get("choices").and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AppError::ExternalService("AI provider returned no text output".into()))?;

    sqlx::query(
        "INSERT INTO ai_agent_messages (conversation_id,role,content) VALUES ($1,'assistant',$2)"
    ).bind(conversation_id).bind(&reply).execute(pool).await?;
    sqlx::query("UPDATE ai_agent_conversations SET updated_at=NOW() WHERE id=$1")
        .bind(conversation_id).execute(pool).await?;

    Ok(AiAgentChatResponse {
        conversation_id,
        agent_id,
        reply,
        model: config.ai_model.clone(),
    })
}

// Existing agent-run API remains supported below.

pub async fn list_runs(
    pool: &PgPool,
    workspace_id: Uuid,
    query: PaginationQuery,
    status: Option<String>,
) -> Result<(Vec<AgentRunWithSteps>, PaginationMeta), AppError> {
    let limit = query.limit();
    let offset = query.offset();
    let page = query.page();

    let runs = sqlx::query_as::<_, AgentRun>(
        "SELECT * FROM agent_runs
         WHERE workspace_id = $1
           AND ($2::text IS NULL OR status = $2)
         ORDER BY created_at DESC
         LIMIT $3 OFFSET $4"
    )
    .bind(workspace_id)
    .bind(&status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count_row = sqlx::query!(
        "SELECT COUNT(*) as total FROM agent_runs
         WHERE workspace_id = $1
           AND ($2::text IS NULL OR status = $2)",
        workspace_id,
        status
    )
    .fetch_one(pool)
    .await?;

    let total = count_row.total.unwrap_or(0);
    let meta = PaginationMeta::new(total, page, limit);

    let mut result = Vec::new();
    for run in runs {
        let steps = sqlx::query_as::<_, AgentStep>(
            "SELECT * FROM agent_steps WHERE run_id = $1 ORDER BY step_number ASC"
        )
        .bind(run.id)
        .fetch_all(pool)
        .await?;

        result.push(AgentRunWithSteps { run, steps });
    }

    Ok((result, meta))
}

pub async fn get_run(pool: &PgPool, workspace_id: Uuid, run_id: Uuid) -> Result<AgentRunWithSteps, AppError> {
    let run = sqlx::query_as::<_, AgentRun>("SELECT * FROM agent_runs WHERE id = $1 AND workspace_id = $2")
        .bind(run_id).bind(workspace_id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::NotFound("Agent run not found".to_string()))?;
    let steps = sqlx::query_as::<_, AgentStep>("SELECT * FROM agent_steps WHERE run_id = $1 ORDER BY step_number ASC")
        .bind(run.id).fetch_all(pool).await?;
    Ok(AgentRunWithSteps { run, steps })
}

pub async fn create_run(pool: &PgPool, workspace_id: Uuid, user_id: Uuid, dto: CreateAgentRunDto) -> Result<AgentRunWithSteps, AppError> {
    let run_id = Uuid::new_v4();
    let input_params = dto.input_params.unwrap_or_else(|| json!({}));
    let mut tx = pool.begin().await?;
    sqlx::query_as::<_, AgentRun>(
        "INSERT INTO agent_runs (id, workspace_id, agent_type, name, status, triggered_by_id, input_params, started_at)
         VALUES ($1, $2, $3, $4, 'WAITING_APPROVAL', $5, $6, NOW()) RETURNING *"
    ).bind(run_id).bind(workspace_id).bind(&dto.agent_type).bind(&dto.name).bind(user_id).bind(&input_params).fetch_one(&mut *tx).await?;
    sqlx::query!(
        "INSERT INTO agent_steps (id, run_id, step_number, name, description, action_type, status, requires_approval, started_at, completed_at)
         VALUES ($1,$2,1,'Data Ingestion & Contact Lead Scoring','Scored contacts based on engagement metrics','LEAD_SCORING','COMPLETED',false,NOW()-INTERVAL '2 minutes',NOW())",
        Uuid::new_v4(), run_id
    ).execute(&mut *tx).await?;
    sqlx::query!(
        "INSERT INTO agent_steps (id, run_id, step_number, name, description, action_type, status, requires_approval, input_payload, started_at)
         VALUES ($1,$2,2,'Executive Approval for Automated Outreach Campaign','Gatekeeper check for multi-channel message dispatch','APPROVAL_GATEWAY','WAITING_APPROVAL',true,$3,NOW())",
        Uuid::new_v4(), run_id, json!({"recipientCount":15,"channel":"WHATSAPP"})
    ).execute(&mut *tx).await?;
    sqlx::query!(
        "INSERT INTO agent_steps (id,run_id,step_number,name,description,action_type,status,requires_approval)
         VALUES ($1,$2,3,'Execute WhatsApp Template Dispatch','Dispatches approved templates to target audience','WHATSAPP_DISPATCH','PENDING',false)",
        Uuid::new_v4(), run_id
    ).execute(&mut *tx).await?;
    tx.commit().await?;
    get_run(pool, workspace_id, run_id).await
}

pub async fn approve_step(pool: &PgPool, workspace_id: Uuid, user_id: Uuid, run_id: Uuid, step_id: Uuid, _dto: StepApprovalDto) -> Result<AgentRunWithSteps, AppError> {
    let _run = sqlx::query_as::<_, AgentRun>("SELECT * FROM agent_runs WHERE id=$1 AND workspace_id=$2")
        .bind(run_id).bind(workspace_id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::NotFound("Agent run not found in workspace".to_string()))?;
    let step = sqlx::query_as::<_, AgentStep>("SELECT * FROM agent_steps WHERE id=$1 AND run_id=$2")
        .bind(step_id).bind(run_id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::NotFound("Agent step not found".to_string()))?;
    if step.status != "WAITING_APPROVAL" {
        return Err(AppError::BadRequest(format!("Step is not waiting for approval (Current status: {})", step.status)));
    }
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "UPDATE agent_steps SET status='APPROVED',approved_by_id=$3,approved_at=NOW(),completed_at=NOW(),updated_at=NOW() WHERE id=$1 AND run_id=$2",
        step_id,run_id,user_id
    ).execute(&mut *tx).await?;
    sqlx::query!(
        "UPDATE agent_steps SET status='COMPLETED',started_at=NOW(),completed_at=NOW(),updated_at=NOW() WHERE run_id=$1 AND status='PENDING'",
        run_id
    ).execute(&mut *tx).await?;
    sqlx::query!("UPDATE agent_runs SET status='COMPLETED',completed_at=NOW(),updated_at=NOW() WHERE id=$1",run_id)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    get_run(pool, workspace_id, run_id).await
}

pub async fn reject_step(pool: &PgPool, workspace_id: Uuid, _user_id: Uuid, run_id: Uuid, step_id: Uuid, dto: StepRejectionDto) -> Result<AgentRunWithSteps, AppError> {
    let _run = sqlx::query_as::<_, AgentRun>("SELECT * FROM agent_runs WHERE id=$1 AND workspace_id=$2")
        .bind(run_id).bind(workspace_id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::NotFound("Agent run not found".to_string()))?;
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "UPDATE agent_steps SET status='REJECTED',rejection_reason=$3,completed_at=NOW(),updated_at=NOW() WHERE id=$1 AND run_id=$2",
        step_id,run_id,dto.reason
    ).execute(&mut *tx).await?;
    sqlx::query!("UPDATE agent_runs SET status='CANCELLED',completed_at=NOW(),updated_at=NOW() WHERE id=$1",run_id)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    get_run(pool, workspace_id, run_id).await
}


fn channel_key(config: &Config) -> Result<[u8; 32], AppError> {
    let master = config.channel_encryption_key.as_deref().ok_or_else(|| {
        AppError::BadRequest("CHANNEL_ENCRYPTION_KEY must be configured before connecting customer channels.".into())
    })?;
    let digest = Sha256::digest(master.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    Ok(key)
}

fn encrypt_channel_secret(config: &Config, secret: &str) -> Result<String, AppError> {
    let key = channel_key(config)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::BadRequest("Invalid channel encryption key".into()))?;
    let nonce_uuid = Uuid::new_v4();
    let nonce_bytes = &nonce_uuid.as_bytes()[..12];
    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, secret.as_bytes())
        .map_err(|_| AppError::ExternalService("Unable to encrypt channel credential".into()))?;
    let mut packed = nonce_bytes.to_vec();
    packed.extend_from_slice(&ciphertext);
    Ok(B64.encode(packed))
}

fn decrypt_channel_secret(config: &Config, value: &str) -> Result<String, AppError> {
    let key = channel_key(config)?;
    let packed = B64.decode(value).map_err(|_| AppError::ExternalService("Invalid stored channel credential".into()))?;
    if packed.len() < 13 { return Err(AppError::ExternalService("Invalid stored channel credential".into())); }
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::BadRequest("Invalid channel encryption key".into()))?;
    let nonce = Nonce::from_slice(&packed[..12]);
    let plaintext = cipher.decrypt(nonce, &packed[12..])
        .map_err(|_| AppError::ExternalService("Unable to decrypt channel credential".into()))?;
    String::from_utf8(plaintext).map_err(|_| AppError::ExternalService("Invalid channel credential encoding".into()))
}

pub async fn list_channel_connections(pool: &PgPool, workspace_id: Uuid, agent_id: Uuid) -> Result<Vec<ChannelConnectionResponse>, AppError> {
    let rows = sqlx::query_as::<_, AiAgentChannelConnection>(
        "SELECT * FROM ai_agent_channel_connections WHERE workspace_id=$1 AND agent_id=$2 ORDER BY created_at DESC"
    ).bind(workspace_id).bind(agent_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| ChannelConnectionResponse {
        id:r.id, agent_id:r.agent_id, channel:r.channel, provider:r.provider,
        status:r.status, external_account_id:r.external_account_id,
        external_sender_id:r.external_sender_id, display_name:r.display_name,
        config:r.config, secret_configured:r.secret_ciphertext.is_some(),
        created_at:r.created_at, updated_at:r.updated_at
    }).collect())
}

pub async fn create_channel_connection(pool: &PgPool, config: &Config, workspace_id: Uuid, agent_id: Uuid, dto: CreateChannelConnectionDto) -> Result<ChannelConnectionResponse, AppError> {
    let agent = get_agent(pool, workspace_id, agent_id).await?;
    let channel = dto.channel.trim().to_uppercase();
    if !matches!(channel.as_str(), "WHATSAPP" | "INSTAGRAM" | "FACEBOOK" | "WEBSITE") {
        return Err(AppError::Validation("Supported channels: WHATSAPP, INSTAGRAM, FACEBOOK, WEBSITE".into()));
    }
    let provider = dto.provider.unwrap_or_else(|| if channel == "WEBSITE" { "GOLD-E".into() } else { "META".into() });
    let secret_ciphertext = dto.secret.as_deref().filter(|s| !s.trim().is_empty()).map(|s| encrypt_channel_secret(config, s)).transpose()?;
    let row=sqlx::query_as::<_,AiAgentChannelConnection>(
        "INSERT INTO ai_agent_channel_connections
        (id,workspace_id,agent_id,channel,provider,status,external_account_id,external_sender_id,display_name,config,secret_ciphertext)
        VALUES ($1,$2,$3,$4,$5,'ACTIVE',$6,$7,$8,$9,$10) RETURNING *"
    ).bind(Uuid::new_v4()).bind(workspace_id).bind(agent.id).bind(&channel).bind(provider)
     .bind(dto.external_account_id).bind(dto.external_sender_id).bind(dto.display_name)
     .bind(dto.config.unwrap_or_else(||json!({}))).bind(secret_ciphertext).fetch_one(pool).await?;
    Ok(ChannelConnectionResponse {
        id:row.id, agent_id:row.agent_id, channel:row.channel, provider:row.provider,
        status:row.status, external_account_id:row.external_account_id,
        external_sender_id:row.external_sender_id, display_name:row.display_name,
        config:row.config, secret_configured:row.secret_ciphertext.is_some(),
        created_at:row.created_at, updated_at:row.updated_at
    })
}

pub async fn delete_channel_connection(pool:&PgPool, workspace_id:Uuid, id:Uuid)->Result<(),AppError>{
    let result=sqlx::query("DELETE FROM ai_agent_channel_connections WHERE id=$1 AND workspace_id=$2")
        .bind(id).bind(workspace_id).execute(pool).await?;
    if result.rows_affected()==0 { return Err(AppError::NotFound("Channel connection not found".into())); }
    Ok(())
}


// List customer conversations for the business owner console.
pub async fn list_agent_conversations(
    pool: &PgPool,
    workspace_id: Uuid,
    agent_id: Uuid,
) -> Result<Vec<crate::models::AgentConversationSummary>, AppError> {
    let _agent = get_agent(pool, workspace_id, agent_id).await?;
    Ok(sqlx::query_as::<_, crate::models::AgentConversationSummary>(
        "SELECT c.id, c.agent_id, c.channel, c.external_user_id, c.status, c.contact_id,
                NULLIF(trim(concat_ws(' ', ct.first_name, ct.last_name)), '') AS contact_name,
                ct.phone AS contact_phone,
                lm.content AS last_message,
                lm.role AS last_message_role,
                lm.created_at AS last_message_at
         FROM ai_agent_conversations c
         LEFT JOIN contacts ct ON ct.id=c.contact_id AND ct.workspace_id=c.workspace_id
         LEFT JOIN LATERAL (
             SELECT content, role, created_at
             FROM ai_agent_messages
             WHERE conversation_id=c.id
             ORDER BY created_at DESC
             LIMIT 1
         ) lm ON TRUE
         WHERE c.workspace_id=$1 AND c.agent_id=$2
         ORDER BY COALESCE(lm.created_at,c.updated_at) DESC"
    )
    .bind(workspace_id).bind(agent_id).fetch_all(pool).await?)
}

pub async fn get_agent_conversation(
    pool: &PgPool,
    workspace_id: Uuid,
    conversation_id: Uuid,
) -> Result<(crate::models::AgentConversationSummary, Vec<AiAgentMessage>), AppError> {
    let conversation = sqlx::query_as::<_, crate::models::AgentConversationSummary>(
        "SELECT c.id, c.agent_id, c.channel, c.external_user_id, c.status, c.contact_id,
                NULLIF(trim(concat_ws(' ', ct.first_name, ct.last_name)), '') AS contact_name,
                ct.phone AS contact_phone,
                lm.content AS last_message,
                lm.role AS last_message_role,
                lm.created_at AS last_message_at
         FROM ai_agent_conversations c
         LEFT JOIN contacts ct ON ct.id=c.contact_id AND ct.workspace_id=c.workspace_id
         LEFT JOIN LATERAL (
             SELECT content, role, created_at
             FROM ai_agent_messages
             WHERE conversation_id=c.id
             ORDER BY created_at DESC
             LIMIT 1
         ) lm ON TRUE
         WHERE c.id=$1 AND c.workspace_id=$2"
    )
    .bind(conversation_id).bind(workspace_id).fetch_optional(pool).await?
    .ok_or_else(|| AppError::NotFound("Customer conversation not found".into()))?;
    let messages = sqlx::query_as::<_, AiAgentMessage>(
        "SELECT * FROM ai_agent_messages WHERE conversation_id=$1 ORDER BY created_at ASC"
    ).bind(conversation_id).fetch_all(pool).await?;
    Ok((conversation, messages))
}

pub async fn owner_reply(
    pool: &PgPool,
    config: &Config,
    workspace_id: Uuid,
    conversation_id: Uuid,
    dto: crate::models::OwnerReplyDto,
) -> Result<AiAgentMessage, AppError> {
    let text = dto.message.trim();
    if text.is_empty() { return Err(AppError::Validation("message is required".into())); }
    if text.len() > 4000 { return Err(AppError::Validation("message is too long".into())); }

    let conversation = sqlx::query_as::<_, AiAgentConversation>(
        "SELECT * FROM ai_agent_conversations WHERE id=$1 AND workspace_id=$2"
    ).bind(conversation_id).bind(workspace_id).fetch_optional(pool).await?
     .ok_or_else(|| AppError::NotFound("Customer conversation not found".into()))?;

    let channel = conversation.channel.to_uppercase();
    let recipient = conversation.external_user_id.clone()
        .ok_or_else(|| AppError::BadRequest("Customer channel recipient is missing".into()))?;
    if channel == "WEBSITE" {
        return Err(AppError::BadRequest("Website owner replies are not supported by the current widget transport. Use a Meta customer channel or the AI handoff flow.".into()));
    }

    let connection = sqlx::query_as::<_, AiAgentChannelConnection>(
        "SELECT * FROM ai_agent_channel_connections
         WHERE workspace_id=$1 AND agent_id=$2 AND channel=$3 AND status='ACTIVE'
           ORDER BY created_at DESC LIMIT 1"
    ).bind(workspace_id).bind(conversation.agent_id).bind(&channel)
     .fetch_optional(pool).await?
     .ok_or_else(|| AppError::NotFound("No active customer-channel connection is configured for this conversation".into()))?;

    let secret = connection.secret_ciphertext.as_deref()
        .ok_or_else(|| AppError::BadRequest("Channel access token is not configured".into()))?;
    let token = decrypt_channel_secret(config, secret)?;
    let target_id = connection.external_account_id.as_deref()
        .ok_or_else(|| AppError::BadRequest("Channel account ID is not configured".into()))?;

    let payload = match channel.as_str() {
        "WHATSAPP" => json!({
            "messaging_product":"whatsapp","to":recipient,"type":"text",
            "text":{"preview_url":false,"body":text}
        }),
        "INSTAGRAM" | "FACEBOOK" => json!({
            "recipient":{"id":recipient},"message":{"text":text}
        }),
        _ => return Err(AppError::BadRequest("Unsupported owner reply channel".into())),
    };

    let url = format!("https://graph.facebook.com/v20.0/{}/messages", target_id);
    let response = Client::new().post(url).bearer_auth(token).json(&payload).send().await
        .map_err(|e| AppError::ExternalService(format!("Meta owner reply failed: {}", e)))?;
    let status = response.status();
    let body: Value = response.json().await.unwrap_or_else(|_| json!({}));
    if !status.is_success() {
        let detail = body.get("error").and_then(|e| e.get("message")).and_then(Value::as_str)
            .unwrap_or("Meta rejected the owner reply");
        return Err(AppError::ExternalService(format!("HTTP {}: {}", status, detail)));
    }

    let external_id = body.get("messages").and_then(Value::as_array).and_then(|a| a.first())
        .and_then(|m| m.get("id")).and_then(Value::as_str).map(str::to_string);

    let saved = sqlx::query_as::<_, AiAgentMessage>(
        "INSERT INTO ai_agent_messages (conversation_id,role,content,metadata)
         VALUES ($1,'assistant',$2,$3) RETURNING *"
    ).bind(conversation_id).bind(text)
     .bind(json!({"source":"OWNER","external_message_id":external_id}))
     .fetch_one(pool).await?;
    sqlx::query("UPDATE ai_agent_conversations SET status='HUMAN',updated_at=NOW() WHERE id=$1")
        .bind(conversation_id).execute(pool).await?;
    Ok(saved)
}

pub async fn connect_website_via_cpanel(
    pool: &PgPool,
    workspace_id: Uuid,
    agent_id: Uuid,
    website_url: &str,
    cpanel_host: &str,
    cpanel_username: &str,
    cpanel_token: &str,
) -> Result<Value, AppError> {
    let agent = get_agent(pool, workspace_id, agent_id).await?;
    if agent.status != "ACTIVE" {
        return Err(AppError::BadRequest("Deploy the AI agent before connecting a website.".into()));
    }
    let public_key = agent.public_key.clone()
        .ok_or_else(|| AppError::BadRequest("This AI agent has no public widget key.".into()))?;

    let website_url = website_url.trim().trim_end_matches('/');
    let parsed = reqwest::Url::parse(website_url)
        .map_err(|_| AppError::Validation("Enter a valid website URL.".into()))?;
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return Err(AppError::Validation("Website URL must use http or https.".into()));
    }

    let host = cpanel_host.trim()
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    if host.is_empty() || cpanel_username.trim().is_empty() || cpanel_token.trim().is_empty() {
        return Err(AppError::Validation("cPanel host, username and API token are required.".into()));
    }

    let base = format!("https://{}:2083/execute", host);
    let auth = format!("cpanel {}:{}", cpanel_username.trim(), cpanel_token.trim());
    let client = Client::new();

    let dir = format!("/home/{}/public_html", cpanel_username.trim());
    let file = "index.html";

    // Read the current file first. We never overwrite a file we cannot read.
    let read = client
        .get(format!("{}/Fileman/get_file_content", base))
        .header("Authorization", &auth)
        .query(&[("dir", dir.as_str()), ("file", file)])
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("cPanel connection failed: {}", e)))?;

    let read_status = read.status();
    let read_body: Value = read.json().await.unwrap_or_else(|_| json!({}));
    if !read_status.is_success() {
        return Err(AppError::ExternalService(format!("cPanel returned HTTP {}", read_status)));
    }
    if read_body.get("status").and_then(Value::as_i64) == Some(0) {
        let reason = read_body.get("errors")
            .and_then(Value::as_array).and_then(|a| a.first())
            .and_then(Value::as_str).unwrap_or("cPanel could not read index.html");
        return Err(AppError::ExternalService(format!("cPanel: {}", reason)));
    }

    let current = read_body.get("data")
        .and_then(|d| d.get("content"))
        .and_then(Value::as_str)
        .or_else(|| read_body.get("data").and_then(Value::as_str))
        .ok_or_else(|| AppError::ExternalService("cPanel did not return index.html content.".into()))?;

    let script = format!(
        r#"<script src="https://api.goldetech.com/api/public/agents/{}/widget.js" defer></script>"#,
        public_key
    );

    if current.contains(&format!("api.goldetech.com/api/public/agents/{}/widget.js", public_key)) {
        let _ = sqlx::query(
            "INSERT INTO ai_agent_channel_connections
             (workspace_id,agent_id,channel,provider,status,display_name,config)
             VALUES ($1,$2,'WEBSITE','GOLD-E','ACTIVE',$3,$4)
             ON CONFLICT DO NOTHING"
        )
        .bind(workspace_id)
        .bind(agent_id)
        .bind(website_url)
        .bind(json!({"installation":"cpanel","website_url":website_url,"file":"index.html"}))
        .execute(pool).await;
        return Ok(json!({
            "installed": true,
            "already_installed": true,
            "website_url": website_url,
            "public_key": public_key,
            "file": "index.html"
        }));
    }

    let lower = current.to_ascii_lowercase();
    let insertion = if let Some(pos) = lower.rfind("</head>") {
        let mut out = String::with_capacity(current.len() + script.len() + 1);
        out.push_str(&current[..pos]);
        out.push_str(&script);
        out.push_str(&current[pos..]);
        out
    } else {
        return Err(AppError::BadRequest("index.html does not contain a </head> tag. Use the manual installer for this site.".into()));
    };

    // Back up the original file before writing the modified index.
    let backup_name = "index.html.golde-backup";
    let backup = client
        .post(format!("{}/Fileman/save_file_content", base))
        .header("Authorization", &auth)
        .form(&[
            ("dir", dir.as_str()),
            ("file", backup_name),
            ("content", current),
        ])
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("cPanel backup failed: {}", e)))?;
    let backup_body: Value = backup.json().await.unwrap_or_else(|_| json!({}));
    if backup_body.get("status").and_then(Value::as_i64) == Some(0) {
        return Err(AppError::ExternalService("cPanel could not create the website backup.".into()));
    }

    let save = client
        .post(format!("{}/Fileman/save_file_content", base))
        .header("Authorization", &auth)
        .form(&[
            ("dir", dir.as_str()),
            ("file", file),
            ("content", insertion.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("cPanel write failed: {}", e)))?;
    let save_status = save.status();
    let save_body: Value = save.json().await.unwrap_or_else(|_| json!({}));
    if !save_status.is_success() || save_body.get("status").and_then(Value::as_i64) == Some(0) {
        return Err(AppError::ExternalService("cPanel could not save index.html.".into()));
    }

    let verify = client.get(website_url).send().await
        .map_err(|e| AppError::ExternalService(format!("Website verification failed: {}", e)))?;
    if !verify.status().is_success() {
        return Err(AppError::ExternalService(format!("Website returned HTTP {} after installation.", verify.status())));
    }
    let live = verify.text().await.unwrap_or_default();
    let verified = live.contains(&format!("api.goldetech.com/api/public/agents/{}/widget.js", public_key));
    if !verified {
        return Err(AppError::ExternalService("File was saved, but the live website did not expose the GOLD-e widget script yet.".into()));
    }

    let _ = sqlx::query(
        "INSERT INTO ai_agent_channel_connections
         (workspace_id,agent_id,channel,provider,status,display_name,config)
         VALUES ($1,$2,'WEBSITE','GOLD-E','ACTIVE',$3,$4)
         ON CONFLICT DO NOTHING"
    )
    .bind(workspace_id)
    .bind(agent_id)
    .bind(website_url)
    .bind(json!({"installation":"cpanel","website_url":website_url,"file":"index.html","backup_file":backup_name}))
    .execute(pool).await;

    Ok(json!({
        "installed": true,
        "already_installed": false,
        "website_url": website_url,
        "public_key": public_key,
        "file": "index.html",
        "backup_file": backup_name
    }))
}


pub async fn public_chat(pool:&PgPool, config:&Config, public_key:&str, dto:PublicAgentChatDto)->Result<PublicAgentChatResponse,AppError>{
    let agent=sqlx::query_as::<_,AiAgent>("SELECT * FROM ai_agents WHERE public_key=$1 AND status='ACTIVE'")
        .bind(public_key).fetch_optional(pool).await?
        .ok_or_else(||AppError::NotFound("Published AI agent not found".into()))?;
    let result=chat(pool,config,agent.workspace_id,agent.id,AiAgentChatDto{
        conversation_id:dto.conversation_id, external_user_id:dto.visitor_id,
        channel:Some("WEBSITE".into()), message:dto.message,
    }).await?;
    Ok(PublicAgentChatResponse{conversation_id:result.conversation_id,reply:result.reply,agent_name:agent.name,channel:"WEBSITE".into()})
}


pub async fn handle_meta_inbound(
    pool:&PgPool, config:&Config, channel:&str, external_account_id:&str, sender_id:&str, message:&str
)->Result<String,AppError>{
    let connection=sqlx::query_as::<_,AiAgentChannelConnection>(
        "SELECT * FROM ai_agent_channel_connections
         WHERE channel=$1 AND external_account_id=$2 AND status='ACTIVE'
         ORDER BY created_at DESC LIMIT 1"
    ).bind(channel).bind(external_account_id).fetch_optional(pool).await?
     .ok_or_else(||AppError::NotFound("No active AI agent channel connection found".into()))?;

    let secret=connection.secret_ciphertext.as_deref().ok_or_else(||AppError::BadRequest("Channel access token is not configured".into()))?;
    let token=decrypt_channel_secret(config,secret)?;

    let contact_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM contacts WHERE workspace_id=$1
         AND regexp_replace(phone,'\\D','','g')=regexp_replace($2,'\\D','','g')
         ORDER BY created_at ASC LIMIT 1"
    ).bind(connection.workspace_id).bind(sender_id).fetch_optional(pool).await?;

    let contact_id = if let Some(id) = contact_id { Some(id) } else {
        Some(sqlx::query_scalar::<_,Uuid>(
            "INSERT INTO contacts (workspace_id,first_name,phone,status)
             VALUES ($1,$2,$3,'ACTIVE') RETURNING id"
        ).bind(connection.workspace_id).bind("Customer").bind(sender_id).fetch_one(pool).await?)
    };

    let conversation_id: Uuid = if let Some(id) = sqlx::query_scalar(
        "SELECT id FROM ai_agent_conversations
         WHERE workspace_id=$1 AND agent_id=$2 AND channel=$3 AND external_user_id=$4
           AND status <> 'CLOSED'
         ORDER BY updated_at DESC LIMIT 1"
    ).bind(connection.workspace_id).bind(connection.agent_id).bind(channel).bind(sender_id)
     .fetch_optional(pool).await? {
        sqlx::query("UPDATE ai_agent_conversations SET contact_id=$2,updated_at=NOW() WHERE id=$1")
            .bind(id).bind(contact_id).execute(pool).await?;
        id
    } else {
        sqlx::query_scalar::<_,Uuid>(
            "INSERT INTO ai_agent_conversations
             (workspace_id,agent_id,contact_id,channel,external_user_id)
             VALUES ($1,$2,$3,$4,$5) RETURNING id"
        ).bind(connection.workspace_id).bind(connection.agent_id).bind(contact_id).bind(channel).bind(sender_id)
         .fetch_one(pool).await?
    };

    let current_status: String = sqlx::query_scalar(
        "SELECT status FROM ai_agent_conversations WHERE id=$1"
    ).bind(conversation_id).fetch_one(pool).await?;

    if current_status == "HUMAN" {
        sqlx::query(
            "INSERT INTO ai_agent_messages (conversation_id,role,content,metadata)
             VALUES ($1,'user',$2,$3)"
        ).bind(conversation_id).bind(message)
         .bind(json!({"source":"CUSTOMER","channel":channel}))
         .execute(pool).await?;
        sqlx::query("UPDATE ai_agent_conversations SET updated_at=NOW() WHERE id=$1")
            .bind(conversation_id).execute(pool).await?;
        return Ok("Customer message recorded for owner".into());
    }

    let result=chat(pool,config,connection.workspace_id,connection.agent_id,AiAgentChatDto{
        conversation_id:Some(conversation_id), external_user_id:Some(sender_id.to_string()),
        channel:Some(channel.to_string()), message:message.to_string()
    }).await?;

    let http=Client::new();
    let payload=match channel {
        "WHATSAPP" => json!({
            "messaging_product":"whatsapp","to":sender_id,"type":"text",
            "text":{"preview_url":false,"body":result.reply}
        }),
        _ => json!({
            "recipient":{"id":sender_id},
            "message":{"text":result.reply}
        }),
    };
    let target_id = connection.external_account_id.as_deref().unwrap_or(external_account_id);
    let url=format!("https://graph.facebook.com/v20.0/{}/messages",target_id);
    let response=http.post(url).bearer_auth(token).json(&payload).send().await
        .map_err(|e|AppError::ExternalService(format!("Meta send failed: {}",e)))?;
    if !response.status().is_success() {
        let body=response.text().await.unwrap_or_default();
        return Err(AppError::ExternalService(format!("Meta send error: {}",body)));
    }
    Ok(result.reply)
}
