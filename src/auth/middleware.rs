use crate::auth::jwt::{verify_token, Claims};
use crate::error::AppError;
use crate::state::AppState;
use axum::{extract::FromRequestParts, http::request::Parts};

#[derive(Clone, Debug)]
pub struct AuthenticatedUser(pub Claims);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized(
                "Malformed Authorization header".to_string(),
            ));
        }

        let token = &auth_header["Bearer ".len()..];
        let claims = verify_token(token, &state.config.jwt_access_secret)?;

        // Enforce verification centrally for every protected route. Checking only
        // during login/refresh is insufficient: a token issued earlier (or a
        // stale session) must not access workspace creation or other APIs until
        // the account's email is verified.
        let account = sqlx::query_as::<_, (bool, bool)>(
            "SELECT is_active, email_verified FROM users WHERE id = $1",
        )
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(AppError::Database)?;

        let (is_active, email_verified) = account
            .ok_or_else(|| AppError::Unauthorized("Invalid authentication token".to_string()))?;

        if !is_active {
            return Err(AppError::Unauthorized(
                "Invalid authentication token".to_string(),
            ));
        }

        if !email_verified {
            return Err(AppError::Forbidden(
                "Please verify your email before accessing this resource.".to_string(),
            ));
        }

        Ok(AuthenticatedUser(claims))
    }
}
