#!/bin/bash
# Phase 2: Harbormaster job dispatch via og-cli
# Usage: ./PHASE2_DISPATCH.sh

set -e

echo "🚀 Dispatching Phase 2 tasks via harbormaster..."

# Task 1: SQL Query Implementation
echo "📋 Submitting Task 1: SQL Queries..."
og-cli job submit \
  --name "phase2-sql-queries" \
  --title "Phase 2: SQL Query Implementation" \
  --description "Implement sqlx queries for manifest storage/retrieval (put/head/get/delete)" \
  --agent cursor-sql \
  --branch phase2/sql-queries \
  --context-files backend/src/registry/manifests.rs,backend/migrations/001_registry_schema.sql,backend/src/db.rs \
  --labels phase2,registry,database

# Task 2: JWT Token Implementation
echo "🔐 Submitting Task 2: JWT Auth..."
og-cli job submit \
  --name "phase2-jwt-auth" \
  --title "Phase 2: JWT Token Implementation" \
  --description "Implement JWT issuance and verification for OCI scopes (issue_token, verify_scope_access)" \
  --agent cursor-auth \
  --branch phase2/jwt-auth \
  --context-files backend/src/registry/auth.rs,backend/src/config.rs,backend/src/models.rs \
  --labels phase2,registry,auth

# Task 4: Cilium Policy Testing (can run in parallel with 1 & 2)
echo "🔗 Submitting Task 4: Cilium Testing..."
og-cli job submit \
  --name "phase2-cilium-test" \
  --title "Phase 2: Cilium Layer 7 Testing" \
  --description "Test Cilium FQDN egress filtering in Kind cluster (Docker Hub, GHCR, registry.dockworker.ai)" \
  --agent cursor-k8s \
  --branch phase2/cilium-test \
  --context-files k8s/cilium-network-policy.yaml,k8s/test-job.yaml,k8s/README.md \
  --labels phase2,network,kubernetes

# Content Audit (parallel, independent)
echo "📝 Submitting Content Audit: dockworker.ai web app..."
og-cli job submit \
  --name "content-audit" \
  --title "Content Audit: dockworker.ai Web App" \
  --description "Full text review for accuracy, tone, grammar, and brand consistency" \
  --agent cursor-content \
  --branch feature/content-audit \
  --labels phase2,content-review,qa

# Task 3: Manifest Handlers Wiring (depends on Tasks 1 & 2)
echo "⏳ Task 3 (Manifest Handlers) will be submitted after Tasks 1 & 2 complete..."
echo ""
echo "✅ Phase 2 tasks submitted to harbormaster!"
echo ""
echo "Monitor progress:"
echo "  og-cli job list --status in-progress"
echo "  og-cli job status phase2-sql-queries"
echo "  og-cli job status phase2-jwt-auth"
echo "  og-cli job status phase2-cilium-test"
echo "  og-cli job status content-audit"
echo ""
echo "View results:"
echo "  og-cli job logs phase2-sql-queries"
echo "  og-cli job artifacts phase2-sql-queries"
echo ""
echo "Expected completion: 4-6 hours (parallel execution)"
