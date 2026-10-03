use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::registry::RegistryState;

/// POST /v2/<name>/blobs/uploads/ - Initiate blob upload
pub async fn initiate_upload(
    State(registry): State<Arc<RegistryState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let upload_id = Uuid::new_v4().to_string();

    // TODO: Create staging upload entry in database
    // TODO: Create temporary upload directory

    let location = format!("/v2/{}/blobs/uploads/{}", name, upload_id);

    (
        StatusCode::ACCEPTED,
        [
            (header::LOCATION, location.as_str()),
            (header::DOCKER_UPLOAD_UUID, upload_id.as_str()),
            (header::RANGE, "0-0"),
        ],
    )
}

/// PATCH /v2/<name>/blobs/uploads/<uuid> - Append blob chunk
pub async fn patch_upload(
    State(_registry): State<Arc<RegistryState>>,
    Path((_name, _uuid)): Path<(String, String)>,
) -> impl IntoResponse {
    // TODO: Receive chunk, append to staging buffer
    // TODO: Verify Content-Range header
    // TODO: Return updated offset in Content-Range response

    StatusCode::ACCEPTED
}

/// PUT /v2/<name>/blobs/uploads/<uuid>?digest=<sha256:...> - Finalize upload
pub async fn complete_upload(
    State(registry): State<Arc<RegistryState>>,
    Path((_name, _uuid)): Path<(String, String)>,
) -> impl IntoResponse {
    // TODO: Verify digest matches final blob
    // TODO: Atomically move from /var/lib/dockworker/blobs/uploads/ to /var/lib/dockworker/blobs/sha256/
    // TODO: Record in database

    StatusCode::CREATED
}

/// HEAD /v2/<name>/blobs/<digest> - Check blob existence
pub async fn head_blob(
    State(_registry): State<Arc<RegistryState>>,
    Path((_name, _digest)): Path<(String, String)>,
) -> impl IntoResponse {
    // TODO: Check if blob exists in CAS store

    StatusCode::OK
}

/// GET /v2/<name>/blobs/<digest> - Download blob
pub async fn get_blob(
    State(_registry): State<Arc<RegistryState>>,
    Path((_name, _digest)): Path<(String, String)>,
) -> impl IntoResponse {
    // TODO: Stream blob from CAS store

    StatusCode::OK
}
