use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::db::Database;
use serde_json::json;

pub async fn github_callback(
    State(_db): State<Arc<Database>>,
) -> impl IntoResponse {
    // TODO: Exchange GitHub code for access token
    // TODO: Fetch user info from GitHub API
    // TODO: Create or update user in DB
    // TODO: Generate JWT
    (StatusCode::OK, Json(json!({"token": "jwt_token_here"})))
}

pub async fn logout(
    State(_db): State<Arc<Database>>,
) -> impl IntoResponse {
    // TODO: Invalidate JWT / session
    (StatusCode::OK, Json(json!({"message": "logged out"})))
}
