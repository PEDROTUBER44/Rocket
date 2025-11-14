use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use deadpool::managed::CreatePoolError;
use deadpool_postgres::ConfigError;
use thiserror::Error;

/// The application's error type.
#[derive(Error, Debug)]
pub enum AppError {
    /// A tokio-postgres error.
    #[error("Postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),

    /// A deadpool-postgres pool error.
    #[error("Database pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    /// A deadpool-postgres build error.
    #[error("Database pool build error: {0}")]
    PoolBuild(#[from] CreatePoolError<ConfigError>),

    /// A Redis error.
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// An I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// An authentication error.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// An authorization error.
    #[error("Authorization failed")]
    Unauthorized,

    /// A resource not found error.
    #[error("Resource not found")]
    NotFound,

    /// A validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// An encryption error.
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// A multipart error.
    #[error("Multipart error: {0}")]
    Multipart(String),

    /// An internal server error.
    #[error("Internal server error: {0}")]
    Internal(String),

    /// A rate limit exceeded error.
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
}

/// A `Result` type that uses `AppError` as the error type.
pub type Result<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Postgres(ref e) => {
                tracing::error!("Postgres error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }

            AppError::Pool(ref e) => {
                tracing::error!("Database pool error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database pool error".to_string())
            }

            AppError::PoolBuild(ref e) => {
                tracing::error!("Database pool build error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database pool build error".to_string())
            }

            AppError::Redis(ref e) => {
                tracing::error!("Redis error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Cache error".to_string())
            }

            AppError::Io(ref e) => {
                tracing::error!("IO error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "File system error".to_string())
            }

            AppError::Authentication(ref msg) => {
                tracing::warn!("Authentication failed: {}", msg);
                (StatusCode::UNAUTHORIZED, msg.clone())
            }

            AppError::Unauthorized => {
                tracing::warn!("Authorization failed");
                (StatusCode::FORBIDDEN, "Forbidden".to_string())
            }

            AppError::NotFound => {
                tracing::debug!("Resource not found");
                (StatusCode::NOT_FOUND, "Resource not found".to_string())
            }

            AppError::Validation(ref msg) => {
                tracing::debug!("Validation error: {}", msg);
                (StatusCode::BAD_REQUEST, msg.clone())
            }

            AppError::Encryption(ref msg) => {
                tracing::error!("Encryption error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Encryption error".to_string())
            }

            AppError::Multipart(ref msg) => {
                tracing::error!("Multipart error: {}", msg);
                (StatusCode::BAD_REQUEST, msg.clone())
            }

            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }

            AppError::RateLimitExceeded(ref msg) => {
                tracing::warn!("Rate limit exceeded: {}", msg);
                (StatusCode::TOO_MANY_REQUESTS, msg.clone())
            }
        };

        let body = sonic_rs::to_string(&sonic_rs::json!({
            "error": message
        }))
        .unwrap_or_else(|_| r#"{"error":"Internal server error"}"#.to_string());

        (status, body).into_response()
    }
}
