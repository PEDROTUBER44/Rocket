use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::session::Session,
    state::AppState,
};

use redis::AsyncCommands;

/// Extracts the session token from the request cookies.
///
/// # Arguments
///
/// * `cookies` - The request cookies.
///
/// # Returns
///
/// An `Option` containing the session ID if found.
fn extract_session_token(cookies: &Cookies) -> Option<Uuid> {
    cookies
        .get("session_id")
        .and_then(|cookie| Uuid::parse_str(cookie.value()).ok())
}

/// A middleware that requires a valid session to be present.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `cookies` - The request cookies.
/// * `request` - The incoming request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Response` or an error `StatusCode`.
pub async fn require_auth(
    State(mut state): State<AppState>,
    cookies: Cookies,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    tracing::debug!("🔐 Checking authentication...");
    
    let session_id = extract_session_token(&cookies)
        .ok_or_else(|| {
            tracing::warn!("❌ No session_id cookie found");
            StatusCode::FORBIDDEN
        })?;

    tracing::debug!("🔑 Found session_id: {}", session_id);

    let session_json: String = state
        .redis
        .get(format!("session:{}", session_id))
        .await
        .map_err(|e| {
            tracing::warn!("❌ Redis error or session not found: {}", e);
            StatusCode::FORBIDDEN
        })?;

    let session: Session = sonic_rs::from_str(&session_json)
        .map_err(|e| {
            tracing::warn!("❌ Invalid session JSON: {}", e);
            StatusCode::FORBIDDEN
        })?;

    if chrono::Utc::now() > session.expires_at {
        tracing::warn!("❌ Session expired for user: {}", session.user_id);
        
        let _: () = state
            .redis
            .del(format!("session:{}", session_id))
            .await
            .unwrap_or(());
        
        return Err(StatusCode::FORBIDDEN);
    }

    tracing::debug!("✅ User authenticated: {}", session.user_id);

    request.extensions_mut().insert(session);

    Ok(next.run(request).await)
}
