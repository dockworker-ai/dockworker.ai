pub mod auth;
pub mod builds;
pub mod user;
pub mod admin;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, json!({"status": "healthy"}))
}
