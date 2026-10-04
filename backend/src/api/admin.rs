use crate::db::Database;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

pub async fn get_stats(State(_db): State<Arc<Database>>) -> impl IntoResponse {
    // TODO: Verify admin role
    // TODO: Calculate platform-wide statistics
    (
        StatusCode::OK,
        Json(json!({
            "total_users": 0,
            "free_users": 0,
            "pro_users": 0,
            "team_accounts": 0,
            "total_builds": 0,
            "avg_build_duration_seconds": null,
            "total_build_minutes_used": 0,
            "success_rate": 0.0
        })),
    )
}
