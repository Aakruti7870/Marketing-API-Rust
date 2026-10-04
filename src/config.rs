use axum::http::HeaderValue;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub environment: String,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub jwt_access_expiration_seconds: i64,
    pub jwt_refresh_expiration_seconds: i64,
    pub cors_origin: String,
    pub rate_limit_requests_per_minute: u64,
    pub whatsapp_simulation_mode: bool,
    pub whatsapp_api_version: String,
    pub whatsapp_phone_number_id: Option<String>,
    pub whatsapp_access_token: Option<String>,
    pub whatsapp_business_account_id: Option<String>,
    pub whatsapp_webhook_verify_token: String,
    pub whatsapp_app_secret: Option<String>,
    pub ai_api_key: Option<String>,
    pub ai_api_base_url: String,
    pub ai_model: String,
    pub ai_image_model: String,
    pub ai_temperature: f32,
    pub public_base_url: String,
    pub channel_encryption_key: Option<String>,
    pub resend_api_key: Option<String>,
    pub email_from: String,
    pub frontend_base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "4000".to_string())
            .parse::<u16>()
            .map_err(|_| "PORT must be a valid u16 number")?;

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://golde:password@localhost:5432/marketing_api?sslmode=disable".to_string()
        });

        let redis_url = env::var("REDIS_URL").ok().filter(|v| !v.trim().is_empty());

        let jwt_access_secret = env::var("JWT_ACCESS_SECRET")
            .unwrap_or_else(|_| "default_super_secret_access_key_123456789".to_string());

        let jwt_refresh_secret = env::var("JWT_REFRESH_SECRET")
            .unwrap_or_else(|_| "default_super_secret_refresh_key_987654321".to_string());

        if environment.eq_ignore_ascii_case("production") {
            if jwt_access_secret.len() < 32
                || jwt_access_secret == "default_super_secret_access_key_123456789"
            {
                return Err(
                    "JWT_ACCESS_SECRET must be configured with a secure value in production"
                        .to_string(),
                );
            }
            if jwt_refresh_secret.len() < 32
                || jwt_refresh_secret == "default_super_secret_refresh_key_987654321"
            {
                return Err(
                    "JWT_REFRESH_SECRET must be configured with a secure value in production"
                        .to_string(),
                );
            }
            if database_url.is_empty()
                || database_url
                    == "postgresql://golde:password@localhost:5432/marketing_api?sslmode=disable"
            {
                return Err(
                    "DATABASE_URL must be configured with a production database in production"
                        .to_string(),
                );
            }

            if let Some(redis) = redis_url.as_deref() {
                let has_scheme = redis.starts_with("redis://") || redis.starts_with("rediss://");
                let has_auth = redis
                    .split_once("://")
                    .and_then(|(_, rest)| rest.split_once('@'))
                    .map(|(credentials, _)| !credentials.is_empty())
                    .unwrap_or(false);
                if !has_scheme || !has_auth {
                    return Err(
                        "REDIS_URL, when set in production, must use redis:// or rediss:// and include AUTH credentials"
                            .to_string(),
                    );
                }
            }
        }

        let jwt_access_expiration_seconds = env::var("JWT_ACCESS_EXPIRATION_SECONDS")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<i64>()
            .unwrap_or(900); // 15 mins

        let jwt_refresh_expiration_seconds = env::var("JWT_REFRESH_EXPIRATION_SECONDS")
            .unwrap_or_else(|_| "604800".to_string())
            .parse::<i64>()
            .unwrap_or(604800); // 7 days

        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());
        let cors_origins: Vec<&str> = cors_origin.split(',').map(str::trim).collect();
        if cors_origin.trim() != "*"
            && (cors_origins.is_empty()
                || cors_origins
                    .iter()
                    .any(|origin| origin.is_empty() || HeaderValue::try_from(*origin).is_err()))
        {
            return Err("CORS_ORIGIN must be '*' for development or a comma-separated list of valid origins".to_string());
        }
        if environment.eq_ignore_ascii_case("production")
            && (cors_origin.trim() == "*"
                || cors_origins
                    .iter()
                    .any(|origin| !origin.starts_with("https://")))
        {
            return Err("Production CORS_ORIGIN must be a comma-separated allowlist of HTTPS origins; wildcard is forbidden".to_string());
        }

        let rate_limit_requests_per_minute = env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .unwrap_or_else(|_| "120".to_string())
            .parse::<u64>()
            .unwrap_or(120);

        let whatsapp_simulation_mode = env::var("WHATSAPP_SIMULATION_MODE")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            == "true";

        let whatsapp_api_version =
            env::var("WHATSAPP_API_VERSION").unwrap_or_else(|_| "v20.0".to_string());
        let whatsapp_phone_number_id = env::var("WHATSAPP_PHONE_NUMBER_ID").ok();
        let whatsapp_access_token = env::var("WHATSAPP_ACCESS_TOKEN").ok();
        let whatsapp_business_account_id = env::var("WHATSAPP_BUSINESS_ACCOUNT_ID").ok();
        let whatsapp_webhook_verify_token = env::var("WHATSAPP_WEBHOOK_VERIFY_TOKEN")
            .unwrap_or_else(|_| "growthos_secure_webhook_verify_token".to_string());
        let whatsapp_app_secret = env::var("WHATSAPP_APP_SECRET").ok();
        if environment.eq_ignore_ascii_case("production")
            && whatsapp_webhook_verify_token == "growthos_secure_webhook_verify_token"
        {
            return Err(
                "WHATSAPP_WEBHOOK_VERIFY_TOKEN must be set to a unique secret in production"
                    .to_string(),
            );
        }
        let ai_api_key = env::var("AI_API_KEY").ok();
        let ai_api_base_url =
            env::var("AI_API_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let ai_model = env::var("AI_MODEL").unwrap_or_else(|_| "gpt-5.6-luna".to_string());
        let ai_image_model =
            env::var("AI_IMAGE_MODEL").unwrap_or_else(|_| "gpt-image-2".to_string());
        let ai_temperature = env::var("AI_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.2);
        let public_base_url =
            env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "https://api.goldetech.com".to_string());
        let channel_encryption_key = env::var("CHANNEL_ENCRYPTION_KEY").ok();
        let resend_api_key = env::var("RESEND_API_KEY")
            .ok()
            .filter(|v| !v.trim().is_empty());
        let email_from = env::var("EMAIL_FROM")
            .unwrap_or_else(|_| "GOLD-e GrowthOS <noreply@kryso.dev>".to_string());
        let frontend_base_url =
            env::var("FRONTEND_BASE_URL").unwrap_or_else(|_| "https://goldetech.com".to_string());

        Ok(Self {
            port,
            host,
            environment,
            database_url,
            redis_url,
            jwt_access_secret,
            jwt_refresh_secret,
            jwt_access_expiration_seconds,
            jwt_refresh_expiration_seconds,
            cors_origin,
            rate_limit_requests_per_minute,
            whatsapp_simulation_mode,
            whatsapp_api_version,
            whatsapp_phone_number_id,
            whatsapp_access_token,
            whatsapp_business_account_id,
            whatsapp_webhook_verify_token,
            whatsapp_app_secret,
            ai_api_key,
            ai_api_base_url,
            ai_model,
            ai_image_model,
            ai_temperature,
            public_base_url,
            channel_encryption_key,
            resend_api_key,
            email_from,
            frontend_base_url,
        })
    }
}
