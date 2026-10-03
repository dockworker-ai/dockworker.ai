// Garbage collection and ephemeral purge worker
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::info;

use crate::db::Database;

/// Ephemeral TTL purge worker for free-tier images and cache
///
/// Runs every 10 minutes and:
/// 1. Deletes manifests older than 24-48 hours (free tier)
/// 2. Marks unreferenced blobs for deletion
/// 3. Sweeps orphaned blobs
pub async fn start_gc_worker(_db: Arc<Database>) {
    let mut gc_interval = interval(Duration::from_secs(600)); // 10 minutes

    loop {
        gc_interval.tick().await;

        if let Err(e) = purge_expired_manifests().await {
            tracing::warn!("Manifest purge failed: {}", e);
        }

        if let Err(e) = sweep_orphaned_blobs().await {
            tracing::warn!("Blob sweep failed: {}", e);
        }
    }
}

/// Delete free-tier manifests older than 24 hours
async fn purge_expired_manifests() -> Result<(), String> {
    // TODO: Execute SQL:
    // DELETE FROM manifests
    // WHERE tenant_tier = 'free'
    //   AND created_at < NOW() - INTERVAL '24 hours'
    // RETURNING id;

    let purged = 0; // placeholder
    info!("Purged {} expired free-tier manifests", purged);
    Ok(())
}

/// Garbage collect blobs not referenced by any manifest
async fn sweep_orphaned_blobs() -> Result<(), String> {
    // TODO: Execute SQL:
    // DELETE FROM blob_refs
    // WHERE digest NOT IN (
    //   SELECT DISTINCT blob_digest
    //   FROM manifests
    //   WHERE deleted_at IS NULL
    // );

    let swept = 0; // placeholder
    info!("Swept {} orphaned blobs", swept);
    Ok(())
}
