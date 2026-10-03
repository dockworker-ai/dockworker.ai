use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub github_id: Option<i64>,
    pub username: String,
    pub email: Option<String>,
    pub tier: UserTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_tier", rename_all = "lowercase")]
pub enum UserTier {
    Free,
    Pro,
    Team,
    Enterprise,
}

impl std::fmt::Display for UserTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => write!(f, "free"),
            Self::Pro => write!(f, "pro"),
            Self::Team => write!(f, "team"),
            Self::Enterprise => write!(f, "enterprise"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserQuota {
    pub user_id: String,
    pub monthly_build_minutes: i32,
    pub remaining_minutes: i32,
    pub max_concurrent_builds: i32,
    pub reset_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Build {
    pub id: String,
    pub user_id: String,
    pub status: BuildStatus,
    pub repo_url: String,
    pub repo_branch: String,
    pub commit_sha: String,
    pub dockerfile_path: String,
    pub image_tag: String,
    pub registry_url: Option<String>,
    pub build_duration_seconds: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "build_status", rename_all = "UPPERCASE")]
pub enum BuildStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Timeout,
    Cancelled,
}

impl std::fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "QUEUED"),
            Self::Running => write!(f, "RUNNING"),
            Self::Succeeded => write!(f, "SUCCEEDED"),
            Self::Failed => write!(f, "FAILED"),
            Self::Timeout => write!(f, "TIMEOUT"),
            Self::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRequest {
    pub repo_url: String,
    pub repo_branch: String,
    pub commit_sha: String,
    pub dockerfile_path: String,
    pub image_tag: String,
    pub registry_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResponse {
    pub id: String,
    pub status: BuildStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogChunk {
    pub build_id: String,
    pub sequence: i64,
    pub stream: String, // "stdout" or "stderr"
    pub payload: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BuilderNode {
    pub id: String,
    pub hostname: String,
    pub region: String,
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub available_cpu: i32,
    pub available_memory_gb: i32,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub free_users: i64,
    pub pro_users: i64,
    pub team_accounts: i64,
    pub total_builds: i64,
    pub avg_build_duration_seconds: Option<i32>,
    pub total_build_minutes_used: i64,
    pub success_rate: f64,
}
