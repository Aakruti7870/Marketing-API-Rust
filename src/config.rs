use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub environment: String,
    pub database_url: String,
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

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://golde:password@localhost:5432/marketing_api?sslmode=disable".to_string());

        let jwt_access_secret = env::var("JWT_ACCESS_SECRET")
            .unwrap_or_else(|_| "default_super_secret_access_key_123456789".to_string());

        let jwt_refresh_secret = env::var("JWT_REFRESH_SECRET")
            .unwrap_or_else(|_| "default_super_secret_refresh_key_987654321".to_string());

        let jwt_access_expiration_seconds = env::var("JWT_ACCESS_EXPIRATION_SECONDS")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<i64>()
            .unwrap_or(900); // 15 mins

        let jwt_refresh_expiration_seconds = env::var("JWT_REFRESH_EXPIRATION_SECONDS")
            .unwrap_or_else(|_| "604800".to_string())
            .parse::<i64>()
            .unwrap_or(604800); // 7 days

        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());

        let rate_limit_requests_per_minute = env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .unwrap_or_else(|_| "120".to_string())
            .parse::<u64>()
            .unwrap_or(120);

        let whatsapp_simulation_mode = env::var("WHATSAPP_SIMULATION_MODE")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            == "true";

        let whatsapp_api_version = env::var("WHATSAPP_API_VERSION").unwrap_or_else(|_| "v20.0".to_string());
        let whatsapp_phone_number_id = env::var("WHATSAPP_PHONE_NUMBER_ID").ok();
        let whatsapp_access_token = env::var("WHATSAPP_ACCESS_TOKEN").ok();
        let whatsapp_business_account_id = env::var("WHATSAPP_BUSINESS_ACCOUNT_ID").ok();
        let whatsapp_webhook_verify_token = env::var("WHATSAPP_WEBHOOK_VERIFY_TOKEN")
            .unwrap_or_else(|_| "growthos_secure_webhook_verify_token".to_string());
        let whatsapp_app_secret = env::var("WHATSAPP_APP_SECRET").ok();

        Ok(Self {
            port,
            host,
            environment,
            database_url,
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
        })
    }
}
