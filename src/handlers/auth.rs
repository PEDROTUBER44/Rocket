use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use tower_cookies::{Cookies, Cookie};
use tower_cookies::cookie::time::Duration;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::{
    error::{AppError, Result},
    models::session::Session,
    services::auth as auth_service,
    state::AppState,
    validation::auth::*,
};

use redis::AsyncCommands;

/// The request payload for user registration.
#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub name: String,
    pub username: String,
    pub password: String,
}

/// The request payload for user login.
#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// The request payload for changing a user's password.
#[derive(Deserialize, Debug)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// The response payload for authentication-related requests.
#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
}

/// Creates a secure cookie with the given name, value, and max age.
fn create_secure_cookie(name: String, value: String, max_age_days: i64) -> Cookie<'static> {
    let mut cookie = Cookie::new(name.clone(), value);

    let is_production = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "development".to_string()) == "production";

    if name != "csrf_token" {
        cookie.set_http_only(true);
    }

    if is_production {
        cookie.set_secure(true);
    }

    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    let duration_secs = max_age_days * 86400;
    cookie.set_max_age(Duration::seconds(duration_secs));
    cookie.set_path("/");

    cookie
}

/// Handles user registration.
#[axum::debug_handler]
pub async fn register(
    State(mut state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse> {
    tracing::info!("📝 Register attempt - Payload: {:?}", payload);
    validate_username(&payload.username)?;
    validate_password(&payload.password)?;
    
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Name cannot be empty".to_string()));
    }

    tracing::info!("✅ Validations passed for: {}", payload.username);
    
    let user = auth_service::create_user(
        &state.db,
        payload.name.clone(),
        payload.username.clone(),
        payload.password.clone(),
        &state.config.master_key,
    ).await?;

    tracing::info!("✅ User registered: {}", user.id);

    let session_id = Uuid::new_v4();
    tracing::debug!("🔑 Generated session_id: {}", session_id);

    let enc_dek = user
        .encrypted_dek
        .clone()
        .ok_or_else(|| AppError::Encryption("Missing encrypted DEK".to_string()))?;
    let dek_salt = user
        .dek_salt
        .clone()
        .ok_or_else(|| AppError::Encryption("Missing DEK salt".to_string()))?;
    let dek_secure = crate::crypto::dek::decrypt_user_dek(&enc_dek, &dek_salt, &payload.password)?;
    let session_dek: Vec<u8> = dek_secure.as_bytes().to_vec();

    let session = Session {
        user_id: user.id,
        dek: session_dek,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::days(state.config.session_duration_days),
    };

    let session_json = sonic_rs::to_string(&session)
        .map_err(|e| AppError::Internal(format!("Session serialization failed: {}", e)))?;

    let expiration_seconds: u64 = (state.config.session_duration_days * 86400) as u64;
    let _: () = state
        .redis
        .set_ex(
            format!("session:{}", session_id),
            &session_json,
            expiration_seconds,
        )
        .await
        .map_err(|e| {
            tracing::error!("❌ Redis set_ex failed: {}", e);
            AppError::Redis(e)
        })?;

    tracing::info!("✅ Session saved to Redis: session:{}", session_id);

    let session_cookie = create_secure_cookie(
        "session_id".to_string(),
        session_id.to_string(),
        state.config.session_duration_days,
    );
    cookies.add(session_cookie);
    tracing::info!("✅ Session cookie added: session_id={}", session_id);

    let csrf_token = crate::crypto::csrf::generate_csrf_token()?;
    tracing::debug!("🔐 Generated CSRF token: {}", &csrf_token[..20.min(csrf_token.len())]);

    let _: () = state
        .redis
        .set_ex(
            format!("csrf:{}", csrf_token),
            "valid",
            3600,
        )
        .await
        .map_err(|e| {
            tracing::error!("❌ Redis set_ex failed para CSRF: {}", e);
            AppError::Redis(e)
        })?;

    let csrf_cookie = create_secure_cookie(
        "csrf_token".to_string(),
        csrf_token,
        1,
    );
    cookies.add(csrf_cookie);
    tracing::info!("✅ CSRF cookie added");

    let response = AuthResponse {
        success: true,
        message: "Registration successful. Welcome!".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

/// Handles user login.
#[axum::debug_handler]
pub async fn login(
    State(mut state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<LoginRequest>,
) -> Result<Response> {
    tracing::info!("🔐 Login attempt - Payload: {:?}", payload);
    validate_username(&payload.username)?;

    let password_plain = payload.password.clone();

    let user = auth_service::authenticate_user(
        &state.db,
        payload.username.clone(),
        payload.password,
        &state.config.master_key,
    )
    .await?;

    let session_id = Uuid::new_v4();
    tracing::debug!("🔑 Generated session_id: {}", session_id);

    let enc_dek = user
        .encrypted_dek
        .clone()
        .ok_or_else(|| AppError::Encryption("Missing encrypted DEK".to_string()))?;
    let dek_salt = user
        .dek_salt
        .clone()
        .ok_or_else(|| AppError::Encryption("Missing DEK salt".to_string()))?;
    let dek_secure = crate::crypto::dek::decrypt_user_dek(&enc_dek, &dek_salt, &password_plain)?;
    let session_dek: Vec<u8> = dek_secure.as_bytes().to_vec();

    let session = Session {
        user_id: user.id,
        dek: session_dek,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::days(state.config.session_duration_days),
    };

    let session_json = sonic_rs::to_string(&session)
        .map_err(|e| AppError::Internal(format!("Session serialization failed: {}", e)))?;

    let expiration_seconds: u64 = (state.config.session_duration_days * 86400) as u64;
    let _: () = state
        .redis
        .set_ex(
            format!("session:{}", session_id),
            &session_json,
            expiration_seconds,
        )
        .await
        .map_err(|e| {
            tracing::error!("❌ Redis set_ex failed: {}", e);
            AppError::Redis(e)
        })?;

    tracing::info!("✅ Session saved to Redis: session:{}", session_id);

    let session_cookie = create_secure_cookie(
        "session_id".to_string(),
        session_id.to_string(),
        state.config.session_duration_days,
    );
    cookies.add(session_cookie);

    tracing::info!("✅ Session cookie added: session_id={}", session_id);

    let csrf_token = crate::crypto::csrf::generate_csrf_token()?;
    tracing::debug!("🔐 Generated CSRF token: {}", &csrf_token[..20.min(csrf_token.len())]);

    let _: () = state
        .redis
        .set_ex(
            format!("csrf:{}", csrf_token),
            "valid",
            3600,
        )
        .await
        .map_err(|e| {
            tracing::error!("❌ Redis set_ex failed para CSRF: {}", e);
            AppError::Redis(e)
        })?;

    tracing::info!("✅ CSRF token saved to Redis");

    let csrf_cookie = create_secure_cookie(
        "csrf_token".to_string(),
        csrf_token,
        1,
    );
    cookies.add(csrf_cookie);

    tracing::info!("✅ CSRF cookie added");
    tracing::info!("✅ User logged in: {}", user.id);

    let response = AuthResponse {
        success: true,
        message: "Login successful".to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// Handles user logout.
#[axum::debug_handler]
pub async fn logout(
    State(mut state): State<AppState>,
    Extension(session): Extension<Session>,
    cookies: Cookies,
) -> Result<Response> {
    tracing::info!("👋 Logout for user: {}", session.user_id);

    let session_id = cookies
        .get("session_id")
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized)?;

    let _: () = state
        .redis
        .del(format!("session:{}", session_id))
        .await?;

    tracing::info!("✅ Session deleted from Redis");

    if let Some(csrf_cookie) = cookies.get("csrf_token") {
        let csrf_token = csrf_cookie.value();
        let _: () = state
            .redis
            .del(format!("csrf:{}", csrf_token))
            .await
            .unwrap_or(());
        tracing::info!("✅ CSRF token deleted from Redis");
    }

    let mut session_cookie = Cookie::new("session_id", "");
    session_cookie.set_max_age(Duration::seconds(0));
    session_cookie.set_path("/");
    cookies.remove(session_cookie);

    let mut csrf_cookie = Cookie::new("csrf_token", "");
    csrf_cookie.set_max_age(Duration::seconds(0));
    csrf_cookie.set_path("/");
    cookies.remove(csrf_cookie);

    tracing::info!("✅ User logged out: {}", session.user_id);

    let response = AuthResponse {
        success: true,
        message: "Logout successful".to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// Handles changing a user's password.
#[axum::debug_handler]
pub async fn change_password(
    State(mut state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Response> {
    tracing::info!("🔑 Change password for user: {}", session.user_id);

    validate_password(&payload.new_password)?;

    auth_service::change_password(
        &state,
        session.user_id,
        payload.old_password,
        payload.new_password,
    )
    .await?;

    tracing::info!("✅ Password changed for user: {}", session.user_id);

    let response = AuthResponse {
        success: true,
        message: "Password changed successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}
