# dockworker.ai

Production-grade multi-tenant container build platform. Free tier: each build is an isolated Kubernetes Job (non-root, user namespace, daemonless rootless BuildKit, auto-deleted).

| Path | What it is |
| --- | --- |
| [`backend/`](backend/) | Control plane (Rust/Axum + gRPC, Kubernetes API integration, log streaming). |
| [`frontend/`](frontend/) | Console & landing page (React, Zustand, Tailwind). |
| [`e2e/`](e2e/) | Playwright E2E tests for CF Pages deployment. |
| [`k8s/`](k8s/) | Phase 1 sandbox infrastructure (namespace, quotas, network policy, test jobs). |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Production architecture spec (hardened design, unit economics, 12-week roadmap). |
| [`docs/planning/`](docs/planning/README.md) | Epics and user stories for Phase 1–5. |

## Deployment

**Design (CF Pages):** https://8e165ae2.dockworker-ai.pages.dev/

**Local Development**

Control plane (Rust, requires Kubernetes API access):

```bash
cd backend && cargo test && cargo run
```

Frontend (React, standalone):

```bash
cd frontend && bun install && bun run dev
```

**Phase 1 Sandbox (Kubernetes)**

Deploy infrastructure:

```bash
kubectl apply -k k8s/                    # Base (namespace, quotas, network policy)
kubectl apply -k k8s/overlays/k8s-1.30/  # K8s 1.30+ with user namespaces (preferred)
# OR
kubectl apply -k k8s/overlays/k8s-pre-1.30/  # Pre-1.30 with AppArmor/SELinux fallback
```

Run 100 test builds (Phase 1 exit criterion):

```bash
# See k8s/README.md for full validation script
kubectl apply -f k8s/test-job.yaml
kubectl wait --for=condition=complete job/dockworker-test-build -n dockworker-runners --timeout=10m
```

**E2E Tests (Playwright)**

Against staging deployment:

```bash
bun install && bunx playwright install
BASE_URL="https://8e165ae2.dockworker-ai.pages.dev" bun run test:e2e
```

## Status

- ✅ **Design:** Figma → React/Vite deployed to CF Pages
- ✅ **Backend:** Rust skeleton + Kubernetes Job runner (runner_job.rs, runner_controller.rs)
- ✅ **E2E Tests:** Playwright suite updated for new design (landing-page, auth, build-flow, performance)
- ✅ **Phase 1 K8s:** Namespace, quotas, network policy, user namespace support
- 🔄 **Phase 1 Testing:** Ready to run 100 clean builds (no pod evictions)
- ⏳ **Phase 2:** Network hardening (Cilium eBPF, egress filtering)
- ⏳ **Phase 3:** Control plane core (gRPC, Redis queue, Axum SSE, log streaming)
- ⏳ **Phase 4:** Identity & quotas (GitHub OAuth, multi-signal Sybil defense)
- ⏳ **Phase 5:** Production ramp (100-user cohort, load testing)
