# dockworker.ai Kubernetes Infrastructure (Phase 1)

This directory contains Kubernetes manifests for the dockworker.ai sandbox environment. Phase 1 focuses on stabilizing the build Job execution with proper resource quotas, network policies, and security contexts.

## Structure

```
k8s/
├── namespace.yaml              # dockworker-runners namespace
├── serviceaccount.yaml         # Service account for build pods
├── resource-quota.yaml         # Memory/CPU/storage quotas
├── network-policy.yaml         # Egress filtering for builds
├── test-job.yaml              # Sample build job for testing
├── kustomization.yaml         # Base kustomization
└── overlays/
    ├── k8s-1.30/              # K8s 1.30+ with user namespaces
    │   ├── kustomization.yaml
    │   └── job-patch.yaml
    └── k8s-pre-1.30/          # Pre-1.30 fallback (AppArmor/SELinux)
        ├── kustomization.yaml
        └── job-patch.yaml
```

## Deployment

### Prerequisites

- Kubernetes 1.25+ cluster
- kubectl configured and authenticated
- 32+ CPU cores, 64Gi+ memory available for build nodes

### Deploy Base Infrastructure

```bash
# Deploy namespace, RBAC, quotas, network policies
kubectl apply -k k8s/

# Verify namespace and quotas
kubectl get namespace dockworker-runners
kubectl describe resourcequota dockworker-quota -n dockworker-runners
```

### Deploy for Kubernetes 1.30+

```bash
# Deploy with user namespace isolation enabled
kubectl apply -k k8s/overlays/k8s-1.30/

# Verify pods launch with hostUsers: false
kubectl get pods -n dockworker-runners -o jsonpath='{.items[0].spec.hostUsers}'
```

### Deploy for Pre-1.30 Clusters

```bash
# Deploy with AppArmor/SELinux fallback
kubectl apply -k k8s/overlays/k8s-pre-1.30/

# Verify AppArmor annotation
kubectl get pods -n dockworker-runners -o jsonpath='{.items[0].metadata.annotations}'
```

## Phase 1 Testing: 100 Clean Builds

Exit criterion: **100 consecutive builds without pod eviction**

### Run Single Test Build

```bash
kubectl apply -f k8s/test-job.yaml
kubectl wait --for=condition=complete job/dockworker-test-build -n dockworker-runners --timeout=10m
kubectl logs -n dockworker-runners -l job-name=dockworker-test-build --all-containers=true
```

### Run 100 Test Builds (Validation Script)

```bash
#!/bin/bash
set -e

NAMESPACE="dockworker-runners"
TOTAL_BUILDS=100
FAILED=0

for i in $(seq 1 $TOTAL_BUILDS); do
  JOB_NAME="test-build-$(printf '%03d' $i)"
  
  # Create job with unique name
  kubectl create -f - <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: $JOB_NAME
  namespace: $NAMESPACE
spec:
  backoffLimit: 0
  activeDeadlineSeconds: 600
  ttlSecondsAfterFinished: 120
  template:
    metadata:
      labels:
        dockworker.ai/test: "true"
    spec:
      restartPolicy: Never
      automountServiceAccountToken: false
      hostUsers: false
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: builder
          image: moby/buildkit:v0.26.2-rootless
          command: ["buildctl-daemonless.sh"]
          args:
            - build
            - "--frontend=dockerfile.v0"
            - "--local=context=/workspace"
            - "--local=dockerfile=/workspace"
            - "--output=type=oci,dest=/tmp/output.tar"
          volumeMounts:
            - name: workspace
              mountPath: /workspace
            - name: scratch
              mountPath: /scratch
            - name: run
              mountPath: /run/user/1000
          resources:
            requests:
              cpu: "1500m"
              memory: "2Gi"
              ephemeral-storage: "10Gi"
            limits:
              cpu: "2000m"
              memory: "4Gi"
              ephemeral-storage: "18Gi"
      volumes:
        - name: workspace
          emptyDir:
            sizeLimit: 2Gi
        - name: scratch
          emptyDir:
            sizeLimit: 24Gi
        - name: run
          emptyDir:
            medium: Memory
            sizeLimit: 64Mi
EOF

  # Wait for job completion
  if kubectl wait --for=condition=complete job/$JOB_NAME -n $NAMESPACE --timeout=10m 2>/dev/null; then
    echo "✓ Build $i/$TOTAL_BUILDS succeeded"
  else
    echo "✗ Build $i/$TOTAL_BUILDS failed or timed out"
    FAILED=$((FAILED + 1))
    # Check for evictions
    EVICTIONS=$(kubectl get events -n $NAMESPACE --field-selector involvedObject.name=$JOB_NAME --field-selector reason=Evicted | wc -l)
    if [ $EVICTIONS -gt 0 ]; then
      echo "  ⚠️  Pod evicted - storage/memory pressure detected"
    fi
  fi
done

echo ""
echo "=== Phase 1 Test Results ==="
echo "Total: $TOTAL_BUILDS, Passed: $((TOTAL_BUILDS - FAILED)), Failed: $FAILED"
if [ $FAILED -eq 0 ]; then
  echo "✅ Phase 1 Exit Criteria Met: 100 clean builds without eviction"
else
  echo "❌ Phase 1 Failed: $FAILED builds did not complete cleanly"
  exit 1
fi
```

### Monitor Resource Usage

```bash
# Watch node resource pressure
watch kubectl top nodes

# Check for eviction reasons
kubectl get events -n dockworker-runners \
  --field-selector reason=Evicted \
  --sort-by='.lastTimestamp'

# Check build pod logs for OOM/disk issues
kubectl logs -n dockworker-runners -l dockworker.ai/test=true \
  --tail=50 --all-containers=true
```

## Phase 1 Exit Criteria

- ✅ 100 consecutive build jobs succeed without pod eviction
- ✅ No memory pressure events (MemoryPressure: false)
- ✅ No disk pressure events (DiskPressure: false)
- ✅ All builds complete within 600-second timeout
- ✅ Resource quota enforcement verified
- ✅ Network policy allows image registry pulls only
- ✅ Security context enforced (runAsNonRoot: true, no capabilities)

## Troubleshooting

### Pod Eviction During Build

**Symptom:** `Status: Failed, Reason: Evicted, Message: Pod ephemeral local storage usage exceeds the total limit`

**Solution:**
1. Check node disk usage: `kubectl top nodes`
2. Increase ephemeral-storage limit in resource-quota.yaml
3. Add more build nodes if consistent evictions occur
4. Review Job's volume size limits (workspace: 2Gi, scratch: 24Gi, etc.)

### Image Pull Backoff

**Symptom:** `ImagePullBackOff` on moby/buildkit image

**Solution:**
1. Verify network policy allows egress to registries
2. Check DNS resolution: `kubectl exec -it pod/test -- nslookup registry-1.docker.io`
3. Verify node internet connectivity

### BuildKit Timeout

**Symptom:** Job fails after 600 seconds (activeDeadlineSeconds)

**Solution:**
1. Increase activeDeadlineSeconds if builds legitimately take longer
2. Profile build time: check buildctl logs for layer caching efficiency
3. Ensure cache repository is configured for faster rebuilds (Phase 2)

## Next Steps

- **Phase 2 (Weeks 3–4):** Network Hardening with Cilium eBPF
- **Phase 3 (Weeks 5–7):** Control Plane Core (gRPC, Redis, Axum SSE)
- **Phase 4 (Weeks 8–9):** Identity & Quotas (GitHub OAuth, Sybil defense)
- **Phase 5 (Weeks 10–12):** Production Ramp (100-user cohort, load testing)

## References

- [Architecture Specification](../docs/ARCHITECTURE.md)
- [Kubernetes User Namespaces](https://kubernetes.io/docs/tasks/administer-cluster/user-namespaces/)
- [BuildKit Rootless Mode](https://github.com/moby/buildkit/blob/master/docs/rootless.md)
