use anyhow::Result;

pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub stripe_secret_key: String,
    pub jwt_secret: String,
    pub redis_url: String,
    pub registry_storage_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/dockworker".to_string()),
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8000".to_string()),
            github_client_id: std::env::var("GITHUB_CLIENT_ID")?,
            github_client_secret: std::env::var("GITHUB_CLIENT_SECRET")?,
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY")?,
            jwt_secret: std::env::var("JWT_SECRET")?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            registry_storage_path: std::env::var("REGISTRY_STORAGE_PATH").ok(),
        })
    }
}
