use anyhow::{bail, Result};
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub stripe_secret_key: String,
    pub jwt_secret: String,
    pub redis_url: String,
    pub registry_storage_path: Option<String>,
    /// Serve `/health` and `/v2` without GitHub, Stripe, or Postgres.
    pub registry_only: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Self::from_map(std::env::vars().collect())
    }

    pub fn from_map(vars: HashMap<String, String>) -> Result<Self> {
        let registry_only = matches!(
            vars.get("REGISTRY_ONLY").map(String::as_str),
            Some("1") | Some("true") | Some("TRUE")
        );
        let jwt_secret = required(&vars, "JWT_SECRET")?;
        let listen_addr = vars
            .get("LISTEN_ADDR")
            .cloned()
            .unwrap_or_else(|| "0.0.0.0:8000".to_string());
        let registry_storage_path = vars.get("REGISTRY_STORAGE_PATH").cloned();

        if registry_only {
            return Ok(Self {
                database_url: String::new(),
                listen_addr,
                github_client_id: String::new(),
                github_client_secret: String::new(),
                stripe_secret_key: String::new(),
                jwt_secret,
                redis_url: String::new(),
                registry_storage_path,
                registry_only: true,
            });
        }

        Ok(Self {
            database_url: vars
                .get("DATABASE_URL")
                .cloned()
                .unwrap_or_else(|| "postgres://localhost/dockworker".to_string()),
            listen_addr,
            github_client_id: required(&vars, "GITHUB_CLIENT_ID")?,
            github_client_secret: required(&vars, "GITHUB_CLIENT_SECRET")?,
            stripe_secret_key: required(&vars, "STRIPE_SECRET_KEY")?,
            jwt_secret,
            redis_url: vars
                .get("REDIS_URL")
                .cloned()
                .unwrap_or_else(|| "redis://localhost:6379".to_string()),
            registry_storage_path,
            registry_only: false,
        })
    }
}

fn required(vars: &HashMap<String, String>, key: &str) -> Result<String> {
    match vars.get(key) {
        Some(value) if !value.is_empty() => Ok(value.clone()),
        _ => bail!("{key} is required"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn registry_only_skips_github_stripe_and_database() {
        let config = Config::from_map(vars(&[
            ("REGISTRY_ONLY", "1"),
            ("JWT_SECRET", "s3cret"),
            ("LISTEN_ADDR", "127.0.0.1:8080"),
            ("REGISTRY_STORAGE_PATH", "/tmp/blobs"),
        ]))
        .unwrap();
        assert!(config.registry_only);
        assert_eq!(config.jwt_secret, "s3cret");
        assert_eq!(config.listen_addr, "127.0.0.1:8080");
        assert_eq!(config.registry_storage_path.as_deref(), Some("/tmp/blobs"));
        assert!(config.github_client_id.is_empty());
        assert!(config.github_client_secret.is_empty());
        assert!(config.stripe_secret_key.is_empty());
        assert!(config.database_url.is_empty());
    }

    #[test]
    fn full_config_still_requires_github_and_stripe() {
        let message = expect_err(Config::from_map(vars(&[("JWT_SECRET", "s3cret")])));
        assert!(message.contains("GITHUB_CLIENT_ID"), "{message}");
    }

    #[test]
    fn registry_only_still_requires_jwt_secret() {
        let message = expect_err(Config::from_map(vars(&[("REGISTRY_ONLY", "1")])));
        assert!(message.contains("JWT_SECRET"), "{message}");
    }

    fn expect_err(result: Result<Config>) -> String {
        match result {
            Ok(_) => panic!("expected config error"),
            Err(err) => err.to_string(),
        }
    }
}
