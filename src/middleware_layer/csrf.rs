use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    http::Method,
};
use tower_cookies::Cookies;
use redis::AsyncCommands;

use crate::{error::AppError, state::AppState};

/// A middleware that verifies the CSRF token.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `cookies` - The request cookies.
/// * `req` - The incoming request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Response` or an error `AppError`.
pub async fn verify_csrf(
    State(mut state): State<AppState>,
    cookies: Cookies,
    req: Request<Body>,
    next: Next,
) -> Response {
    if req.method() == Method::GET
        || req.method() == Method::HEAD
        || req.method() == Method::OPTIONS
    {
        tracing::debug!("✅ CSRF exemption: {} request", req.method());
        return next.run(req).await;
    }

    let csrf_token_cookie = match cookies.get("csrf_token") {
        Some(c) => c.value().to_string(),
        None => {
            tracing::warn!("❌ CSRF: Cookie csrf_token não encontrado");
            return AppError::Authentication("Missing CSRF token cookie".to_string())
                .into_response();
        }
    };

    let headers = req.headers();
    let csrf_token_header = match headers
        .get("x-csrf-token")
        .or_else(|| headers.get("X-CSRF-Token"))
    {
        Some(token) => match token.to_str() {
            Ok(t) => t.to_string(),
            Err(_) => {
                tracing::warn!("❌ CSRF: Header com formato inválido");
                return AppError::Authentication("Invalid CSRF token format".to_string())
                    .into_response();
            }
        },
        None => {
            tracing::warn!("❌ CSRF: Header x-csrf-token não encontrado");
            return AppError::Authentication("Missing CSRF token header".to_string())
                .into_response();
        }
    };

    tracing::debug!("🔍 CSRF validation - Cookie: {}..., Header: {}...", 
        &csrf_token_cookie[..20.min(csrf_token_cookie.len())],
        &csrf_token_header[..20.min(csrf_token_header.len())]
    );

    if csrf_token_cookie != csrf_token_header {
        tracing::warn!("❌ CSRF: Tokens não conferem");
        return AppError::Authentication("CSRF token mismatch".to_string()).into_response();
    }

    let csrf_key = format!("csrf:{}", csrf_token_cookie);
    
    match state
        .redis
        .get::<_, Option<String>>(&csrf_key)
        .await
    {
        Ok(Some(_)) => {
            tracing::debug!("✅ CSRF token válido");
            next.run(req).await
        }
        Ok(None) => {
            tracing::warn!("❌ CSRF: Token expirado ou inválido");
            AppError::Authentication("CSRF token expired or invalid".to_string()).into_response()
        }
        Err(e) => {
            tracing::error!("❌ CSRF: Erro no Redis: {}", e);
            AppError::Authentication("CSRF validation error".to_string()).into_response()
        }
    }
}
