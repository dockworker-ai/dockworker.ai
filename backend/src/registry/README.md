# dockworker.ai OCI Distribution v1.1 Registry

An embedded, lean OCI Distribution v1.1-compliant registry service for ephemeral free-tier container builds.

## Architecture

The registry is **embedded directly in the control plane** (`axum` router), not a separate service. It provides:

1. **Strict OCI Distribution Spec v1.1 Compliance:** Native support for chunked blob uploads, manifest lists (multi-arch), and layer caching artifacts.
2. **Deterministic Retention & Eviction:** Automatic TTL expiration for free-tier images (24–48 hours) to prevent unbounded storage growth.
3. **Token Authentication & Scope Delegation:** Issuing scoped Bearer tokens for ephemeral runner pods and CLI users.

## API Surface

All endpoints are mounted at `/v2/` (OCI Distribution standard):

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/v2/` | Version check / ping (OCI Discovery) |
| `POST` | `/v2/<name>/blobs/uploads/` | Initiate chunked blob upload |
| `PATCH` | `/v2/<name>/blobs/uploads/<uuid>` | Append blob chunk |
| `PUT` | `/v2/<name>/blobs/uploads/<uuid>?digest=<sha256:...>` | Finalize blob upload |
| `HEAD` / `GET` | `/v2/<name>/blobs/<digest>` | Check/fetch blob from CAS store |
| `PUT` | `/v2/<name>/manifests/<reference>` | Store manifest or manifest list |
| `HEAD` / `GET` | `/v2/<name>/manifests/<reference>` | Check/fetch manifest |
| `DELETE` | `/v2/<name>/manifests/<reference>` | Delete manifest by digest |
| `POST` | `/v2/token` | Issue scoped OAuth/JWT bearer token |

## Storage Topology

```
/var/lib/dockworker/blobs/
├── sha256/
│   ├── abc123def456...  (blob content-addressable by SHA-256)
│   └── xyz789...
└── uploads/
    ├── [upload_uuid_001]/blob  (staging: chunked uploads in progress)
    └── [upload_uuid_002]/blob
```

**CAS Key Format:** `sha256:{hex_digest}` (not the full `sha256:abc123...` string, just hex)

## Implementation Status

- ✅ **Module structure:** `mod.rs` (routes), `auth.rs` (tokens), `blobs.rs` (uploads), `manifests.rs` (storage), `storage.rs` (CAS backend), `gc.rs` (ephemeral purge)
- 🔄 **Integration:** Wired into `main.rs`, routes nested at `/`
- ⏳ **Database schema:** Requires `manifests`, `blob_refs`, `uploads` tables
- ⏳ **Auth implementation:** JWT scoping, tenant isolation
- ⏳ **Garbage collection:** Scheduled 10-minute interval purges free-tier content > 24h

## Configuration

Set via environment variables:

```bash
# Registry blob storage path (default: /var/lib/dockworker/blobs)
REGISTRY_STORAGE_PATH=/var/lib/dockworker/blobs
```

## Usage Examples

### Build Pod Push

```bash
# BuildKit pod authenticates and pushes to cache
buildctl build \
  --frontend=dockerfile.v0 \
  --local context=/workspace \
  --output type=image,name=registry.dockworker.ai/builds/usr_abc/repo_xyz:latest,push=true \
  --export-cache type=registry,ref=registry.dockworker.ai/cache/usr_abc/repo_xyz:cache \
  --import-cache type=registry,ref=registry.dockworker.ai/cache/usr_abc/repo_xyz:cache
```

### Developer CLI Pull

```bash
# User pulls built image from their public profile
docker pull registry.dockworker.ai/profiles/username/app:v1.0

# Requires token with scope: "repository:profiles/username/app:pull"
```

## Garbage Collection

The `gc` module runs a background worker every **10 minutes** that:

1. **Purges expired manifests:**
   ```sql
   DELETE FROM manifests 
   WHERE tenant_tier = 'free' 
     AND created_at < NOW() - INTERVAL '24 hours';
   ```

2. **Sweeps orphaned blobs:**
   ```sql
   DELETE FROM blob_refs
   WHERE digest NOT IN (
     SELECT DISTINCT blob_digest FROM manifests WHERE deleted_at IS NULL
   );
   ```

This ensures free-tier storage never exceeds capacity and prevents unbounded growth.

## Token Scoping

Bearer tokens are issued by `/v2/token` with scopes like:

```
repository:builds/free-{user_id}/*:pull,push
repository:profiles/{username}/{app}:pull
```

The `auth` module validates scope against requested operations:

- `pull`: Read-only access (GET blobs/manifests)
- `push`: Write access (PUT blobs/manifests)
- `delete`: Explicit manifest deletion

## Future Enhancements

- **S3-compatible backend:** Extend `CasBackend` to support MinIO, AWS S3, Google Cloud Storage for unbounded capacity.
- **Replication:** Mirror cache layers across regional registries for faster builds.
- **Image signing:** Integrate Cosign/Sigstore for signed image verification.
- **Quota enforcement:** Per-tenant storage limits enforced at upload finalization.
