use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

use crate::registry::storage::StorageError;
use crate::registry::{oci_error, RegistryState};

/// PUT /v2/<name>/manifests/<reference>
pub async fn put_manifest(
    registry: &RegistryState,
    name: &str,
    reference: &str,
    media_type: &str,
    body: &[u8],
) -> Response {
    match registry
        .storage
        .put_manifest(name, reference, media_type, body)
        .await
    {
        Ok(digest) => {
            let location = format!("/v2/{name}/manifests/{reference}");
            let response = Response::builder()
                .status(StatusCode::CREATED)
                .header("docker-content-digest", &digest)
                .header(header::LOCATION, &location)
                .header(header::CONTENT_TYPE, media_type_or_default(media_type));
            response
                .body(axum::body::Body::empty())
                .unwrap_or_else(|_| {
                    oci_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "UNKNOWN",
                        "response failed",
                    )
                })
        }
        Err(StorageError::DigestMismatch { .. }) => oci_error(
            StatusCode::BAD_REQUEST,
            "DIGEST_INVALID",
            "manifest digest does not match content",
        ),
        Err(StorageError::Invalid(message)) => {
            let code = if message.contains("tag") {
                "TAG_INVALID"
            } else {
                "NAME_INVALID"
            };
            oci_error(StatusCode::BAD_REQUEST, code, &message)
        }
        Err(err) => map_manifest_error(err),
    }
}

/// HEAD /v2/<name>/manifests/<reference>
pub async fn head_manifest(registry: &RegistryState, name: &str, reference: &str) -> Response {
    manifest_response(registry, name, reference, false).await
}

/// GET /v2/<name>/manifests/<reference>
pub async fn get_manifest(registry: &RegistryState, name: &str, reference: &str) -> Response {
    manifest_response(registry, name, reference, true).await
}

/// DELETE /v2/<name>/manifests/<reference>
pub async fn delete_manifest(registry: &RegistryState, name: &str, reference: &str) -> Response {
    match registry.storage.delete_manifest(name, reference).await {
        Ok(()) => StatusCode::ACCEPTED.into_response(),
        Err(err) => map_manifest_error(err),
    }
}

async fn manifest_response(
    registry: &RegistryState,
    name: &str,
    reference: &str,
    include_body: bool,
) -> Response {
    match registry.storage.get_manifest(name, reference).await {
        Ok(record) => {
            let len = record.bytes.len();
            let builder = Response::builder()
                .status(StatusCode::OK)
                .header("docker-content-digest", &record.digest)
                .header(header::CONTENT_TYPE, &record.media_type)
                .header(header::CONTENT_LENGTH, len.to_string());
            let body = if include_body {
                axum::body::Body::from(record.bytes)
            } else {
                axum::body::Body::empty()
            };
            builder.body(body).unwrap_or_else(|_| {
                oci_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "UNKNOWN",
                    "response failed",
                )
            })
        }
        Err(err) => map_manifest_error(err),
    }
}

fn map_manifest_error(err: StorageError) -> Response {
    match err {
        StorageError::NotFound => oci_error(
            StatusCode::NOT_FOUND,
            "MANIFEST_UNKNOWN",
            "manifest unknown",
        ),
        StorageError::Invalid(message) => {
            oci_error(StatusCode::BAD_REQUEST, "NAME_INVALID", &message)
        }
        StorageError::DigestMismatch { .. } => oci_error(
            StatusCode::BAD_REQUEST,
            "DIGEST_INVALID",
            "manifest digest does not match content",
        ),
        StorageError::Io(io_err) => {
            tracing::warn!("registry manifest error: {io_err}");
            oci_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "UNKNOWN",
                "storage failure",
            )
        }
    }
}

fn media_type_or_default(media_type: &str) -> HeaderValue {
    let value = if media_type.is_empty() {
        "application/vnd.oci.image.manifest.v1+json"
    } else {
        media_type
    };
    HeaderValue::from_str(value)
        .unwrap_or_else(|_| HeaderValue::from_static("application/vnd.oci.image.manifest.v1+json"))
}
