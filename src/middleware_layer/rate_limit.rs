use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use sonic_rs::JsonValueTrait;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::session::Session,
    state::AppState,
};

/// Extracts the real IP address from the request extensions.
///
/// # Arguments
///
/// * `req` - The incoming request.
///
/// # Returns
///
/// The IP address as a string, or "unknown" if not found.
fn extract_real_ip(req: &Request<Body>) -> String {
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// A middleware that rate limits user registration.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `req` - The incoming request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Response` or an error `AppError`.
pub async fn rate_limit_register(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_real_ip(&req);
    let key = format!("rate_limit:register:{}", ip);
    
    let count: Option<i32> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut state.redis.clone())
        .await
        .unwrap_or(None);

    if let Some(attempts) = count {
        if attempts >= 2 {
            let ttl: Option<i32> = redis::cmd("TTL")
                .arg(&key)
                .query_async(&mut state.redis.clone())
                .await
                .unwrap_or(None);

            return AppError::RateLimitExceeded(format!(
                "Registration limit exceeded. Try again in {} minutes",
                ttl.unwrap_or(0) / 60
            )).into_response();
        }
    }

    let _: () = redis::cmd("INCR")
        .arg(&key)
        .query_async(&mut state.redis.clone())
        .await
        .unwrap_or(());

    let _: () = redis::cmd("EXPIRE")
        .arg(&key)
        .arg(43200)
        .query_async(&mut state.redis.clone())
        .await
        .unwrap_or(());

    next.run(req).await
}

/// A middleware that rate limits user login attempts.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `req` - The incoming request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Response` or an error `AppError`.
pub async fn rate_limit_login(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    async fn extract_username_from_body(body_bytes: &[u8]) -> Option<String> {
        if let Ok(json) = sonic_rs::from_slice::<sonic_rs::Value>(body_bytes) {
            json.get("username")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();

    let username = extract_username_from_body(&body_bytes)
        .await
        .unwrap_or_else(|| "unknown".to_string());

    let key = format!("rate_limit:login:{}", username);
    
    let count: Option<i32> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut state.redis.clone())
        .await
        .unwrap_or(None);

    if let Some(attempts) = count {
        if attempts >= 5 {
            let ttl: Option<i32> = redis::cmd("TTL")
                .arg(&key)
                .query_async(&mut state.redis.clone())
                .await
                .unwrap_or(None);

            return AppError::Authentication(format!(
                "Too many failed login attempts. Try again in {} minutes",
                ttl.unwrap_or(0) / 60
            )).into_response();
        }
    }

    let new_body = Body::from(body_bytes.clone());
    let new_req = Request::from_parts(parts, new_body);
    
    let response = next.run(new_req).await;

    if response.status().is_client_error() {
        let _: () = redis::cmd("INCR")
            .arg(&key)
            .query_async(&mut state.redis.clone())
            .await
            .unwrap_or(());

        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(43200)
            .query_async(&mut state.redis.clone())
            .await
            .unwrap_or(());
    } else if response.status().is_success() {
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut state.redis.clone())
            .await
            .unwrap_or(());
    }

    response
}

/// A middleware that rate limits password change attempts.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `session` - The user's session.
/// * `req` - The incoming request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Response` or an error `AppError`.
pub async fn rate_limit_change_password(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let user_id = session.user_id;
    let key = format!("rate_limit:change_password:{}", user_id);
    
    let count: Option<i32> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut state.redis.clone())
        .await
        .unwrap_or(None);

    if let Some(attempts) = count {
        if attempts >= 2 {
            let ttl: Option<i32> = redis::cmd("TTL")
                .arg(&key)
                .query_async(&mut state.redis.clone())
                .await
                .unwrap_or(None);

            return AppError::RateLimitExceeded(format!(
                "Password change limit exceeded. Try again in {} hours",
                ttl.unwrap_or(0) / 3600
            )).into_response();
        }
    }

    let response = next.run(req).await;

    if response.status().is_success() {
        let _: () = redis::cmd("INCR")
            .arg(&key)
            .query_async(&mut state.redis.clone())
            .await
            .unwrap_or(());

        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(86400)
            .query_async(&mut state.redis.clone())
            .await
            .unwrap_or(());
    }

    response
}

/// A middleware that checks if the user has enough storage quota.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `session` - The user's session.
/// * `req` - The incoming request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Response` or an error `AppError`.
pub async fn check_storage_quota(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let user_id = session.user_id;
    
    let storage_info = sqlx::query!(
        r#"
        SELECT storage_quota_bytes, storage_used_bytes
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match storage_info {
        Ok(Some(info)) => {
            let available = info.storage_quota_bytes - info.storage_used_bytes;
            
            if available <= 0 {
                return AppError::Validation(
                    "Storage quota exceeded. Cannot upload more files.".to_string()
                ).into_response();
            }
            
            next.run(req).await
        }
        Ok(None) => {
            AppError::Unauthorized.into_response()
        }
        Err(e) => {
            tracing::error!("Error checking storage quota: {}", e);
            AppError::Internal("Failed to check storage quota".to_string()).into_response()
        }
    }
}

/// A middleware that rate limits file downloads.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `session` - The user's session.
/// * `req` - The incoming request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Response` or an error `AppError`.
pub async fn rate_limit_file_download(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let user_id = session.user_id;
    
    let path = req.uri().path();
    let file_id_str = path.split('/').last().unwrap_or("");
    
    let file_id = match Uuid::parse_str(file_id_str) {
        Ok(id) => id,
        Err(_) => return next.run(req).await,
    };

    let file_owner = sqlx::query!(
        r#"
        SELECT user_id, is_deleted
        FROM files
        WHERE id = $1
        "#,
        file_id
    )
    .fetch_optional(&state.db)
    .await;

    match file_owner {
        Ok(Some(file)) => {
            if file.is_deleted {
                return AppError::NotFound.into_response();
            }
            
            if file.user_id != user_id {
                return AppError::Unauthorized.into_response();
            }

            let key = format!("rate_limit:download:{}:{}", user_id, file_id);
            
            let count: Option<i32> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut state.redis.clone())
                .await
                .unwrap_or(None);

            if let Some(downloads) = count {
                if downloads >= 3 {
                    let ttl: Option<i32> = redis::cmd("TTL")
                        .arg(&key)
                        .query_async(&mut state.redis.clone())
                        .await
                        .unwrap_or(None);

                    return AppError::RateLimitExceeded(format!(
                        "Download limit exceeded for this file. Try again in {} hours",
                        ttl.unwrap_or(0) / 3600
                    )).into_response();
                }
            }

            let response = next.run(req).await;

            if response.status().is_success() {
                let _: () = redis::cmd("INCR")
                    .arg(&key)
                    .query_async(&mut state.redis.clone())
                    .await
                    .unwrap_or(());

                let _: () = redis::cmd("EXPIRE")
                    .arg(&key)
                    .arg(86400)
                    .query_async(&mut state.redis.clone())
                    .await
                    .unwrap_or(());
            }

            response
        }
        Ok(None) => AppError::NotFound.into_response(),
        Err(e) => {
            tracing::error!("Error checking file ownership: {}", e);
            AppError::Internal("Failed to verify file ownership".to_string()).into_response()
        }
    }
}

/// A middleware that verifies that the current user owns the requested file.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `session` - The user's session.
/// * `req` - The incoming request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Response` or an error `AppError`.
pub async fn verify_file_ownership(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let user_id = session.user_id;
    
    let path = req.uri().path();
    let file_id_str = path.split('/').last().unwrap_or("");
    
    let file_id = match Uuid::parse_str(file_id_str) {
        Ok(id) => id,
        Err(_) => return next.run(req).await,
    };

    let file_owner = sqlx::query!(
        r#"
        SELECT user_id, is_deleted
        FROM files
        WHERE id = $1
        "#,
        file_id
    )
    .fetch_optional(&state.db)
    .await;

    match file_owner {
        Ok(Some(file)) => {
            if file.is_deleted {
                return AppError::NotFound.into_response();
            }
            
            if file.user_id != user_id {
                return AppError::Unauthorized.into_response();
            }

            next.run(req).await
        }
        Ok(None) => AppError::NotFound.into_response(),
        Err(e) => {
            tracing::error!("Error checking file ownership: {}", e);
            AppError::Internal("Failed to verify file ownership".to_string()).into_response()
        }
    }
}
