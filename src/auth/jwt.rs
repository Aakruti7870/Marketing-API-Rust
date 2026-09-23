use crate::config::Config;
use crate::error::AppError;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,           // user_id
    pub email: String,
    pub role: String,
    pub workspace_id: Option<Uuid>,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

pub fn generate_access_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    workspace_id: Option<Uuid>,
    config: &Config,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    let exp = now + config.jwt_access_expiration_seconds as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        workspace_id,
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_access_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT creation error: {}", e)))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid authentication token: {}", e)))?;

    Ok(token_data.claims)
}
