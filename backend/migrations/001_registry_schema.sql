-- Phase 2: Registry schema for OCI Distribution v1.1 compliance
-- Tables: manifests, blob_refs, chunked_uploads
-- Supports free-tier TTL purge, multi-tenant isolation, content-addressable storage

CREATE TABLE manifests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id TEXT NOT NULL,
    tenant_tier TEXT NOT NULL CHECK (tenant_tier IN ('free', 'pro', 'team', 'enterprise')),
    repository_name TEXT NOT NULL,
    reference TEXT NOT NULL,  -- Can be tag or digest (sha256:abc...)
    digest TEXT NOT NULL,  -- Canonical digest (sha256:...)
    content_type TEXT NOT NULL DEFAULT 'application/vnd.docker.distribution.manifest.v2+json',
    manifest_json BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,

    CONSTRAINT unique_manifest_reference UNIQUE (tenant_id, repository_name, reference),
    CONSTRAINT unique_manifest_digest UNIQUE (tenant_id, repository_name, digest),
    CONSTRAINT valid_reference CHECK (
        reference ~ '^[a-z0-9]+:[a-f0-9]{64}$' OR
        reference ~ '^[a-z0-9]+$'
    )
);

CREATE INDEX idx_manifests_tenant_id ON manifests(tenant_id);
CREATE INDEX idx_manifests_tenant_tier ON manifests(tenant_tier);
CREATE INDEX idx_manifests_repository ON manifests(tenant_id, repository_name);
CREATE INDEX idx_manifests_created_at ON manifests(created_at);
CREATE INDEX idx_manifests_deleted_at ON manifests(deleted_at) WHERE deleted_at IS NULL;

-- Blob references: many-to-many between manifests and blobs
-- Tracks which blobs are referenced by which manifests (for GC)
CREATE TABLE blob_refs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manifest_id UUID NOT NULL REFERENCES manifests(id) ON DELETE CASCADE,
    blob_digest TEXT NOT NULL,  -- sha256:...
    layer_index INT NOT NULL,  -- Order within the manifest
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT unique_manifest_blob UNIQUE (manifest_id, blob_digest)
);

CREATE INDEX idx_blob_refs_manifest_id ON blob_refs(manifest_id);
CREATE INDEX idx_blob_refs_blob_digest ON blob_refs(blob_digest);
CREATE INDEX idx_blob_refs_created_at ON blob_refs(created_at);

-- Chunked uploads: tracks in-progress blob uploads
CREATE TABLE chunked_uploads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    upload_uuid TEXT NOT NULL UNIQUE,
    tenant_id TEXT NOT NULL,
    repository_name TEXT NOT NULL,
    expected_digest TEXT,  -- Optional: client can pre-declare expected digest
    current_size BIGINT DEFAULT 0,
    max_size BIGINT DEFAULT 30000000000,  -- 30GB default
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,

    CONSTRAINT valid_digest CHECK (
        expected_digest IS NULL OR expected_digest ~ '^sha256:[a-f0-9]{64}$'
    )
);

CREATE INDEX idx_chunked_uploads_upload_uuid ON chunked_uploads(upload_uuid);
CREATE INDEX idx_chunked_uploads_tenant_id ON chunked_uploads(tenant_id);
CREATE INDEX idx_chunked_uploads_created_at ON chunked_uploads(created_at);
CREATE INDEX idx_chunked_uploads_completed_at ON chunked_uploads(completed_at) WHERE completed_at IS NULL;

-- Blobs: CAS metadata (actual blobs stored on filesystem/S3)
CREATE TABLE blobs (
    digest TEXT PRIMARY KEY,  -- sha256:...
    size BIGINT NOT NULL,
    media_type TEXT DEFAULT 'application/octet-stream',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT valid_digest CHECK (digest ~ '^sha256:[a-f0-9]{64}$')
);

CREATE INDEX idx_blobs_created_at ON blobs(created_at);
CREATE INDEX idx_blobs_last_accessed_at ON blobs(last_accessed_at);

-- Tokens: scoped bearer tokens for push/pull operations
CREATE TABLE tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id TEXT NOT NULL,
    jti TEXT NOT NULL UNIQUE,  -- JWT ID for revocation
    scope TEXT NOT NULL,  -- "repository:builds/free-*:pull,push"
    actions TEXT[] NOT NULL,  -- ARRAY['pull', 'push']
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    revoked_at TIMESTAMP WITH TIME ZONE,

    CONSTRAINT valid_scope CHECK (scope ~ '^repository:[a-z0-9/_*-]+:(pull|push|delete|manage)(,(pull|push|delete|manage))*$')
);

CREATE INDEX idx_tokens_tenant_id ON tokens(tenant_id);
CREATE INDEX idx_tokens_jti ON tokens(jti);
CREATE INDEX idx_tokens_expires_at ON tokens(expires_at);
CREATE INDEX idx_tokens_created_at ON tokens(created_at);
