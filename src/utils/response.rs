use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use super::pagination::PaginationMeta;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

pub fn json_success<T: Serialize>(data: T, message: &str) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: Some(message.to_string()),
            data: Some(data),
        }),
    )
}

pub fn json_created<T: Serialize>(data: T, message: &str) -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: Some(message.to_string()),
            data: Some(data),
        }),
    )
}

pub fn json_paginated<T: Serialize>(data: Vec<T>, pagination: PaginationMeta, message: &str) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(PaginatedResponse {
            success: true,
            message: Some(message.to_string()),
            data,
            pagination,
        }),
    )
}
