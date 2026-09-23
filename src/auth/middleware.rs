use crate::auth::jwt::{verify_token, Claims};
use crate::error::AppError;
use crate::state::AppState;
use axum::{extract::FromRequestParts, http::request::Parts};

#[derive(Clone, Debug)]
pub struct AuthenticatedUser(pub Claims);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Malformed Authorization header".to_string()));
        }

        let token = &auth_header["Bearer ".len()..];
        let claims = verify_token(token, &state.config.jwt_access_secret)?;
        Ok(AuthenticatedUser(claims))
    }
}
