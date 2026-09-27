use crate::config::Config;
use crate::error::AppError;
use crate::models::{
    AgentRun, AgentRunWithSteps, AgentStep, AiAgent, AiAgentChatDto, AiAgentChatResponse,
    AiAgentConversation, AiAgentMessage, CreateAgentRunDto, CreateAiAgentDto, StepApprovalDto,
    StepRejectionDto, UpdateAiAgentDto,
};
use crate::utils::pagination::{PaginationMeta, PaginationQuery};
use reqwest::Client;
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
    let industry = dto.industry.unwrap_or_else(|| tpl.industry.to_string());
    let role = dto.role.unwrap_or_else(|| tpl.role.to_string());
    let description = dto.description.or_else(|| Some(tpl.description.to_string()));
    let system_prompt = dto.system_prompt.unwrap_or_else(|| tpl.system_prompt.to_string());
    let capabilities = dto.capabilities.unwrap_or(tpl.capabilities);
    let tools = dto.tools.unwrap_or(tpl.tools);
    let channels = dto.channels.unwrap_or(tpl.channels);

    sqlx::query_as::<_, AiAgent>(
        "INSERT INTO ai_agents
         (id,workspace_id,name,template_key,industry,role,description,status,system_prompt,capabilities,tools,channels,settings,created_by_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,'DRAFT',$8,$9,$10,$11,$12,$13)
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

    let messages: Vec<Value> = input.iter().cloned().collect();
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
