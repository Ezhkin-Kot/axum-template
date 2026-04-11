use std::time::SystemTimeError;

use axum::response::IntoResponse;
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("SystemTime error: {0}")]
    DataTime(#[from] SystemTimeError),
    #[error("Token expired")]
    Expired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Refresh token not found in database")]
    RefreshNotFound,
}

impl IntoResponse for TokenError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Database(er) => (StatusCode::INTERNAL_SERVER_ERROR, er.to_string()),
            Self::DataTime(er) => (StatusCode::INTERNAL_SERVER_ERROR, er.to_string()),
            Self::Jwt(er) => (StatusCode::UNAUTHORIZED, er.to_string()),
            Self::Expired => (
                StatusCode::UNAUTHORIZED,
                "Refresh token expired".to_string(),
            ),
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            Self::RefreshNotFound => (
                StatusCode::UNAUTHORIZED,
                "Refresh token not found".to_string(),
            ),
        }
        .into_response()
    }
}
