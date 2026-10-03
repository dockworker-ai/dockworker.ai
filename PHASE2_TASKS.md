# Phase 2: Network Hardening & Registry Persistence

**Scope:** Weeks 3–4  
**Exit Criteria:** All non-approved egress drops; DNS works; 20 parallel builds stream live logs to UI  
**Dispatch Mechanism:** `og-cli` (harbormaster backing)

---

## Task 1: SQL Query Implementation

**Agent:** cursor-sql  
**Scope:** backend/src/registry/manifests.rs + database integration

```bash
og-cli job submit \
  --name "phase2-sql-queries" \
  --description "Implement sqlx queries for manifest storage/retrieval" \
  --agent cursor-sql \
  --branch phase2/sql-queries \
  --context-files backend/src/registry/manifests.rs,backend/migrations/001_registry_schema.sql,backend/src/db.rs
```

**Acceptance Criteria:**
- [ ] `put_manifest`: INSERT into `manifests` + `blob_refs` for each layer
  - Compute SHA-256 digest of manifest JSON
  - Validate all referenced blob digests exist
  - Record `created_at` for TTL-based purge
- [ ] `head_manifest`: SELECT manifest by `(tenant_id, repository_name, reference)`
  - Return `Docker-Content-Digest` header (canonical digest)
  - Return `Content-Length`
- [ ] `get_manifest`: SELECT `manifest_json` + `content_type`
  - Stream response (not buffered)
- [ ] `delete_manifest`: DELETE by digest (soft-delete: set `deleted_at`)
  - Trigger blob GC: mark blobs unreferenced if no other manifests use them
- [ ] `cargo sqlx prepare` passes offline (type-checked against migrations)
- [ ] Tests: verify each query against running Postgres instance

**Deliverable:** PR to `phase2/sql-queries` with working sqlx handlers

---

## Task 2: JWT Token Implementation

**Agent:** cursor-auth  
**Scope:** backend/src/registry/auth.rs + JWT signing

```bash
og-cli job submit \
  --name "phase2-jwt-auth" \
  --description "Implement JWT issuance and verification for OCI scopes" \
  --agent cursor-auth \
  --branch phase2/jwt-auth \
  --context-files backend/src/registry/auth.rs,backend/src/config.rs,backend/src/models.rs
```

**Acceptance Criteria:**
- [ ] `issue_token(scope: &str)` → JWT with claims:
  - `sub`: build_id or tenant_id
  - `scope`: "repository:builds/free-*:pull,push" (parsed from request)
  - `actions`: array of ["pull", "push", "delete", "manage"]
  - `exp`: 1 hour from now
  - `iat`, `jti` (JWT ID for revocation)
- [ ] Sign with HS256 (JWT_SECRET from config)
- [ ] `verify_scope_access(token: &str, scope: &str, action: &str)` → bool
  - Decode JWT (verify signature)
  - Extract `scope` claim
  - Validate requested action is in allowed actions
  - Check expiration
- [ ] Scope format validation:
  - `repository:builds/{tenant_id}/*:pull,push`
  - `repository:profiles/{username}/{app}:pull`
- [ ] Token revocation support (check `tokens.revoked_at` in DB)
- [ ] Tests: hardcoded secret, verify round-trip (issue → verify)

**Deliverable:** PR to `phase2/jwt-auth` with working token handlers

---

## Task 3: Manifest Handlers Wiring

**Agent:** cursor-integration  
**Scope:** Wire handlers → SQL + JWT verification

```bash
og-cli job submit \
  --name "phase2-manifest-wiring" \
  --description "Wire registry manifest handlers to SQL queries + token auth" \
  --agent cursor-integration \
  --branch phase2/manifest-handlers \
  --context-files backend/src/registry/manifests.rs,backend/src/registry/auth.rs,backend/src/registry/mod.rs
```

**Acceptance Criteria:**
- [ ] `put_manifest` handler:
  - Extract JWT from `Authorization: Bearer <token>` header
  - Verify scope includes `push` action
  - Parse manifest JSON body
  - Call SQL: `put_manifest()`
  - Return 201 Created + `Docker-Content-Digest` header
- [ ] `head_manifest` handler:
  - Call SQL: `head_manifest()`
  - Return 200 + `Content-Type`, `Docker-Content-Digest`, `Content-Length` headers
  - Return 404 if not found
- [ ] `get_manifest` handler:
  - Call SQL: `get_manifest()`
  - Stream manifest_json in response body
  - Return correct `Content-Type`
  - Return 404 if not found
- [ ] `delete_manifest` handler:
  - Verify scope includes `delete` action
  - Call SQL: `delete_manifest()`
  - Return 202 Accepted (async GC)
  - Return 404 if not found
- [ ] Error handling:
  - 401 Unauthorized: invalid/expired JWT
  - 403 Forbidden: scope mismatch (e.g., `pull` scope trying `push`)
  - 400 Bad Request: malformed manifest JSON
  - 500 Internal Server Error: DB connectivity loss
- [ ] Integration tests: hit handlers with curl/httpie against test DB

**Deliverable:** PR to `phase2/manifest-handlers` with wired handlers + integration tests

---

## Task 4: Cilium Layer 7 Policy Testing

**Agent:** cursor-k8s  
**Scope:** Validate k8s/cilium-network-policy.yaml in Kind cluster

```bash
og-cli job submit \
  --name "phase2-cilium-test" \
  --description "Test Cilium Layer 7 FQDN egress filtering in Kind cluster" \
  --agent cursor-k8s \
  --branch phase2/cilium-test \
  --context-files k8s/cilium-network-policy.yaml,k8s/test-job.yaml,k8s/README.md
```

**Acceptance Criteria:**
- [ ] Kind cluster setup (k8s 1.30+, Cilium v1.14+)
- [ ] Deploy policy: `kubectl apply -f k8s/cilium-network-policy.yaml`
- [ ] Deploy test Job in dockworker-runners namespace
- [ ] Test each egress rule:
  - [ ] **DNS (UDP 53):** `nslookup registry.dockworker.ai` → succeeds
  - [ ] **Docker Hub (HTTPS):** `curl https://production.cloudflare.docker.com/v2/` → succeeds
  - [ ] **GHCR (HTTPS):** `curl https://ghcr.io/v2/` → succeeds
  - [ ] **registry.dockworker.ai (HTTPS):** `curl https://registry.dockworker.ai/v2/` → succeeds
  - [ ] **registry.dockworker.ai (HTTP):** `curl http://registry.dockworker.ai/v2/` → succeeds (fallback)
  - [ ] **Non-whitelisted FQDN:** `curl https://google.com` → blocked (connection refused or timeout)
- [ ] Test ingress (deny-all):
  - [ ] No inbound traffic allowed to pod (e.g., `nc -zv pod-ip 5000` fails)
- [ ] Cilium monitor output: verify `DROP` events for blocked traffic
- [ ] Generate test report: PASS/FAIL per rule + packet captures
- [ ] Update k8s/README.md with test results + reproduction steps

**Deliverable:** PR to `phase2/cilium-test` with test results + CI job to re-run tests

---

## Parallel Execution

All 4 tasks can run in parallel:
- **Task 1 & 2** are independent (SQL queries vs. JWT auth)
- **Task 3** depends on Tasks 1 & 2 (integration)
- **Task 4** is independent (infrastructure testing)

Recommended order: Submit Tasks 1, 2, 4 in parallel. Task 3 starts once Tasks 1 & 2 complete.

---

## Integration & Merge (Claude Code Role)

1. Wait for all 4 PRs to complete
2. Run `cargo test` + `cargo build --release`
3. Gemini: Adversarial audit (OCI spec compliance, SQL race conditions, eBPF drops)
4. Merge PRs into main in order: 1 → 2 → 3 → 4
5. Deploy Phase 2 to staging: run 20 parallel test builds
6. Exit criterion: All non-approved egress drops; DNS works; 20 builds stream live

---

## References

- **SQL Schema:** `backend/migrations/001_registry_schema.sql`
- **Cilium Policy:** `k8s/cilium-network-policy.yaml`
- **Architecture:** `docs/ARCHITECTURE.md` (Section 2: OCI Distribution v1.1)
- **Registry Spec:** `backend/src/registry/README.md`
