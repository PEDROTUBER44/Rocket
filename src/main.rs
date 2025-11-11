use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use axum::{
    Router,
    routing::{get, post, delete},
    middleware::from_fn_with_state,
    extract::DefaultBodyLimit,
};

use http::{Method, header};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_cookies::CookieManagerLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_http::{
    services::ServeDir,
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, DefaultOnFailure},
    cors::CorsLayer,
};

use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod state;
mod db;
mod crypto {
    pub mod aes;
    pub mod dek;
    pub mod kek;
    pub mod csrf;
}

mod models {
    pub mod user;
    pub mod session;
    pub mod file;
    pub mod folder;
}

mod repositories {
    pub mod user;
    pub mod file;
    pub mod folder;
}

mod services {
    pub mod auth;
    pub mod files;
    pub mod folders;
}

mod handlers {
    pub mod auth;
    pub mod files;
    pub mod folders;
}

mod middleware_layer {
    pub mod auth;
    pub mod csrf;
    pub mod rate_limit;
}

mod validation {
    pub mod auth;
}

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    tracing::info!("✅ Configuration loaded successfully");

    let state = AppState::new(&config).await?;
    tracing::info!("✅ AppState initialized with optimized pools");

    // Garantir que KEK v1 existe na startup
    match crypto::kek::ensure_kek_exists(
        &state.db,
        state.config.master_key.as_ref(),
        &state.kek_cache,
    )
    .await
    {
        Ok(version) => {
            tracing::info!("✅ KEK validation completed - version: {}", version);
        }
        Err(e) => {
            tracing::error!("❌ Failed to ensure KEK exists: {}", e);
            return Err(e.into());
        }
    }

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://[::1]:3000".parse().unwrap(),
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::COOKIE,
            "x-csrf-token".parse().unwrap(),
        ])
        .allow_credentials(true)
        .expose_headers(["x-csrf-token".parse().unwrap()])
        .max_age(Duration::from_secs(86400));

    let protected_governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10_000)
            .burst_size(50_000)
            .use_headers()
            .finish()
            .unwrap(),
    );

    let register_routes = Router::new()
        .route("/api/auth/register", post(handlers::auth::register))
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware_layer::rate_limit::rate_limit_register,
        ))
        .with_state(state.clone());

    let login_routes = Router::new()
        .route("/api/auth/login", post(handlers::auth::login))
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware_layer::rate_limit::rate_limit_login,
        ))
        .with_state(state.clone());

    let init_routes = Router::new()
        .route("/api/files/upload/init", post(handlers::files::init_upload))
        .route("/api/files/upload/chunk", post(handlers::files::upload_chunk))
        .route(
            "/api/files/upload/finalize",
            post(handlers::files::finalize_upload),
        )
        .route(
            "/api/files/upload/cancel",
            post(handlers::files::cancel_upload),
        )
        .route(
            "/api/files/recalculate-quota",
            post(handlers::files::recalculate_user_quota),
        )
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware_layer::auth::require_auth,
        ))
        .with_state(state.clone());

    let protected_routes = Router::new()
        .route("/api/auth/logout", post(handlers::auth::logout))
        .route(
            "/api/auth/change-password",
            post(handlers::auth::change_password),
        )
        .route(
            "/api/files/storage/info",
            get(handlers::files::storage_info),
        )
        .route("/api/files", get(handlers::files::list_files))
        .route("/api/files/{file_id}", get(handlers::files::download_file))
        .route("/api/files/{file_id}", delete(handlers::files::delete_file))
        .route(
            "/api/folders/list",
            get(handlers::folders::list_folder_contents),
        )
        .route(
            "/api/folders/{folder_id}",
            get(handlers::folders::get_folder_stats),
        )
        .route("/api/folders", post(handlers::folders::create_folder))
        .route(
            "/api/folders/{folder_id}",
            delete(handlers::folders::delete_folder),
        )
        .layer(tower_governor::GovernorLayer::new(
            protected_governor_conf.clone(),
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware_layer::csrf::verify_csrf,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware_layer::auth::require_auth,
        ))
        .with_state(state.clone());

    let app = Router::new()
        .merge(register_routes)
        .merge(login_routes)
        .merge(init_routes)
        .merge(protected_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true))
                .on_request(DefaultOnRequest::default().level(Level::DEBUG))
                .on_response(DefaultOnResponse::default().level(Level::DEBUG))
                .on_failure(DefaultOnFailure::default().level(Level::ERROR)),
        )
        .layer(CookieManagerLayer::new())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(cors)
        .fallback_service(ServeDir::new("files/public"));

    let cleanup_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            tracing::info!("🧹 Running scheduled cleanup of expired uploads...");
            match handlers::files::cleanup_expired_uploads(cleanup_state.clone()).await {
                Ok(_) => {
                    tracing::info!("✅ Cleanup job completed successfully");
                }
                Err(e) => {
                    tracing::error!("❌ Cleanup job failed: {}", e);
                }
            }
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 Server listening on http://{}", addr);
    tracing::info!("✅ Background cleanup job started (runs every hour)");
    tracing::info!("✅ All systems operational");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
