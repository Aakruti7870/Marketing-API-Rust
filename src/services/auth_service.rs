use crate::auth::jwt::{generate_access_token, AuthTokens};
use crate::auth::password::{hash_password, verify_password};
use crate::config::Config;
use crate::error::AppError;
use crate::models::{User, UserProfile};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterDto {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub workspace_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenDto {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    pub email: String,
    pub email_verification_required: bool,
    pub email_sent: bool,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailDto {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ResendVerificationDto {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserProfile,
    pub tokens: AuthTokens,
    pub workspace_id: Uuid,
}

pub async fn register(
    pool: &PgPool,
    config: &Config,
    dto: RegisterDto,
) -> Result<RegistrationResponse, AppError> {
    let email = dto.email.trim().to_lowercase();
    let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await?;
    if existing.is_some() {
        return Err(AppError::Conflict(
            "An account with this email already exists".to_string(),
        ));
    }

    let password_hash = hash_password(&dto.password)?;
    let user_id = Uuid::new_v4();
    let mut tx = pool.begin().await?;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, password_hash, first_name, last_name, phone, role, email_verified)
         VALUES ($1, $2, $3, $4, $5, $6, 'USER', false) RETURNING *",
    )
    .bind(user_id)
    .bind(&email)
    .bind(&password_hash)
    .bind(&dto.first_name)
    .bind(&dto.last_name)
    .bind(&dto.phone)
    .fetch_one(&mut *tx)
    .await?;

    let ws_name = dto
        .workspace_name
        .unwrap_or_else(|| format!("{}'s Workspace", dto.first_name));
    let ws_slug = format!(
        "{}-{}",
        ws_name.to_lowercase().replace(' ', "-"),
        &Uuid::new_v4().to_string()[..6]
    );
    let ws_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO workspaces (id, name, slug, description, owner_id) VALUES ($1, $2, $3, $4, $5)",
        ws_id, ws_name, ws_slug, "Primary workspace", user_id
    ).execute(&mut *tx).await?;

    sqlx::query!(
        "INSERT INTO workspace_members (id, workspace_id, user_id, role) VALUES ($1, $2, $3, 'OWNER')",
        Uuid::new_v4(), ws_id, user_id
    ).execute(&mut *tx).await?;

    let raw_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));
    let token_expires_at = Utc::now() + Duration::minutes(30);
    sqlx::query!(
        "INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(), user_id, token_hash, token_expires_at
    ).execute(&mut *tx).await?;

    tx.commit().await?;

    let email_sent = match crate::services::email_service::send_verification_email(
        config, &email, &raw_token,
    )
    .await
    {
        Ok(()) => true,
        Err(error) => {
            tracing::error!(user_id = %user_id, error = %error, "Verification email could not be sent during registration");
            false
        }
    };

    Ok(RegistrationResponse {
        email,
        email_verification_required: true,
        email_sent,
    })
}

pub async fn login(
    pool: &PgPool,
    config: &Config,
    dto: LoginDto,
) -> Result<AuthResponse, AppError> {
    let email = dto.email.trim().to_lowercase();
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !user.is_active {
        return Err(AppError::Forbidden("Account is deactivated".to_string()));
    }
    if !verify_password(&dto.password, &user.password_hash)? {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }
    let email_verified: bool = sqlx::query_scalar("SELECT email_verified FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(pool)
        .await?;
    if !email_verified {
        return Err(AppError::Forbidden(
            "Please verify your email before signing in.".to_string(),
        ));
    }

    let member_record = sqlx::query!(
        "SELECT workspace_id FROM workspace_members WHERE user_id = $1 ORDER BY joined_at ASC LIMIT 1",
        user.id
    ).fetch_optional(pool).await?;
    let ws_id = member_record
        .map(|m| m.workspace_id)
        .unwrap_or_else(Uuid::nil);

    let family = Uuid::new_v4();
    let refresh_token_string = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::seconds(config.jwt_refresh_expiration_seconds);
    sqlx::query!(
        "INSERT INTO refresh_tokens (id, token, user_id, family, expires_at) VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(), refresh_token_string, user.id, family, expires_at
    )
    .execute(pool)
    .await?;

    let access_token =
        generate_access_token(user.id, &user.email, &user.role, Some(ws_id), config)?;
    Ok(AuthResponse {
        user: user.into(),
        tokens: AuthTokens {
            access_token,
            refresh_token: refresh_token_string,
            token_type: "Bearer".to_string(),
            expires_in: config.jwt_access_expiration_seconds,
        },
        workspace_id: ws_id,
    })
}

pub async fn refresh(
    pool: &PgPool,
    config: &Config,
    dto: RefreshTokenDto,
) -> Result<AuthTokens, AppError> {
    let token_record = sqlx::query!(
        "SELECT * FROM refresh_tokens WHERE token = $1",
        dto.refresh_token
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    if token_record.is_revoked {
        sqlx::query!(
            "UPDATE refresh_tokens SET is_revoked = true WHERE family = $1",
            token_record.family
        )
        .execute(pool)
        .await?;
        return Err(AppError::Forbidden(
            "Compromised token detected. All sessions revoked.".to_string(),
        ));
    }
    if token_record.expires_at < Utc::now() {
        return Err(AppError::Unauthorized(
            "Refresh token has expired".to_string(),
        ));
    }

    sqlx::query!(
        "UPDATE refresh_tokens SET is_revoked = true WHERE id = $1",
        token_record.id
    )
    .execute(pool)
    .await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(token_record.user_id)
        .fetch_one(pool)
        .await?;

    let email_verified: bool = sqlx::query_scalar("SELECT email_verified FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(pool)
        .await?;
    if !email_verified {
        return Err(AppError::Forbidden(
            "Please verify your email before signing in.".to_string(),
        ));
    }

    let member_record = sqlx::query!(
        "SELECT workspace_id FROM workspace_members WHERE user_id = $1 ORDER BY joined_at ASC LIMIT 1",
        user.id
    ).fetch_optional(pool).await?;
    let ws_id = member_record.map(|m| m.workspace_id);

    let new_refresh_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::seconds(config.jwt_refresh_expiration_seconds);
    sqlx::query!(
        "INSERT INTO refresh_tokens (id, token, user_id, family, expires_at) VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(), new_refresh_token, user.id, token_record.family, expires_at
    ).execute(pool).await?;

    let access_token = generate_access_token(user.id, &user.email, &user.role, ws_id, config)?;
    Ok(AuthTokens {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt_access_expiration_seconds,
    })
}

pub async fn logout(pool: &PgPool, refresh_token: Option<String>) -> Result<(), AppError> {
    if let Some(token) = refresh_token {
        sqlx::query!(
            "UPDATE refresh_tokens SET is_revoked = true WHERE token = $1",
            token
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_me(pool: &PgPool, user_id: Uuid) -> Result<UserProfile, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(user.into())
}

pub async fn verify_email(pool: &PgPool, dto: VerifyEmailDto) -> Result<(), AppError> {
    let token_hash = hex::encode(Sha256::digest(dto.token.trim().as_bytes()));
    let mut tx = pool.begin().await?;
    let record = sqlx::query!(
        "UPDATE email_verification_tokens SET used_at = NOW()
         WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()
         RETURNING user_id",
        token_hash
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::BadRequest(
            "This verification link is invalid or expired. Request a new email.".to_string(),
        )
    })?;

    sqlx::query!(
        "UPDATE users SET email_verified = true, updated_at = NOW() WHERE id = $1",
        record.user_id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE email_verification_tokens SET used_at = COALESCE(used_at, NOW()) WHERE user_id = $1 AND used_at IS NULL",
        record.user_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn resend_verification(
    pool: &PgPool,
    config: &Config,
    dto: ResendVerificationDto,
) -> Result<(), AppError> {
    let email = dto.email.trim().to_lowercase();
    let user = sqlx::query!(
        "SELECT id, email_verified FROM users WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await?;
    // Same response for unknown, verified, and rate-limited addresses.
    let Some(user) = user else {
        return Ok(());
    };
    if user.email_verified {
        return Ok(());
    }

    let recently_sent: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM email_verification_tokens WHERE user_id = $1 AND created_at > NOW() - INTERVAL '60 seconds')"
    )
    .bind(user.id)
    .fetch_one(pool)
    .await?;
    if recently_sent {
        return Ok(());
    }

    let raw_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));
    let expires_at = Utc::now() + Duration::minutes(30);
    sqlx::query!(
        "INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(), user.id, token_hash, expires_at
    ).execute(pool).await?;

    if let Err(error) =
        crate::services::email_service::send_verification_email(config, &email, &raw_token).await
    {
        tracing::error!(user_id = %user.id, error = %error, "Verification resend email failed");
    }
    Ok(())
}
