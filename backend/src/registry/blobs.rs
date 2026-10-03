use axum::{
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::registry::storage::StorageError;
use crate::registry::{oci_error, RegistryState};

/// POST /v2/<name>/blobs/uploads/
pub async fn initiate_upload(registry: &RegistryState, name: &str) -> Response {
    let upload_id = Uuid::new_v4().to_string();
    if let Err(err) = registry.storage.create_upload(&upload_id).await {
        return map_upload_error(err);
    }
    accepted(name, &upload_id, 0)
}

/// POST /v2/<name>/blobs/uploads/?digest= with the full body.
pub async fn monolithic_upload(
    registry: &RegistryState,
    name: &str,
    digest: &str,
    body: &[u8],
) -> Response {
    let upload_id = Uuid::new_v4().to_string();
    if let Err(err) = registry.storage.create_upload(&upload_id).await {
        return map_upload_error(err);
    }
    if let Err(err) = registry.storage.append_chunk(&upload_id, body).await {
        return map_upload_error(err);
    }
    finish(registry, name, &upload_id, digest).await
}

/// PATCH /v2/<name>/blobs/uploads/<uuid>
pub async fn patch_upload(
    registry: &RegistryState,
    name: &str,
    uuid: &str,
    headers: &HeaderMap,
    chunk: &[u8],
) -> Response {
    let current = match registry.storage.upload_len(uuid).await {
        Ok(len) => len,
        Err(err) => return map_upload_error(err),
    };
    if let Some(response) = reject_content_range(headers, current, chunk.len() as u64) {
        return response;
    }
    match registry.storage.append_chunk(uuid, chunk).await {
        Ok(len) => accepted(name, uuid, len),
        Err(err) => map_upload_error(err),
    }
}

/// PUT /v2/<name>/blobs/uploads/<uuid>?digest=<sha256:...>
pub async fn complete_upload(
    registry: &RegistryState,
    name: &str,
    uuid: &str,
    digest: &str,
    headers: &HeaderMap,
    chunk: &[u8],
) -> Response {
    if !chunk.is_empty() {
        let current = match registry.storage.upload_len(uuid).await {
            Ok(len) => len,
            Err(err) => return map_upload_error(err),
        };
        if let Some(response) = reject_content_range(headers, current, chunk.len() as u64) {
            return response;
        }
        if let Err(err) = registry.storage.append_chunk(uuid, chunk).await {
            return map_upload_error(err);
        }
    }
    finish(registry, name, uuid, digest).await
}

/// HEAD /v2/<name>/blobs/<digest>
pub async fn head_blob(registry: &RegistryState, digest: &str) -> Response {
    match registry.storage.blob_len(digest).await {
        Ok(len) => blob_headers(StatusCode::OK, digest, len, None),
        Err(StorageError::NotFound) => {
            oci_error(StatusCode::NOT_FOUND, "BLOB_UNKNOWN", "blob unknown")
        }
        Err(StorageError::Invalid(message)) => {
            oci_error(StatusCode::BAD_REQUEST, "DIGEST_INVALID", &message)
        }
        Err(err) => map_upload_error(err),
    }
}

/// GET /v2/<name>/blobs/<digest>
pub async fn get_blob(registry: &RegistryState, digest: &str) -> Response {
    match registry.storage.get_blob(digest).await {
        Ok(bytes) => {
            let len = bytes.len() as u64;
            blob_headers(StatusCode::OK, digest, len, Some(bytes))
        }
        Err(StorageError::NotFound) => {
            oci_error(StatusCode::NOT_FOUND, "BLOB_UNKNOWN", "blob unknown")
        }
        Err(StorageError::Invalid(message)) => {
            oci_error(StatusCode::BAD_REQUEST, "DIGEST_INVALID", &message)
        }
        Err(err) => map_upload_error(err),
    }
}

async fn finish(registry: &RegistryState, name: &str, uuid: &str, digest: &str) -> Response {
    match registry.storage.finalize_upload(uuid, digest).await {
        Ok(_) => created_blob(name, digest),
        Err(err) => map_upload_error(err),
    }
}

fn accepted(name: &str, upload_id: &str, len: u64) -> Response {
    let location = format!("/v2/{name}/blobs/uploads/{upload_id}");
    let mut headers = HeaderMap::new();
    if insert_str(&mut headers, header::LOCATION, &location).is_err()
        || insert_static(&mut headers, "docker-upload-uuid", upload_id).is_err()
        || insert_str(&mut headers, header::RANGE, &range_value(len)).is_err()
    {
        return oci_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UNKNOWN",
            "invalid header",
        );
    }
    (StatusCode::ACCEPTED, headers).into_response()
}

fn created_blob(name: &str, digest: &str) -> Response {
    let location = format!("/v2/{name}/blobs/{digest}");
    let mut headers = HeaderMap::new();
    if insert_str(&mut headers, header::LOCATION, &location).is_err()
        || insert_static(&mut headers, "docker-content-digest", digest).is_err()
    {
        return oci_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UNKNOWN",
            "invalid header",
        );
    }
    (StatusCode::CREATED, headers).into_response()
}

fn blob_headers(status: StatusCode, digest: &str, len: u64, body: Option<Vec<u8>>) -> Response {
    let response = Response::builder()
        .status(status)
        .header("docker-content-digest", digest)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, len.to_string());
    if let Some(bytes) = body {
        response
            .body(axum::body::Body::from(bytes))
            .unwrap_or_else(|_| {
                oci_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "UNKNOWN",
                    "response failed",
                )
            })
    } else {
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
}

fn range_value(len: u64) -> String {
    if len == 0 {
        "0-0".to_string()
    } else {
        format!("0-{}", len - 1)
    }
}

/// `Content-Range` is inclusive. When the client sends it, the start must be
/// the current length and the end must cover exactly `chunk_len` bytes.
fn reject_content_range(headers: &HeaderMap, current: u64, chunk_len: u64) -> Option<Response> {
    let value = headers.get(header::CONTENT_RANGE)?;
    let Ok(text) = value.to_str() else {
        return Some(oci_error(
            StatusCode::RANGE_NOT_SATISFIABLE,
            "BLOB_UPLOAD_INVALID",
            "invalid content range",
        ));
    };
    let Some((start, end)) = parse_content_range(text) else {
        return Some(oci_error(
            StatusCode::RANGE_NOT_SATISFIABLE,
            "BLOB_UPLOAD_INVALID",
            "invalid content range",
        ));
    };
    let expected_end = current.saturating_add(chunk_len).saturating_sub(1);
    if chunk_len == 0 || start != current || end != expected_end {
        return Some(oci_error(
            StatusCode::RANGE_NOT_SATISFIABLE,
            "BLOB_UPLOAD_INVALID",
            "content range does not match upload offset",
        ));
    }
    None
}

fn parse_content_range(value: &str) -> Option<(u64, u64)> {
    let value = value.trim().trim_start_matches("bytes ").trim();
    let (start, rest) = value.split_once('-')?;
    let end = rest.split('/').next()?.trim();
    if end.is_empty() || end == "*" {
        return None;
    }
    Some((start.trim().parse().ok()?, end.parse().ok()?))
}

fn map_upload_error(err: StorageError) -> Response {
    match &err {
        StorageError::NotFound => oci_error(
            StatusCode::NOT_FOUND,
            "BLOB_UPLOAD_UNKNOWN",
            "upload unknown",
        ),
        StorageError::DigestMismatch { .. } => {
            oci_error(StatusCode::BAD_REQUEST, "DIGEST_INVALID", &err.to_string())
        }
        StorageError::Invalid(message) => {
            oci_error(StatusCode::BAD_REQUEST, "BLOB_UPLOAD_INVALID", message)
        }
        StorageError::Io(io_err) => {
            tracing::warn!("registry storage error: {io_err}");
            oci_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "UNKNOWN",
                "storage failure",
            )
        }
    }
}

fn insert_str(headers: &mut HeaderMap, name: HeaderName, value: &str) -> Result<(), ()> {
    headers.insert(name, HeaderValue::from_str(value).map_err(|_| ())?);
    Ok(())
}

fn insert_static(headers: &mut HeaderMap, name: &'static str, value: &str) -> Result<(), ()> {
    headers.insert(
        HeaderName::from_static(name),
        HeaderValue::from_str(value).map_err(|_| ())?,
    );
    Ok(())
}
