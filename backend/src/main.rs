mod api;
mod db;
mod models;
mod config;
mod auth;
mod quota;
mod runner_job;
mod runner_controller;
mod registry;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env()?;
    let db = db::Database::connect(&config.database_url).await?;
    let db = Arc::new(db);

    // Start ephemeral GC worker (non-blocking)
    tokio::spawn(registry::gc::start_gc_worker(db.clone()));

    // Initialize registry service state
    let registry_state = Arc::new(registry::RegistryState::new(
        db.clone(),
        config
            .registry_storage_path
            .unwrap_or_else(|| "/var/lib/dockworker/blobs".to_string()),
    ));

    let app = Router::new()
        // Health check
        .route("/health", get(api::health))

        // Auth
        .route("/auth/github/callback", get(api::auth::github_callback))
        .route("/auth/logout", post(api::auth::logout))

        // Build API
        .route("/builds", post(api::builds::trigger))
        .route("/builds/:build_id", get(api::builds::get_status))
        .route("/builds/:build_id/logs", get(api::builds::stream_logs))
        .route("/builds", get(api::builds::list_user_builds))

        // User
        .route("/user/quota", get(api::user::get_quota))
        .route("/user/profile", get(api::user::get_profile))

        // Admin
        .route("/admin/stats", get(api::admin::get_stats))

        .layer(CorsLayer::permissive())
        .with_state(db)
        // Registry routes (OCI Distribution v1.1)
        .nest("/v2", registry::routes().with_state(registry_state));

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!(
        "dockworker control plane listening on {}",
        config.listen_addr
    );
    tracing::info!("OCI registry available at /v2/");

    axum::serve(listener, app).await?;
    Ok(())
}
