use crate::auth::AuthenticatedUser;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TenantContext {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub user_role: String,
    pub workspace_role: String,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for TenantContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let AuthenticatedUser(claims) = AuthenticatedUser::from_request_parts(parts, state).await?;

        // 1. Resolve workspace_id from x-workspace-id header, query parameter, or claims
        let mut target_workspace_id: Option<Uuid> = None;

        if let Some(h) = parts.headers.get("x-workspace-id").and_then(|v| v.to_str().ok()) {
            if let Ok(parsed) = Uuid::parse_str(h) {
                target_workspace_id = Some(parsed);
            }
        }

        if target_workspace_id.is_none() {
            if let Some(query) = parts.uri.query() {
                for pair in query.split('&') {
                    let mut kv = pair.split('=');
                    if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                        if k == "workspaceId" {
                            if let Ok(parsed) = Uuid::parse_str(v) {
                                target_workspace_id = Some(parsed);
                                break;
                            }
                        }
                    }
                }
            }
        }

        if target_workspace_id.is_none() {
            target_workspace_id = claims.workspace_id;
        }

        // 2. If still none, check user's primary membership
        let workspace_id = match target_workspace_id {
            Some(id) => id,
            None => {
                let first_member = sqlx::query!(
                    "SELECT workspace_id FROM workspace_members WHERE user_id = $1 ORDER BY joined_at ASC LIMIT 1",
                    claims.sub
                )
                .fetch_optional(&state.pool)
                .await
                .map_err(AppError::Database)?;

                match first_member {
                    Some(m) => m.workspace_id,
                    None => return Err(AppError::BadRequest("No workspace found for user. Provide x-workspace-id header.".to_string())),
                }
            }
        };

        // 3. Verify membership & retrieve workspace role
        let member_record = sqlx::query!(
            "SELECT role FROM workspace_members WHERE workspace_id = $1 AND user_id = $2",
            workspace_id,
            claims.sub
        )
        .fetch_optional(&state.pool)
        .await
        .map_err(AppError::Database)?;

        let workspace_role = if claims.role == "SYSTEM_ADMIN" {
            "OWNER".to_string()
        } else if let Some(m) = member_record {
            m.role
        } else {
            return Err(AppError::Forbidden("You do not belong to this workspace".to_string()));
        };

        Ok(TenantContext {
            workspace_id,
            user_id: claims.sub,
            user_role: claims.role,
            workspace_role,
        })
    }
}
