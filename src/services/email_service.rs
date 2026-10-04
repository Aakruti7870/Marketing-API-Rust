use crate::config::Config;
use crate::error::AppError;
use reqwest::redirect::Policy;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct ResendEmail<'a> {
    from: &'a str,
    to: [&'a str; 1],
    subject: &'a str,
    html: String,
}

pub async fn send_verification_email(
    config: &Config,
    recipient: &str,
    token: &str,
) -> Result<(), AppError> {
    let api_key = config.resend_api_key.as_deref().ok_or_else(|| {
        AppError::ExternalService("Email delivery is not configured on the server".to_string())
    })?;

    let base_url = config.frontend_base_url.trim_end_matches('/');
    let verify_url = format!("{base_url}/?verify_email_token={token}");
    let html = format!(
        "<div style=\"font-family:Arial,sans-serif;max-width:560px;margin:auto;color:#222\">\
         <h2>Verify your GOLD-e GrowthOS email</h2>\
         <p>Confirm this email address to activate your GrowthOS account.</p>\
         <p><a href=\"{verify_url}\" style=\"display:inline-block;padding:12px 20px;background:#6d43d8;color:#fff;text-decoration:none;border-radius:8px\">Verify email address</a></p>\
         <p>This link expires in 30 minutes and can only be used once.</p>\
         <p>If you did not create this account, you can ignore this message.</p></div>"
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(Policy::none())
        .build()
        .map_err(|_| AppError::ExternalService("Unable to initialize email delivery".to_string()))?;
    let response = client
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&ResendEmail {
            from: &config.email_from,
            to: [recipient],
            subject: "Verify your GOLD-e GrowthOS email",
            html,
        })
        .send()
        .await
        .map_err(|_| AppError::ExternalService("Email provider request failed".to_string()))?;

    if !response.status().is_success() {
        tracing::error!(status = %response.status(), "Resend rejected verification email");
        return Err(AppError::ExternalService(
            "Email provider rejected the verification message".to_string(),
        ));
    }
    Ok(())
}
