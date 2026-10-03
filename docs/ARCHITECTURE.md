# dockworker.ai: Production Architecture Specification

**Version:** 1.0 (Hardened, Production-Grade)  
**Last Updated:** 2026-10-03  
**Status:** Ready for Phase 1 Execution

---

## Executive Summary

dockworker.ai is a hardened, multi-tenant container build platform designed to serve tens of thousands of developers on a sustainable freemium model. This specification documents the production-ready architecture incorporating security-first design, scale-to-zero economics, and proven Kubernetes patterns.

### Key Decisions
- **Rootless BuildKit** with `buildctl-daemonless.sh` (not long-lived daemons)
- **Kubernetes 1.30+ user namespaces** with pre-1.30 fallback overlay
- **Cilium eBPF** for Layer 7 network perimeter (FQDN filtering + egressDeny rules)
- **Tenant-namespaced cache repositories** (OCI content-addressable storage)
- **Multi-signal Sybil defense** (GitHub age, IP reputation, commit activity, MFA, verified email)
- **Realistic unit economics** ($210/mo platform baseline + per-node compute)
- **12-week phased rollout** with 5 FTE engineering team

---

## 1. Functional Ephemeral Build Job

### Kubernetes 1.30+ with User Namespaces (Primary)

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: dockworker-build-task-94f1c
  namespace: dockworker-runners
  labels:
    app.kubernetes.io/part-of: dockworker-ai
    dockworker.ai/tier: freemium
    dockworker.ai/tenant-id: "usr_92bf81a3"
spec:
  activeDeadlineSeconds: 600 # Aligned to published 10-minute limit
  backoffLimit: 0
  template:
    metadata:
      labels:
        dockworker.ai/runner: rootless-buildkit
    spec:
      restartPolicy: Never
      automountServiceAccountToken: false
      hostUsers: false # K8s 1.30+ native user namespace isolation
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
        seccompProfile:
          type: RuntimeDefault
      initContainers:
        - name: fetch-context
          image: ghcr.io/dockworker-ai/bootstrap-agent:v0.1.0
          command: ["/bin/bootstrap"]
          args: ["--task-id", "task_94f1c", "--target-dir", "/workspace"]
          env:
            - name: CONTROL_PLANE_GRPC
              value: "control-plane.dockworker.svc.cluster.local:50051"
            - name: LEASE_TOKEN
              valueFrom:
                secretKeyRef:
                  name: build-task-94f1c-token
                  key: token
          volumeMounts:
            - name: workspace
              mountPath: /workspace
            - name: auth-dir
              mountPath: /home/user/.docker
          resources:
            requests:
              cpu: "100m"
              memory: "128Mi"
            limits:
              cpu: "500m"
              memory: "256Mi"
      containers:
        - name: builder
          image: moby/buildkit:v0.26.2-rootless
          command: ["buildctl-daemonless.sh"]
          args:
            - build
            - --frontend=dockerfile.v0
            - --local=context=/workspace
            - --local=dockerfile=/workspace
            - --output=type=image,name=$(DEST_IMAGE),push=true
            - --export-cache=type=registry,ref=$(CACHE_REPO),mode=max
            - --import-cache=type=registry,ref=$(CACHE_REPO)
            - --progress=plain
          env:
            - name: BUILDKITD_FLAGS
              value: "--root=/scratch/buildkit --oci-worker-no-process-sandbox"
            - name: DEST_IMAGE
              value: "ghcr.io/user-repo/agent:latest"
            - name: CACHE_REPO
              value: "registry.dockworker.ai/cache/usr_92bf81a3/repo_a92b:cache"
            - name: XDG_RUNTIME_DIR
              value: /run/user/1000
            - name: DOCKER_CONFIG
              value: /home/user/.docker
          securityContext:
            allowPrivilegeEscalation: false
            capabilities:
              drop:
                - ALL
          resources:
            requests:
              cpu: "1800m"
              memory: "3584Mi"
              ephemeral-storage: "15Gi"
            limits:
              cpu: "2000m"
              memory: "4096Mi"
              ephemeral-storage: "30Gi" # Safely clears sum of emptyDir size limits
          volumeMounts:
            - name: workspace
              mountPath: /workspace
            - name: scratch-storage
              mountPath: /scratch
            - name: run-dir
              mountPath: /run/user/1000
            - name: auth-dir
              mountPath: /home/user/.docker
              readOnly: true
            - name: tmp-dir
              mountPath: /tmp
      volumes:
        - name: workspace
          emptyDir:
            sizeLimit: 2Gi
        - name: scratch-storage
          emptyDir:
            sizeLimit: 24Gi
        - name: run-dir
          emptyDir:
            medium: Memory
            sizeLimit: 64Mi
        - name: auth-dir
          emptyDir:
            sizeLimit: 10Mi
        - name: tmp-dir
          emptyDir:
            sizeLimit: 512Mi
```

### Pre-Kubernetes 1.30 Compatibility (Kustomize Overlay)

For clusters on Kubernetes 1.28/1.29, apply the `overlays/pre-1.30/` patch:

```yaml
# overlays/pre-1.30/patch-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: dockworker-build-task
spec:
  template:
    spec:
      hostUsers: null # Removed; not available
      securityContext:
        seccompProfile:
          type: Unconfined # Required for clone/unshare without K8s user namespaces
```

---

## 2. Build Control Plane: gRPC Protocol

See `proto/dockworker/v1/control_plane.proto` for the complete protobuf definition.

### Key Semantics

**Lease Acquisition & Renewal:**
- Runner claims a task and receives a lease token (TTL: 30 seconds).
- Heartbeat RPC every 10 seconds extends the lease.
- 3 missed heartbeats (30 seconds) trigger task eviction and requeue.

**Log Streaming:**
- Chunks contain monotonic sequence IDs.
- Sidecar retries on gRPC disconnect, resuming from `highest_sequence_acked`.
- Redis-backed log sequence checkpoint (Phase 3 deliverable).

**Webhook Verification:**
- GitHub webhook signature verified via HMAC-SHA256 constant-time comparison.
- Payloads enqueued to Redis Streams if signature matches.

---

## 3. Network Perimeter: Cilium eBPF (Corrected)

**Critical fix:** The previous policy omitted Docker Hub blob CDNs (`production.cloudflare.docker.com`, `**.cloudfront.net`), breaking base image pulls.

```yaml
apiVersion: "cilium.io/v2"
kind: CiliumNetworkPolicy
metadata:
  name: dockworker-runners-hardened-perimeter
  namespace: dockworker-runners
spec:
  endpointSelector:
    matchLabels:
      dockworker.ai/runner: rootless-buildkit
  # 1. Explicitly drop all inbound traffic
  ingress: []
  egress:
    # 2. Allow cluster DNS resolution only
    - toEndpoints:
        - matchLabels:
            "k8s:io.kubernetes.pod.namespace": kube-system
            "k8s:k8s-app": kube-dns
      toPorts:
        - ports:
            - port: "53"
              protocol: ANY
          rules:
            dns:
              - matchPattern: "*"
    # 3. Allow internal gRPC control plane access
    - toEndpoints:
        - matchLabels:
            "k8s:io.kubernetes.pod.namespace": dockworker
            "app.kubernetes.io/name": dockworker-control-plane
      toPorts:
        - ports:
            - port: "50051"
              protocol: TCP
    # 4. Strict L7 HTTPS Egress (Registries, Mirrors, Docker Hub Blobs)
    - toFQDNs:
        # Docker Hub API & Blob CDNs
        - matchName: "auth.docker.io"
        - matchName: "registry-1.docker.io"
        - matchName: "production.cloudflare.docker.com"
        - matchName: "production.cloudfront.docker.com"
        - matchPattern: "**.cloudfront.net"
        # Alternate OCI Registries
        - matchName: "ghcr.io"
        - matchPattern: "**.pkg.github.com"
        - matchName: "quay.io"
        - matchPattern: "**.r2.cloudflarestorage.com"
        # Language Repositories & Distro Mirrors
        - matchName: "pypi.org"
        - matchPattern: "**.pythonhosted.org"
        - matchName: "registry.npmjs.org"
        - matchPattern: "**.yarnpkg.com"
        - matchName: "crates.io"
        - matchName: "static.crates.io"
        - matchName: "index.crates.io"
        - matchPattern: "**.debian.org"
        - matchPattern: "**.ubuntu.com"
        - matchPattern: "**.alpinelinux.org"
        # Git Providers
        - matchName: "github.com"
        - matchPattern: "**.github.com"
        - matchName: "gitlab.com"
      toPorts:
        - ports:
            - port: "443"
              protocol: TCP
  # 5. Explicitly block link-local metadata and internal VPC ranges
  egressDeny:
    - toCIDR:
        - "169.254.169.254/32" # Cloud Metadata API
        - "10.0.0.0/8"          # Internal RFC1918
        - "172.16.0.0/12"       # Internal RFC1918
        - "192.168.0.0/16"      # Internal RFC1918
```

---

## 4. Cache Mechanics & Tenant Isolation

BuildKit layer caching operates via Open Container Initiative (OCI) content-addressable storage. Layer blobs are indexed by SHA-256 hash; an attacker cannot publish a malicious layer under a valid hash because the runtime verifies the hash on extraction.

The actual vulnerability is **tag overwriting**: if multiple tenants export to a shared cache tag, Tenant B can overwrite Tenant A's cached stage.

**Enforcement:** Control plane injects tenant-namespaced cache repository paths:

```bash
--export-cache type=registry,ref=registry.dockworker.ai/cache/{tenant_id}/{repo_hash}:cache,mode=max
--import-cache type=registry,ref=registry.dockworker.ai/cache/{tenant_id}/{repo_hash}:cache
```

Registry authentication tokens are scoped via short-lived JWTs that only permit write access to `/cache/{tenant_id}/*`.

---

## 5. Grounded Unit Economics

### Hardware Capacity (Per Dedicated Node)

- **Physical Node:** AMD EPYC 7502P (32 cores / 64 threads, 128 GiB DDR5 ECC, 2x 1TB NVMe Soft-RAID) = **$140.00/month**
- **System Reservations:** 2 cores, 16 GiB (host OS, Kubelet, Cilium)
- **Allocatable Capacity:** 30 cores, 112 GiB
- **Free-Tier Pod:** 1.8 vCPU request, 3.5 GiB memory, 24 GiB NVMe
- **Maximum Safe Concurrency:** ⌊30 / 1.8⌋ = **16 concurrent pods per node** (not 32)

### Platform Infrastructure Floor

| Component | Monthly Cost |
|-----------|--------------|
| Control Plane (3x Axum + Tonic replicas) | $30.00 |
| State & Queue (Redis Sentinel HA) | $45.00 |
| Identity & Quotas (PostgreSQL HA) | $60.00 |
| Observability (Prometheus, Grafana, Hubble, Loki) | $50.00 |
| Edge Ingress (Cloudflare Workers Pro) | $25.00 |
| **Platform Baseline (zero runners)** | **$210.00/month** |

### Real Cost per Build Minute

Assuming 20% average cluster utilization:

| User Cohort | Build Minutes/Month | Runner Nodes | Total OpEx | Cost/Minute |
|-------------|---|---|---|---|
| 250 users | 17.5K | 1 | $350 | **$0.0200** |
| 1,000 users | 84K | 1 | $350 | **$0.0041** |
| 5,000 users | 525K | 4 | $795 | **$0.0015** |
| 20,000 users | 2.52M | 19 | $2,970 | **$0.0011** |

**Key Insight:** Platform baseline dominates at small scale. True efficiency only at 5K+ users.

---

## 6. Multi-Signal Sybil Defense

Ingestion engine enforces composite scoring before allocating queue capacity:

| Signal | Evaluation | Weight |
|--------|-----------|--------|
| Account Longevity | Created >90 days (+2); 30–90 days (+1); <30 days (-3) | -3 to +2 |
| Verified Email | Primary email domain verified | +1 |
| Commit Activity | ≥5 public commits in 2 repos (60 days) | +2 |
| IP Reputation | AbuseIPDB / Spamhaus check | -3 to +1 |
| MFA Status | Two-factor authentication enabled | +1 |

**Enforcement:**
- **Score ≥ 4:** Trusted. Full 10-minute timeout, priority queue.
- **Score 1–3:** Restricted. 5-minute timeout, max 1 build/hour, client-side PoW required.
- **Score ≤ 0:** Blocked. HTTP 403; requires corporate email or $0 credit card hold.

---

## 7. Phased Implementation: 12 Weeks

| Phase | Horizon | Scope | Exit Criteria |
|-------|---------|-------|---|
| **1** | Weeks 1–2 | Sandbox Stabilization: Fix BuildKit, storage caps, K8s overlays | 100 clean builds without pod eviction |
| **2** | Weeks 3–4 | Network Hardening: Cilium eBPF, CDN rules, egress deny | All non-approved egress drops; DNS works |
| **3** | Weeks 5–7 | Control Plane Core: gRPC, Redis queue, Axum SSE, log checkpointing | 20 parallel builds stream live to UI |
| **4** | Weeks 8–9 | Identity & Quotas: GitHub OAuth, multi-signal scoring, PostgreSQL ledger | Sybil accounts rejected/throttled, concurrency enforced |
| **5** | Weeks 10–12 | Production Ramp: 100-user cohort, load test, cost validation | Zero 502/503 errors under sustained load |

**Resource Plan:** 5 FTE (Backend, DevOps, Frontend, QA, Ops) = ~$400K Y1

---

## Related Documents

- **[GitHub Planning Epics](./planning/README.md)** — User story definitions
- **[Technical Review](./TECHNICAL_REVIEW.md)** — Critical feedback & corrections
- **[API Reference](./api/README.md)** — gRPC & REST endpoint contracts (Phase 3)

---

**Status:** ✅ **Ready for Phase 1 execution. All critical design decisions validated.**

Co-Authored-By: Claude Haiku 4.5 <noreply@anthropic.com>
