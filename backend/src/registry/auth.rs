use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::registry::RegistryState;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    service: String,
    scope: String, // e.g., "repository:builds/free-*:pull,push"
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    token: String,
    access_token: String,
    expires_in: u64,
}

/// Issue a scoped OCI Distribution v1.1 bearer token
///
/// Scope format: `repository:<name>:<actions>`
/// Example: `repository:builds/usr_abc123:pull,push`
pub async fn issue_token(
    State(_registry): State<Arc<RegistryState>>,
    Query(req): Query<TokenRequest>,
) -> impl IntoResponse {
    // TODO: Validate scope against tenant/build permissions
    // TODO: Issue JWT with exp, sub (build_id), scope claims

    let token = format!("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."); // Placeholder

    (
        StatusCode::OK,
        Json(TokenResponse {
            token: token.clone(),
            access_token: token,
            expires_in: 3600,
        }),
    )
}

/// Verify bearer token scope against requested action
pub fn verify_scope_access(token: &str, scope: &str, action: &str) -> bool {
    // TODO: Decode JWT, extract scope claim
    // TODO: Validate requested action against allowed scope
    // Example: scope="repository:builds/free-*:pull,push", action="pull" → true
    true
}
