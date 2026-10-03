pub mod auth;
pub mod builds;
pub mod user;
pub mod admin;

use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

pub async fn health() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"status": "healthy"})))
}
