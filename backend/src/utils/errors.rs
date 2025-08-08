use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<HashMap<String, Vec<String>>>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub enum AppError {
    ValidationError(HashMap<String, Vec<String>>),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    BadRequest(String),
    UnsupportedMediaType(String),
    PayloadTooLarge(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message, details) = match &self {
            AppError::ValidationError(errors) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Validation failed".to_string(),
                Some(errors.clone()),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg.clone(),
                None,
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                msg.clone(),
                None,
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                msg.clone(),
                None,
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                msg.clone(),
                None,
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg.clone(),
                None,
            ),
            AppError::UnsupportedMediaType(msg) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                msg.clone(),
                None,
            ),
            AppError::PayloadTooLarge(msg) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "PAYLOAD_TOO_LARGE",
                msg.clone(),
                None,
            ),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                msg.clone(),
                None,
            ),
        };

        let error_response = ErrorResponse {
            error: error_type.to_string(),
            message,
            details,
            timestamp: Utc::now(),
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut error_map = HashMap::new();
        
        for (field, field_errors) in errors.field_errors() {
            let messages: Vec<String> = field_errors
                .iter()
                .map(|error| {
                    error.message
                        .as_ref()
                        .map(|msg| msg.to_string())
                        .unwrap_or_else(|| format!("Invalid value for field '{}'", field))
                })
                .collect();
            error_map.insert(field.to_string(), messages);
        }
        
        AppError::ValidationError(error_map)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    AppError::Conflict("Resource already exists".to_string())
                } else {
                    AppError::InternalServerError("Database error occurred".to_string())
                }
            }
            _ => AppError::InternalServerError("Database error occurred".to_string()),
        }
    }
}