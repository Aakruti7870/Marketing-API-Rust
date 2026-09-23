use crate::error::AppError;
use crate::models::{Workspace, WorkspaceMemberDetail};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceDto {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberDto {
    pub email: String,
    pub role: Option<String>,
}

pub async fn list_user_workspaces(pool: &PgPool, user_id: Uuid) -> Result<Vec<Workspace>, AppError> {
    let workspaces = sqlx::query_as::<_, Workspace>(
        "SELECT w.* FROM workspaces w
         JOIN workspace_members wm ON w.id = wm.workspace_id
         WHERE wm.user_id = $1
         ORDER BY wm.joined_at ASC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(workspaces)
}

pub async fn create_workspace(
    pool: &PgPool,
    user_id: Uuid,
    dto: CreateWorkspaceDto,
) -> Result<Workspace, AppError> {
    let ws_id = Uuid::new_v4();
    let ws_slug = format!("{}-{}", dto.name.to_lowercase().replace(' ', "-"), &Uuid::new_v4().to_string()[..6]);

    let mut tx = pool.begin().await?;

    let workspace = sqlx::query_as::<_, Workspace>(
        "INSERT INTO workspaces (id, name, slug, description, owner_id)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *"
    )
    .bind(ws_id)
    .bind(&dto.name)
    .bind(&ws_slug)
    .bind(&dto.description)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO workspace_members (id, workspace_id, user_id, role)
         VALUES ($1, $2, $3, 'OWNER')",
        Uuid::new_v4(),
        ws_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(workspace)
}

pub async fn get_workspace(pool: &PgPool, workspace_id: Uuid) -> Result<Workspace, AppError> {
    let workspace = sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = $1")
        .bind(workspace_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    Ok(workspace)
}

pub async fn update_workspace(
    pool: &PgPool,
    workspace_id: Uuid,
    dto: UpdateWorkspaceDto,
) -> Result<Workspace, AppError> {
    let workspace = sqlx::query_as::<_, Workspace>(
        "UPDATE workspaces
         SET name = COALESCE($2, name),
             description = COALESCE($3, description),
             updated_at = NOW()
         WHERE id = $1
         RETURNING *"
    )
    .bind(workspace_id)
    .bind(dto.name)
    .bind(dto.description)
    .fetch_one(pool)
    .await?;

    Ok(workspace)
}

pub async fn delete_workspace(pool: &PgPool, workspace_id: Uuid) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM workspaces WHERE id = $1", workspace_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_members(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<WorkspaceMemberDetail>, AppError> {
    let rows = sqlx::query!(
        "SELECT wm.id, wm.workspace_id, wm.user_id, u.email, u.first_name, u.last_name, wm.role, wm.joined_at
         FROM workspace_members wm
         JOIN users u ON wm.user_id = u.id
         WHERE wm.workspace_id = $1
         ORDER BY wm.joined_at ASC",
        workspace_id
    )
    .fetch_all(pool)
    .await?;

    let members = rows
        .into_iter()
        .map(|r| WorkspaceMemberDetail {
            id: r.id,
            workspace_id: r.workspace_id,
            user_id: r.user_id,
            email: r.email,
            first_name: r.first_name,
            last_name: r.last_name,
            role: r.role,
            joined_at: r.joined_at,
        })
        .collect();

    Ok(members)
}

pub async fn add_member(
    pool: &PgPool,
    workspace_id: Uuid,
    dto: AddMemberDto,
) -> Result<WorkspaceMemberDetail, AppError> {
    let email = dto.email.trim().to_lowercase();
    let user = sqlx::query!("SELECT id, first_name, last_name FROM users WHERE email = $1", email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User with this email not registered in system".to_string()))?;

    let role = dto.role.unwrap_or_else(|| "MEMBER".to_string());
    let member_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO workspace_members (id, workspace_id, user_id, role)
         VALUES ($1, $2, $3, $4)",
        member_id,
        workspace_id,
        user.id,
        role
    )
    .execute(pool)
    .await?;

    Ok(WorkspaceMemberDetail {
        id: member_id,
        workspace_id,
        user_id: user.id,
        email,
        first_name: user.first_name,
        last_name: user.last_name,
        role,
        joined_at: chrono::Utc::now(),
    })
}

pub async fn remove_member(
    pool: &PgPool,
    workspace_id: Uuid,
    member_id: Uuid,
) -> Result<(), AppError> {
    let member = sqlx::query!("SELECT role FROM workspace_members WHERE id = $1 AND workspace_id = $2", member_id, workspace_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Member not found in workspace".to_string()))?;

    if member.role == "OWNER" {
        return Err(AppError::BadRequest("Cannot remove workspace owner".to_string()));
    }

    sqlx::query!("DELETE FROM workspace_members WHERE id = $1 AND workspace_id = $2", member_id, workspace_id)
        .execute(pool)
        .await?;

    Ok(())
}
