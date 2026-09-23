use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(err) => {
                tracing::error!("Database query failed: {:?}", err);
                if let sqlx::Error::RowNotFound = err {
                    (StatusCode::NOT_FOUND, "Resource not found".to_string())
                } else if err.to_string().contains("duplicate key") || err.to_string().contains("unique constraint") {
                    (StatusCode::CONFLICT, "Resource conflict or duplicate key".to_string())
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database transaction failed".to_string())
                }
            }
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::ExternalService(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Internal(err) => {
                tracing::error!("Internal server error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected server error occurred".to_string())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
