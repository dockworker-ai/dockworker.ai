use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::db::Database;
use serde_json::json;

pub async fn get_quota(
    State(_db): State<Arc<Database>>,
) -> impl IntoResponse {
    // TODO: Get authenticated user from JWT
    // TODO: Query quota from DB
    (StatusCode::OK, Json(json!({
        "monthly_minutes": 200,
        "remaining_minutes": 150,
        "max_concurrent": 1,
        "reset_date": "2025-11-01T00:00:00Z"
    })))
}

pub async fn get_profile(
    State(_db): State<Arc<Database>>,
) -> impl IntoResponse {
    // TODO: Get authenticated user from JWT
    // TODO: Return user profile
    (StatusCode::OK, Json(json!({
        "id": "user_123",
        "username": "developer",
        "email": "dev@example.com",
        "tier": "free",
        "created_at": "2025-01-01T00:00:00Z"
    })))
}
