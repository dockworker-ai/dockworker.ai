// OCI Distribution registry service for ephemeral free-tier builds.
pub mod auth;
pub mod blobs;
pub mod gc;
pub mod manifests;
pub mod storage;

use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::db::Database;

const MAX_UPLOAD_BYTES: usize = 32 * 1024 * 1024;

pub struct RegistryState {
    /// Postgres is optional so blob tests can run without a database.
    /// Manifest rows are still on disk until the SQL path is wired.
    #[allow(dead_code)]
    db: Option<Arc<Database>>,
    pub(crate) storage: storage::CasBackend,
    pub(crate) jwt_secret: String,
}

impl RegistryState {
    pub fn new(db: Arc<Database>, storage_path: String, jwt_secret: String) -> Self {
        Self {
            db: Some(db),
            storage: storage::CasBackend::new(std::path::PathBuf::from(storage_path)),
            jwt_secret,
        }
    }

    /// Blob and manifest storage with no Postgres. Used by `REGISTRY_ONLY=1`.
    pub fn standalone(
        storage_path: impl Into<std::path::PathBuf>,
        jwt_secret: impl Into<String>,
    ) -> Self {
        Self {
            db: None,
            storage: storage::CasBackend::new(storage_path.into()),
            jwt_secret: jwt_secret.into(),
        }
    }

    #[cfg(test)]
    pub fn for_tests(storage_path: std::path::PathBuf, jwt_secret: impl Into<String>) -> Self {
        Self::standalone(storage_path, jwt_secret)
    }
}

/// OCI Distribution ping. Clients require this before a push.
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
        .route("/", get(v2_check))
        .route("/token", get(auth::issue_token).post(auth::issue_token))
        // One wildcard so repository names can contain slashes (`free/cache/app`).
        // Axum 0.7 only allows a wildcard at the end of a route.
        .route(
            "/*path",
            get(dispatch)
                .head(dispatch)
                .post(dispatch)
                .patch(dispatch)
                .put(dispatch)
                .delete(dispatch),
        )
        .layer(CorsLayer::permissive())
}

enum RegOp {
    Initiate { repo: String },
    Upload { repo: String, uuid: String },
    Blob { digest: String },
    Manifest { repo: String, reference: String },
}

async fn dispatch(
    State(state): State<Arc<RegistryState>>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Response {
    let op = match parse_repo_path(&path) {
        Ok(op) => op,
        Err(message) => {
            return oci_error(StatusCode::BAD_REQUEST, "NAME_INVALID", &message);
        }
    };

    let method = req.method().clone();
    let headers = req.headers().clone();
    let query = req.uri().query().map(str::to_string);
    let body = match axum::body::to_bytes(req.into_body(), MAX_UPLOAD_BYTES).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return oci_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "BLOB_UPLOAD_INVALID",
                "upload exceeds 32MiB",
            );
        }
    };

    match op {
        RegOp::Initiate { repo } => {
            if method != Method::POST {
                return method_not_allowed();
            }
            if let Some(digest) = query_param(query.as_deref(), "digest") {
                if !body.is_empty() {
                    return blobs::monolithic_upload(&state, &repo, &digest, &body).await;
                }
            }
            blobs::initiate_upload(&state, &repo).await
        }
        RegOp::Upload { repo, uuid } => match method {
            Method::PATCH => blobs::patch_upload(&state, &repo, &uuid, &headers, &body).await,
            Method::PUT => {
                let Some(digest) = query_param(query.as_deref(), "digest") else {
                    return oci_error(
                        StatusCode::BAD_REQUEST,
                        "DIGEST_INVALID",
                        "digest query parameter is required",
                    );
                };
                blobs::complete_upload(&state, &repo, &uuid, &digest, &headers, &body).await
            }
            _ => method_not_allowed(),
        },
        RegOp::Blob { digest } => match method {
            Method::HEAD => blobs::head_blob(&state, &digest).await,
            Method::GET => blobs::get_blob(&state, &digest).await,
            _ => method_not_allowed(),
        },
        RegOp::Manifest { repo, reference } => {
            let media_type = headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/vnd.oci.image.manifest.v1+json");
            match method {
                Method::PUT => {
                    manifests::put_manifest(&state, &repo, &reference, media_type, &body).await
                }
                Method::HEAD => manifests::head_manifest(&state, &repo, &reference).await,
                Method::GET => manifests::get_manifest(&state, &repo, &reference).await,
                Method::DELETE => manifests::delete_manifest(&state, &repo, &reference).await,
                _ => method_not_allowed(),
            }
        }
    }
}

fn parse_repo_path(path: &str) -> Result<RegOp, String> {
    let path = path.trim_matches('/');
    if path.is_empty() {
        return Err("invalid repository path".into());
    }
    let parts: Vec<String> = path.split('/').map(percent_decode).collect();
    if parts
        .iter()
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err("invalid repository path".into());
    }

    let n = parts.len();
    if n >= 2 && parts[n - 2] == "blobs" && parts[n - 1] == "uploads" {
        let repo = parts[..n - 2].join("/");
        storage::validate_repo(&repo).map_err(|err| err.to_string())?;
        return Ok(RegOp::Initiate { repo });
    }
    if n >= 3 && parts[n - 3] == "blobs" && parts[n - 2] == "uploads" {
        let repo = parts[..n - 3].join("/");
        storage::validate_repo(&repo).map_err(|err| err.to_string())?;
        storage::validate_upload_id(&parts[n - 1]).map_err(|err| err.to_string())?;
        return Ok(RegOp::Upload {
            repo,
            uuid: parts[n - 1].clone(),
        });
    }
    if n >= 2 && parts[n - 2] == "blobs" {
        let repo = parts[..n - 2].join("/");
        storage::validate_repo(&repo).map_err(|err| err.to_string())?;
        return Ok(RegOp::Blob {
            digest: parts[n - 1].clone(),
        });
    }
    if n >= 2 && parts[n - 2] == "manifests" {
        let repo = parts[..n - 2].join("/");
        storage::validate_repo(&repo).map_err(|err| err.to_string())?;
        return Ok(RegOp::Manifest {
            repo,
            reference: parts[n - 1].clone(),
        });
    }
    Err("unknown registry route".into())
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(value) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(value);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    let query = query?;
    for pair in query.split('&') {
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(raw_key) == key {
            return Some(percent_decode(raw_value));
        }
    }
    None
}

pub(crate) fn oci_error(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "errors": [{ "code": code, "message": message }]
        })),
    )
        .into_response()
}

fn method_not_allowed() -> Response {
    oci_error(
        StatusCode::METHOD_NOT_ALLOWED,
        "UNSUPPORTED",
        "method not allowed",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use storage::sha256_digest;
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn app(dir: &TempDir) -> Router {
        Router::new().nest_service(
            "/v2",
            routes().with_state(Arc::new(RegistryState::for_tests(
                dir.path().to_path_buf(),
                "test-secret",
            ))),
        )
    }

    async fn send(app: &Router, builder: axum::http::request::Builder, body: &[u8]) -> Response {
        app.clone()
            .oneshot(builder.body(Body::from(body.to_vec())).unwrap())
            .await
            .unwrap()
    }

    async fn bytes_of(response: Response) -> (StatusCode, HeaderMap, Vec<u8>) {
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = axum::body::to_bytes(response.into_body(), MAX_UPLOAD_BYTES)
            .await
            .unwrap();
        (status, headers, bytes.to_vec())
    }

    #[tokio::test]
    async fn blob_and_manifest_round_trip() {
        let dir = TempDir::new().unwrap();
        let app = app(&dir);
        let repo = "free/cache/demo";

        for uri in ["/v2", "/v2/"] {
            let (status, headers, _) =
                bytes_of(send(&app, Request::builder().method("GET").uri(uri), b"").await).await;
            assert_eq!(status, StatusCode::OK, "{uri}");
            assert_eq!(
                headers
                    .get("docker-distribution-api-version")
                    .and_then(|v| v.to_str().ok()),
                Some("registry/2.0"),
                "{uri}"
            );
        }

        let (status, headers, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("POST")
                    .uri(format!("/v2/{repo}/blobs/uploads/")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        let location = headers
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(location.starts_with(&format!("/v2/{repo}/blobs/uploads/")));
        assert!(headers.get("docker-upload-uuid").is_some());

        let (status, headers, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("PATCH")
                    .uri(&location)
                    .header("content-range", "0-4"),
                b"hello",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(headers.get(header::RANGE).unwrap(), "0-4");

        let wrong = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        let (status, _, body) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("PUT")
                    .uri(format!("{location}?digest={wrong}")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(String::from_utf8_lossy(&body).contains("DIGEST_INVALID"));

        let (status, _, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("PATCH")
                    .uri(&location)
                    .header("content-range", "5-10"),
                b" world",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);

        let payload = b"hello world";
        let digest = sha256_digest(payload);
        let (status, headers, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("PUT")
                    .uri(format!("{location}?digest={digest}")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(
            headers
                .get("docker-content-digest")
                .unwrap()
                .to_str()
                .unwrap(),
            digest
        );

        let blob_uri = format!("/v2/{repo}/blobs/{digest}");
        let (status, headers, _) =
            bytes_of(send(&app, Request::builder().method("HEAD").uri(&blob_uri), b"").await).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            headers
                .get("docker-content-digest")
                .unwrap()
                .to_str()
                .unwrap(),
            digest
        );

        let (status, _, body) =
            bytes_of(send(&app, Request::builder().method("GET").uri(&blob_uri), b"").await).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, payload);

        let (status, _, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("GET")
                    .uri(format!("/v2/{repo}/blobs/sha256:1111111111111111111111111111111111111111111111111111111111111111")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let manifest = br#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","layers":[]}"#;
        let manifest_digest = sha256_digest(manifest);
        let (status, headers, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("PUT")
                    .uri(format!("/v2/{repo}/manifests/v1.0"))
                    .header("content-type", "application/vnd.oci.image.manifest.v1+json"),
                manifest,
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(
            headers
                .get("docker-content-digest")
                .unwrap()
                .to_str()
                .unwrap(),
            manifest_digest
        );

        let (status, headers, body) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("GET")
                    .uri(format!("/v2/{repo}/manifests/v1.0")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, manifest);
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap().to_str().unwrap(),
            "application/vnd.oci.image.manifest.v1+json"
        );

        let (status, _, body) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("GET")
                    .uri(format!("/v2/{repo}/manifests/{manifest_digest}")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, manifest);

        let (status, _, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/v2/{repo}/manifests/v1.0")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);

        let (status, _, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("GET")
                    .uri(format!("/v2/{repo}/manifests/v1.0")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn monolithic_upload_and_last_chunk_put() {
        let dir = TempDir::new().unwrap();
        let app = app(&dir);
        let payload = b"monolith";
        let digest = sha256_digest(payload);
        let (status, _, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("POST")
                    .uri(format!("/v2/app/blobs/uploads/?digest={digest}")),
                payload,
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let (status, _, body) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("GET")
                    .uri(format!("/v2/app/blobs/{digest}")),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, payload);

        let (status, headers, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("POST")
                    .uri("/v2/app/blobs/uploads/"),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        let location = headers.get(header::LOCATION).unwrap().to_str().unwrap();
        let chunk = b"tail";
        let chunk_digest = sha256_digest(chunk);
        let (status, _, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("PUT")
                    .uri(format!("{location}?digest={chunk_digest}")),
                chunk,
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn rejects_parent_segments() {
        let dir = TempDir::new().unwrap();
        let app = app(&dir);
        let (status, _, _) = bytes_of(
            send(
                &app,
                Request::builder()
                    .method("POST")
                    .uri("/v2/free/../blobs/uploads/"),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn token_endpoint_issues_a_verifiable_token() {
        let dir = TempDir::new().unwrap();
        let app = app(&dir);
        let (status, _, body) = bytes_of(
            send(
                &app,
                Request::builder().method("GET").uri(
                    "/v2/token?service=registry.dockworker.ai&scope=repository:free/cache/demo:pull,push",
                ),
                b"",
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = parsed["token"].as_str().unwrap();
        assert!(auth::verify_scope_access(
            "test-secret",
            token,
            "free/cache/demo",
            "push"
        ));
    }
}
