use crate::{db::Database, models::*};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

pub async fn trigger(
    State(_db): State<Arc<Database>>,
    Json(_req): Json<BuildRequest>,
) -> impl IntoResponse {
    // TODO: Validate user quota
    // TODO: Enqueue build job
    // TODO: Return build ID
    (StatusCode::ACCEPTED, Json(json!({"id": "build_123"})))
}

pub async fn get_status(
    State(_db): State<Arc<Database>>,
    Path(_build_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Query build status from DB
    (StatusCode::OK, Json(json!({"status": "RUNNING"})))
}

pub async fn stream_logs(
    State(_db): State<Arc<Database>>,
    Path(_build_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Implement Server-Sent Events (SSE) streaming
    (StatusCode::OK, "")
}

pub async fn list_user_builds(State(_db): State<Arc<Database>>) -> impl IntoResponse {
    // TODO: List user's builds from DB
    (StatusCode::OK, Json(json!({"builds": []})))
}
