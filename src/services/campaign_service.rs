use crate::config::Config;
use crate::error::AppError;
use crate::models::{Campaign, Contact, CreateCampaignDto, UpdateCampaignDto};
use crate::services::whatsapp_service::send_whatsapp_message;
use crate::utils::pagination::{PaginationMeta, PaginationQuery};
use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_campaigns(
    pool: &PgPool,
    workspace_id: Uuid,
    query: PaginationQuery,
) -> Result<(Vec<Campaign>, PaginationMeta), AppError> {
    let limit = query.limit();
    let offset = query.offset();
    let page = query.page();

    let campaigns = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns
         WHERE workspace_id = $1
         ORDER BY created_at DESC
         LIMIT $2 OFFSET $3"
    )
    .bind(workspace_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count_row = sqlx::query!(
        "SELECT COUNT(*) as total FROM campaigns WHERE workspace_id = $1",
        workspace_id
    )
    .fetch_one(pool)
    .await?;

    let total = count_row.total.unwrap_or(0);
    let meta = PaginationMeta::new(total, page, limit);

    Ok((campaigns, meta))
}

pub async fn create_campaign(
    pool: &PgPool,
    workspace_id: Uuid,
    user_id: Uuid,
    dto: CreateCampaignDto,
) -> Result<Campaign, AppError> {
    let campaign_id = Uuid::new_v4();
    let channel = dto.channel.unwrap_or_else(|| "WHATSAPP".to_string());
    let target_tags = dto.target_tags.unwrap_or_default();
    let template_params = dto.template_params.unwrap_or_else(|| serde_json::json!({}));

    let campaign = sqlx::query_as::<_, Campaign>(
        "INSERT INTO campaigns (
            id, workspace_id, name, description, channel, status,
            target_tags, template_id, template_params, schedule_time, created_by_id
         )
         VALUES ($1, $2, $3, $4, $5, 'DRAFT', $6, $7, $8, $9, $10)
         RETURNING *"
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .bind(&dto.name)
    .bind(&dto.description)
    .bind(&channel)
    .bind(&target_tags)
    .bind(&dto.template_id)
    .bind(&template_params)
    .bind(dto.schedule_time)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(campaign)
}

pub async fn get_campaign(
    pool: &PgPool,
    workspace_id: Uuid,
    campaign_id: Uuid,
) -> Result<Campaign, AppError> {
    let campaign = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE id = $1 AND workspace_id = $2"
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

    Ok(campaign)
}

pub async fn update_campaign(
    pool: &PgPool,
    workspace_id: Uuid,
    campaign_id: Uuid,
    dto: UpdateCampaignDto,
) -> Result<Campaign, AppError> {
    let campaign = sqlx::query_as::<_, Campaign>(
        "UPDATE campaigns
         SET name = COALESCE($3, name),
             description = COALESCE($4, description),
             channel = COALESCE($5, channel),
             status = COALESCE($6, status),
             target_tags = COALESCE($7, target_tags),
             template_id = COALESCE($8, template_id),
             template_params = COALESCE($9, template_params),
             updated_at = NOW()
         WHERE id = $1 AND workspace_id = $2
         RETURNING *"
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .bind(dto.name)
    .bind(dto.description)
    .bind(dto.channel)
    .bind(dto.status)
    .bind(dto.target_tags)
    .bind(dto.template_id)
    .bind(dto.template_params)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

    Ok(campaign)
}

pub async fn launch_campaign(
    pool: &PgPool,
    http_client: &Client,
    config: &Config,
    workspace_id: Uuid,
    campaign_id: Uuid,
) -> Result<Campaign, AppError> {
    let campaign = get_campaign(pool, workspace_id, campaign_id).await?;

    // Find contacts matching target tags or all active contacts
    let contacts = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts
         WHERE workspace_id = $1 AND status = 'ACTIVE'
         ORDER BY created_at ASC
         LIMIT 50"
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;

    let message_body = format!("Notice from {}: Special opportunity available now.", campaign.name);

    for contact in contacts {
        if let Ok(res) = send_whatsapp_message(
            http_client,
            config,
            &contact.phone,
            &message_body,
            campaign.template_id.as_deref(),
        )
        .await
        {
            let _ = sqlx::query!(
                "INSERT INTO messages (\
                    id, workspace_id, campaign_id, contact_id, channel, direction,\
                    external_message_id, status, content, template_name, sent_at\
                 )\
                 VALUES ($1, $2, $3, $4, 'WHATSAPP', 'OUTBOUND', $5, $6, $7, $8, NOW())\
                 ON CONFLICT (external_message_id) DO NOTHING",
                Uuid::new_v4(),
                workspace_id,
                campaign.id,
                contact.id,
                res.message_id,
                res.status,
                message_body,
                campaign.template_id
            )
            .execute(pool)
            .await;
        }
    }

    let updated = sqlx::query_as::<_, Campaign>(
        "UPDATE campaigns SET status = 'RUNNING', updated_at = NOW() WHERE id = $1 AND workspace_id = $2 RETURNING *"
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .fetch_one(pool)
    .await?;

    Ok(updated)
}

pub async fn pause_campaign(
    pool: &PgPool,
    workspace_id: Uuid,
    campaign_id: Uuid,
) -> Result<Campaign, AppError> {
    let updated = sqlx::query_as::<_, Campaign>(
        "UPDATE campaigns SET status = 'PAUSED', updated_at = NOW() WHERE id = $1 AND workspace_id = $2 RETURNING *"
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

    Ok(updated)
}
