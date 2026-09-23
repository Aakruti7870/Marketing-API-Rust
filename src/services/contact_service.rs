use crate::error::AppError;
use crate::models::{Contact, CreateContactDto, UpdateContactDto};
use crate::utils::pagination::{PaginationMeta, PaginationQuery};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_contacts(
    pool: &PgPool,
    workspace_id: Uuid,
    query: PaginationQuery,
    search: Option<String>,
) -> Result<(Vec<Contact>, PaginationMeta), AppError> {
    let limit = query.limit();
    let offset = query.offset();
    let page = query.page();

    let search_pattern = search.map(|s| format!("%{}%", s));

    let contacts = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts
         WHERE workspace_id = $1
           AND ($2::text IS NULL OR first_name ILIKE $2 OR last_name ILIKE $2 OR company ILIKE $2 OR phone ILIKE $2)
         ORDER BY created_at DESC
         LIMIT $3 OFFSET $4"
    )
    .bind(workspace_id)
    .bind(&search_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count_row = sqlx::query!(
        "SELECT COUNT(*) as total FROM contacts
         WHERE workspace_id = $1
           AND ($2::text IS NULL OR first_name ILIKE $2 OR last_name ILIKE $2 OR company ILIKE $2 OR phone ILIKE $2)",
        workspace_id,
        search_pattern
    )
    .fetch_one(pool)
    .await?;

    let total = count_row.total.unwrap_or(0);
    let meta = PaginationMeta::new(total, page, limit);

    Ok((contacts, meta))
}

pub async fn get_contact(
    pool: &PgPool,
    workspace_id: Uuid,
    contact_id: Uuid,
) -> Result<Contact, AppError> {
    let contact = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE id = $1 AND workspace_id = $2"
    )
    .bind(contact_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Contact not found".to_string()))?;

    Ok(contact)
}

pub async fn create_contact(
    pool: &PgPool,
    workspace_id: Uuid,
    dto: CreateContactDto,
) -> Result<Contact, AppError> {
    let contact_id = Uuid::new_v4();
    let tags = dto.tags.unwrap_or_default();
    let status = dto.status.unwrap_or_else(|| "ACTIVE".to_string());
    let custom_fields = dto.custom_fields.unwrap_or_else(|| serde_json::json!({}));

    let contact = sqlx::query_as::<_, Contact>(
        "INSERT INTO contacts (id, workspace_id, first_name, last_name, email, phone, company, title, tags, status, custom_fields)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING *"
    )
    .bind(contact_id)
    .bind(workspace_id)
    .bind(&dto.first_name)
    .bind(&dto.last_name)
    .bind(&dto.email)
    .bind(&dto.phone)
    .bind(&dto.company)
    .bind(&dto.title)
    .bind(&tags)
    .bind(&status)
    .bind(&custom_fields)
    .fetch_one(pool)
    .await?;

    Ok(contact)
}

pub async fn update_contact(
    pool: &PgPool,
    workspace_id: Uuid,
    contact_id: Uuid,
    dto: UpdateContactDto,
) -> Result<Contact, AppError> {
    let contact = sqlx::query_as::<_, Contact>(
        "UPDATE contacts
         SET first_name = COALESCE($3, first_name),
             last_name = COALESCE($4, last_name),
             email = COALESCE($5, email),
             phone = COALESCE($6, phone),
             company = COALESCE($7, company),
             title = COALESCE($8, title),
             tags = COALESCE($9, tags),
             status = COALESCE($10, status),
             custom_fields = COALESCE($11, custom_fields),
             updated_at = NOW()
         WHERE id = $1 AND workspace_id = $2
         RETURNING *"
    )
    .bind(contact_id)
    .bind(workspace_id)
    .bind(dto.first_name)
    .bind(dto.last_name)
    .bind(dto.email)
    .bind(dto.phone)
    .bind(dto.company)
    .bind(dto.title)
    .bind(dto.tags)
    .bind(dto.status)
    .bind(dto.custom_fields)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Contact not found".to_string()))?;

    Ok(contact)
}

pub async fn delete_contact(
    pool: &PgPool,
    workspace_id: Uuid,
    contact_id: Uuid,
) -> Result<(), AppError> {
    let res = sqlx::query!(
        "DELETE FROM contacts WHERE id = $1 AND workspace_id = $2",
        contact_id,
        workspace_id
    )
    .execute(pool)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Contact not found".to_string()));
    }

    Ok(())
}
