use crate::config::Config;
use crate::error::AppError;
use crate::models::{Contact, Message, SendMessageDto};
use crate::services::whatsapp_service::send_whatsapp_message;
use crate::utils::pagination::{PaginationMeta, PaginationQuery};
use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_messages(
    pool: &PgPool,
    workspace_id: Uuid,
    query: PaginationQuery,
    status: Option<String>,
) -> Result<(Vec<Message>, PaginationMeta), AppError> {
    let limit = query.limit();
    let offset = query.offset();
    let page = query.page();

    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages
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
        "SELECT COUNT(*) as total FROM messages
         WHERE workspace_id = $1
           AND ($2::text IS NULL OR status = $2)",
        workspace_id,
        status
    )
    .fetch_one(pool)
    .await?;

    let total = count_row.total.unwrap_or(0);
    let meta = PaginationMeta::new(total, page, limit);

    Ok((messages, meta))
}

pub async fn send_message(
    pool: &PgPool,
    http_client: &Client,
    config: &Config,
    workspace_id: Uuid,
    dto: SendMessageDto,
) -> Result<Message, AppError> {
    // 1. Fetch contact
    let contact = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE id = $1 AND workspace_id = $2"
    )
    .bind(dto.contact_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Contact not found".to_string()))?;

    // 2. Dispatch via WhatsApp service (with auto simulation fallback)
    let send_res = send_whatsapp_message(
        http_client,
        config,
        &contact.phone,
        &dto.content,
        dto.template_name.as_deref(),
    )
    .await?;

    let message_id = Uuid::new_v4();
    let template_data = dto.template_data.unwrap_or_else(|| serde_json::json!({}));

    // 3. Persist message record
    let message = sqlx::query_as::<_, Message>(
        "INSERT INTO messages (
            id, workspace_id, contact_id, channel, direction,
            external_message_id, status, content, template_name, template_data, sent_at
         )
         VALUES ($1, $2, $3, 'WHATSAPP', 'OUTBOUND', $4, $5, $6, $7, $8, NOW())
         RETURNING *"
    )
    .bind(message_id)
    .bind(workspace_id)
    .bind(dto.contact_id)
    .bind(&send_res.message_id)
    .bind(&send_res.status)
    .bind(&dto.content)
    .bind(&dto.template_name)
    .bind(&template_data)
    .fetch_one(pool)
    .await?;

    // 4. Update contact last contacted timestamp
    sqlx::query!(
        "UPDATE contacts SET last_contacted_at = NOW() WHERE id = $1",
        dto.contact_id
    )
    .execute(pool)
    .await?;

    Ok(message)
}

pub async fn get_message(
    pool: &PgPool,
    workspace_id: Uuid,
    message_id: Uuid,
) -> Result<Message, AppError> {
    let msg = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE id = $1 AND workspace_id = $2"
    )
    .bind(message_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Message not found".to_string()))?;

    Ok(msg)
}
