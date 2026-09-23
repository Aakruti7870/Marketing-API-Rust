use crate::error::AppError;
use crate::models::{AgentRun, AgentRunWithSteps, AgentStep, CreateAgentRunDto, StepApprovalDto, StepRejectionDto};
use crate::utils::pagination::{PaginationMeta, PaginationQuery};
use sqlx::PgPool;
use uuid::Uuid;

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

pub async fn get_run(
    pool: &PgPool,
    workspace_id: Uuid,
    run_id: Uuid,
) -> Result<AgentRunWithSteps, AppError> {
    let run = sqlx::query_as::<_, AgentRun>(
        "SELECT * FROM agent_runs WHERE id = $1 AND workspace_id = $2"
    )
    .bind(run_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Agent run not found".to_string()))?;

    let steps = sqlx::query_as::<_, AgentStep>(
        "SELECT * FROM agent_steps WHERE run_id = $1 ORDER BY step_number ASC"
    )
    .bind(run.id)
    .fetch_all(pool)
    .await?;

    Ok(AgentRunWithSteps { run, steps })
}

pub async fn create_run(
    pool: &PgPool,
    workspace_id: Uuid,
    user_id: Uuid,
    dto: CreateAgentRunDto,
) -> Result<AgentRunWithSteps, AppError> {
    let run_id = Uuid::new_v4();
    let input_params = dto.input_params.unwrap_or_else(|| serde_json::json!({}));

    let mut tx = pool.begin().await?;

    let _run = sqlx::query_as::<_, AgentRun>(
        "INSERT INTO agent_runs (id, workspace_id, agent_type, name, status, triggered_by_id, input_params, started_at)
         VALUES ($1, $2, $3, $4, 'WAITING_APPROVAL', $5, $6, NOW())
         RETURNING *"
    )
    .bind(run_id)
    .bind(workspace_id)
    .bind(&dto.agent_type)
    .bind(&dto.name)
    .bind(user_id)
    .bind(&input_params)
    .fetch_one(&mut *tx)
    .await?;

    // Step 1: Completed analysis
    sqlx::query!(
        "INSERT INTO agent_steps (
            id, run_id, step_number, name, description, action_type, status,
            requires_approval, started_at, completed_at
         )
         VALUES ($1, $2, 1, 'Data Ingestion & Contact Lead Scoring', 'Scored contacts based on engagement metrics', 'LEAD_SCORING', 'COMPLETED', false, NOW() - INTERVAL '2 minutes', NOW())",
        Uuid::new_v4(),
        run_id
    )
    .execute(&mut *tx)
    .await?;

    // Step 2: WAITING_APPROVAL step
    sqlx::query!(
        "INSERT INTO agent_steps (
            id, run_id, step_number, name, description, action_type, status,
            requires_approval, input_payload, started_at
         )
         VALUES ($1, $2, 2, 'Executive Approval for Automated Outreach Campaign', 'Gatekeeper check for multi-channel message dispatch', 'APPROVAL_GATEWAY', 'WAITING_APPROVAL', true, $3, NOW())",
        Uuid::new_v4(),
        run_id,
        serde_json::json!({ "recipientCount": 15, "channel": "WHATSAPP" })
    )
    .execute(&mut *tx)
    .await?;

    // Step 3: Pending dispatch
    sqlx::query!(
        "INSERT INTO agent_steps (
            id, run_id, step_number, name, description, action_type, status,
            requires_approval
         )
         VALUES ($1, $2, 3, 'Execute WhatsApp Template Dispatch', 'Dispatches approved templates to target audience', 'WHATSAPP_DISPATCH', 'PENDING', false)",
        Uuid::new_v4(),
        run_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    get_run(pool, workspace_id, run_id).await
}

pub async fn approve_step(
    pool: &PgPool,
    workspace_id: Uuid,
    user_id: Uuid,
    run_id: Uuid,
    step_id: Uuid,
    _dto: StepApprovalDto,
) -> Result<AgentRunWithSteps, AppError> {
    let _run = sqlx::query_as::<_, AgentRun>(
        "SELECT * FROM agent_runs WHERE id = $1 AND workspace_id = $2"
    )
    .bind(run_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Agent run not found in workspace".to_string()))?;

    let step = sqlx::query_as::<_, AgentStep>(
        "SELECT * FROM agent_steps WHERE id = $1 AND run_id = $2"
    )
    .bind(step_id)
    .bind(run_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Agent step not found".to_string()))?;

    if step.status != "WAITING_APPROVAL" {
        return Err(AppError::BadRequest(format!(
            "Step is not waiting for approval (Current status: {})",
            step.status
        )));
    }

    let mut tx = pool.begin().await?;

    sqlx::query!(
        "UPDATE agent_steps
         SET status = 'APPROVED',
             approved_by_id = $3,
             approved_at = NOW(),
             completed_at = NOW(),
             updated_at = NOW()
         WHERE id = $1 AND run_id = $2",
        step_id,
        run_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE agent_steps
         SET status = 'COMPLETED',
             started_at = NOW(),
             completed_at = NOW(),
             updated_at = NOW()
         WHERE run_id = $1 AND status = 'PENDING'",
        run_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE agent_runs
         SET status = 'COMPLETED',
             completed_at = NOW(),
             updated_at = NOW()
         WHERE id = $1",
        run_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    get_run(pool, workspace_id, run_id).await
}

pub async fn reject_step(
    pool: &PgPool,
    workspace_id: Uuid,
    _user_id: Uuid,
    run_id: Uuid,
    step_id: Uuid,
    dto: StepRejectionDto,
) -> Result<AgentRunWithSteps, AppError> {
    let _run = sqlx::query_as::<_, AgentRun>(
        "SELECT * FROM agent_runs WHERE id = $1 AND workspace_id = $2"
    )
    .bind(run_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Agent run not found".to_string()))?;

    let mut tx = pool.begin().await?;

    sqlx::query!(
        "UPDATE agent_steps
         SET status = 'REJECTED',
             rejection_reason = $3,
             completed_at = NOW(),
             updated_at = NOW()
         WHERE id = $1 AND run_id = $2",
        step_id,
        run_id,
        dto.reason
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE agent_runs
         SET status = 'CANCELLED',
             completed_at = NOW(),
             updated_at = NOW()
         WHERE id = $1",
        run_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    get_run(pool, workspace_id, run_id).await
}
