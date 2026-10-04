use axum::{
    routing::{get, post},
    Router,
};
use dockworker_control_plane::{api, config, db, registry};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env()?;
    if config.registry_only {
        return serve_registry_only(config).await;
    }

    let db = db::Database::connect(&config.database_url).await?;
    let db = Arc::new(db);

    // Start ephemeral GC worker (non-blocking)
    tokio::spawn(registry::gc::start_gc_worker(db.clone()));

    // Initialize registry service state
    let registry_state = Arc::new(registry::RegistryState::new(
        db.clone(),
        config
            .registry_storage_path
            .clone()
            .unwrap_or_else(|| "/var/lib/dockworker/blobs".to_string()),
        config.jwt_secret.clone(),
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
        // Registry routes (OCI Distribution v1.1).
        // nest_service matches both /v2 and /v2/. OCI clients ping the trailing-slash form.
        .nest_service("/v2", registry::routes().with_state(registry_state));

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!(
        "dockworker control plane listening on {}",
        config.listen_addr
    );
    tracing::info!("OCI registry available at /v2/");

    axum::serve(listener, app).await?;
    Ok(())
}

/// Listen for OCI push and pull without GitHub, Stripe, or Postgres.
async fn serve_registry_only(config: config::Config) -> anyhow::Result<()> {
    let storage_path = config
        .registry_storage_path
        .clone()
        .unwrap_or_else(|| "/var/lib/dockworker/blobs".to_string());
    let registry_state = Arc::new(registry::RegistryState::standalone(
        storage_path,
        config.jwt_secret.clone(),
    ));

    let app = Router::new()
        .route("/health", get(api::health))
        .nest_service("/v2", registry::routes().with_state(registry_state));

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!("dockworker registry listening on {}", config.listen_addr);
    tracing::info!("OCI registry available at /v2/");

    axum::serve(listener, app).await?;
    Ok(())
}
