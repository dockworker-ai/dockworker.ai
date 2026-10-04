use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::registry::RegistryState;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    service: Option<String>,
    scope: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    token: String,
    access_token: String,
    expires_in: u64,
}

const TOKEN_TTL_SECS: u64 = 3600;

/// GET or POST /v2/token?service=&scope=repository:<name>:<actions>
///
/// OCI clients request this with GET. The token is an HMAC over a JSON
/// payload, signed with the registry secret. Blob routes do not require it yet.
pub async fn issue_token(
    State(registry): State<Arc<RegistryState>>,
    Query(req): Query<TokenRequest>,
) -> impl IntoResponse {
    let service = req.service.unwrap_or_else(|| "dockworker".to_string());
    let scope = req.scope.unwrap_or_default();
    let token = issue_scoped_token(&registry.jwt_secret, &service, &scope, TOKEN_TTL_SECS);
    (
        StatusCode::OK,
        Json(TokenResponse {
            access_token: token.clone(),
            token,
            expires_in: TOKEN_TTL_SECS,
        }),
    )
}

pub fn issue_scoped_token(secret: &str, service: &str, scope: &str, ttl_secs: u64) -> String {
    let now = unix_now();
    let exp = now.saturating_add(ttl_secs);
    sign_token(secret, service, scope, exp)
}

fn sign_token(secret: &str, service: &str, scope: &str, exp: u64) -> String {
    let payload = serde_json::json!({
        "exp": exp,
        "scope": scope,
        "service": service,
    });
    let payload_bytes = payload.to_string().into_bytes();
    let sig = mac(secret, &payload_bytes);
    format!("{}.{}", b64url_encode(&payload_bytes), b64url_encode(&sig))
}

/// True when `token` is signed by `secret`, unexpired, and its scope grants
/// `action` on `repo`. A scope name may end in `*` to match a prefix.
pub fn verify_scope_access(secret: &str, token: &str, repo: &str, action: &str) -> bool {
    let Some((payload_b64, sig_b64)) = token.split_once('.') else {
        return false;
    };
    if payload_b64.is_empty() || sig_b64.is_empty() || sig_b64.contains('.') {
        return false;
    }
    let Some(payload) = b64url_decode(payload_b64) else {
        return false;
    };
    let Some(sig) = b64url_decode(sig_b64) else {
        return false;
    };
    if !ct_eq(&mac(secret, &payload), &sig) {
        return false;
    }
    let Ok(value) = serde_json::from_slice::<Value>(&payload) else {
        return false;
    };
    let Some(exp) = value.get("exp").and_then(|v| v.as_u64()) else {
        return false;
    };
    if unix_now() >= exp {
        return false;
    }
    let Some(scope) = value.get("scope").and_then(|v| v.as_str()) else {
        return false;
    };
    scope_allows(scope, repo, action)
}

fn scope_allows(scope_field: &str, repo: &str, action: &str) -> bool {
    for entry in scope_field.split_whitespace() {
        let Some(rest) = entry.strip_prefix("repository:") else {
            continue;
        };
        let Some((pattern, actions)) = rest.rsplit_once(':') else {
            continue;
        };
        let action_ok = actions.split(',').any(|candidate| candidate == action);
        if action_ok && repo_pattern_matches(pattern, repo) {
            return true;
        }
    }
    false
}

fn repo_pattern_matches(pattern: &str, repo: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => repo.starts_with(prefix),
        None => pattern == repo,
    }
}

fn mac(secret: &str, payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b".");
    hasher.update(payload);
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (left, right) in a.iter().zip(b.iter()) {
        diff |= left ^ right;
    }
    diff == 0
}

fn b64url_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push(TABLE[(n & 63) as usize] as char);
        i += 3;
    }
    if i < data.len() {
        let remaining = data.len() - i;
        let mut n = (data[i] as u32) << 16;
        if remaining == 2 {
            n |= (data[i + 1] as u32) << 8;
        }
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if remaining == 2 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        }
    }
    out
}

fn b64url_decode(input: &str) -> Option<Vec<u8>> {
    fn val(byte: u8) -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'-' => Some(62),
            b'_' => Some(63),
            _ => None,
        }
    }

    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        let a = val(bytes[i])?;
        let b = val(bytes[i + 1])?;
        let c = val(bytes[i + 2])?;
        let d = val(bytes[i + 3])?;
        let n = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | (d as u32);
        out.push((n >> 16) as u8);
        out.push((n >> 8) as u8);
        out.push(n as u8);
        i += 4;
    }
    let rest = bytes.len() - i;
    if rest == 2 {
        let a = val(bytes[i])?;
        let b = val(bytes[i + 1])?;
        let n = ((a as u32) << 18) | ((b as u32) << 12);
        out.push((n >> 16) as u8);
    } else if rest == 3 {
        let a = val(bytes[i])?;
        let b = val(bytes[i + 1])?;
        let c = val(bytes[i + 2])?;
        let n = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6);
        out.push((n >> 16) as u8);
        out.push((n >> 8) as u8);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_token_allows_action_and_glob() {
        let secret = "test-secret";
        let token = issue_scoped_token(
            secret,
            "registry.dockworker.ai",
            "repository:free/*:pull,push",
            3600,
        );
        assert!(verify_scope_access(
            secret,
            &token,
            "free/cache/demo",
            "pull"
        ));
        assert!(verify_scope_access(
            secret,
            &token,
            "free/cache/demo",
            "push"
        ));
        assert!(!verify_scope_access(
            secret,
            &token,
            "free/cache/demo",
            "delete"
        ));
        assert!(!verify_scope_access(secret, &token, "other/app", "pull"));
        assert!(!verify_scope_access(
            "other-secret",
            &token,
            "free/cache/demo",
            "pull"
        ));
    }

    #[test]
    fn exact_repo_scope_does_not_prefix_match() {
        let secret = "test-secret";
        let token = issue_scoped_token(secret, "svc", "repository:free/cache/demo:pull", 3600);
        assert!(verify_scope_access(
            secret,
            &token,
            "free/cache/demo",
            "pull"
        ));
        assert!(!verify_scope_access(
            secret,
            &token,
            "free/cache/demo/extra",
            "pull"
        ));
    }

    #[test]
    fn expired_token_is_rejected() {
        let token = sign_token("test-secret", "svc", "repository:app:pull", 1);
        assert!(!verify_scope_access("test-secret", &token, "app", "pull"));
    }
}
