// OCI Distribution v1.1 registry service for ephemeral free-tier builds
pub mod auth;
pub mod blobs;
pub mod manifests;
pub mod storage;
pub mod gc;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, head, patch, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::db::Database;

pub struct RegistryState {
    db: Arc<Database>,
    storage_path: String,
}

impl RegistryState {
    pub fn new(db: Arc<Database>, storage_path: String) -> Self {
        Self { db, storage_path }
    }
}

// OCI Distribution v1.1 ping endpoint
pub async fn v2_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            ("Docker-Distribution-API-Version", "registry/2.0"),
            ("Content-Length", "0"),
        ],
    )
}

pub fn routes() -> Router<Arc<RegistryState>> {
    Router::new()
        // Version check (OCI Distribution spec ping)
        .route("/", get(v2_check))

        // Blob upload endpoints
        .route("/:name/blobs/uploads/", post(blobs::initiate_upload))
        .route("/:name/blobs/uploads/:uuid", patch(blobs::patch_upload))
        .route("/:name/blobs/uploads/:uuid", put(blobs::complete_upload))

        // Blob fetch/exists
        .route("/:name/blobs/:digest", head(blobs::head_blob))
        .route("/:name/blobs/:digest", get(blobs::get_blob))

        // Manifest operations
        .route("/:name/manifests/:reference", put(manifests::put_manifest))
        .route("/:name/manifests/:reference", head(manifests::head_manifest))
        .route("/:name/manifests/:reference", get(manifests::get_manifest))
        .route("/:name/manifests/:reference", delete(manifests::delete_manifest))

        // Token endpoint for scoped auth
        .route("/token", post(auth::issue_token))

        .layer(CorsLayer::permissive())
}
