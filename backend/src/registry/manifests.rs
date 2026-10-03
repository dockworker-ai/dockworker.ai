use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::Value;
use std::sync::Arc;

use crate::registry::RegistryState;

/// PUT /v2/<name>/manifests/<reference> - Store manifest or manifest list
pub async fn put_manifest(
    State(_registry): State<Arc<RegistryState>>,
    Path((_name, _reference)): Path<(String, String)>,
    Json(_body): Json<Value>,
) -> impl IntoResponse {
    // TODO: Compute SHA-256 digest of manifest JSON
    // TODO: Validate all referenced blob digests exist
    // TODO: Store manifest in database with tag/digest reference
    // TODO: Record creation timestamp for free-tier TTL purge

    StatusCode::CREATED
}

/// HEAD /v2/<name>/manifests/<reference> - Check manifest exists
pub async fn head_manifest(
    State(_registry): State<Arc<RegistryState>>,
    Path((_name, _reference)): Path<(String, String)>,
) -> impl IntoResponse {
    // TODO: Lookup manifest by tag or digest
    // TODO: Return Content-Type and Content-Length headers

    StatusCode::OK
}

/// GET /v2/<name>/manifests/<reference> - Fetch manifest
pub async fn get_manifest(
    State(_registry): State<Arc<RegistryState>>,
    Path((_name, _reference)): Path<(String, String)>,
) -> impl IntoResponse {
    // TODO: Lookup manifest by tag or digest
    // TODO: Return manifest JSON + Docker-Content-Digest header

    StatusCode::OK
}

/// DELETE /v2/<name>/manifests/<reference> - Delete manifest by digest
pub async fn delete_manifest(
    State(_registry): State<Arc<RegistryState>>,
    Path((_name, _reference)): Path<(String, String)>,
) -> impl IntoResponse {
    // TODO: Delete manifest by digest (required by OCI spec)
    // TODO: Mark unreferenced blobs for garbage collection

    StatusCode::ACCEPTED
}
