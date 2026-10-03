use std::path::PathBuf;
use tokio::fs;
use sha2::{Sha256, Digest};

/// Content-Addressable Storage (CAS) backend
///
/// Stores blobs keyed strictly by their SHA-256 digest:
/// /var/lib/dockworker/blobs/sha256/{digest_hex}
///
/// Staging directory for chunked uploads:
/// /var/lib/dockworker/blobs/uploads/{upload_uuid}
pub struct CasBackend {
    blobs_path: PathBuf,
    uploads_path: PathBuf,
}

impl CasBackend {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            blobs_path: base_path.join("sha256"),
            uploads_path: base_path.join("uploads"),
        }
    }

    /// Create a new upload session
    pub async fn create_upload(&self, upload_id: &str) -> Result<PathBuf, std::io::Error> {
        let upload_dir = self.uploads_path.join(upload_id);
        fs::create_dir_all(&upload_dir).await?;
        Ok(upload_dir)
    }

    /// Append chunk to upload staging buffer
    pub async fn append_chunk(&self, upload_id: &str, chunk: &[u8]) -> Result<(), std::io::Error> {
        let staging_file = self.uploads_path.join(upload_id).join("blob");
        // TODO: Append chunk atomically
        Ok(())
    }

    /// Finalize upload: verify digest and move to CAS store
    pub async fn finalize_upload(
        &self,
        upload_id: &str,
        expected_digest: &str,
    ) -> Result<PathBuf, String> {
        let staging_file = self.uploads_path.join(upload_id).join("blob");

        // Read staging buffer and compute digest
        let data = fs::read(&staging_file)
            .await
            .map_err(|e| format!("Failed to read staging: {}", e))?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let computed_digest = format!("sha256:{:x}", hasher.finalize());

        // Verify digest matches
        if computed_digest != expected_digest {
            return Err(format!(
                "Digest mismatch: expected {}, got {}",
                expected_digest, computed_digest
            ));
        }

        // Atomic move to CAS store
        let digest_hex = expected_digest.strip_prefix("sha256:").unwrap_or(expected_digest);
        let cas_path = self.blobs_path.join(digest_hex);

        fs::rename(&staging_file, &cas_path)
            .await
            .map_err(|e| format!("Failed to move to CAS: {}", e))?;

        // Clean up upload directory
        let _ = fs::remove_dir(self.uploads_path.join(upload_id)).await;

        Ok(cas_path)
    }

    /// Check if blob exists in CAS store
    pub async fn blob_exists(&self, digest: &str) -> bool {
        let digest_hex = digest.strip_prefix("sha256:").unwrap_or(digest);
        let path = self.blobs_path.join(digest_hex);
        fs::try_exists(&path).await.unwrap_or(false)
    }

    /// Get blob data from CAS store
    pub async fn get_blob(&self, digest: &str) -> Result<Vec<u8>, std::io::Error> {
        let digest_hex = digest.strip_prefix("sha256:").unwrap_or(digest);
        let path = self.blobs_path.join(digest_hex);
        fs::read(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cas_upload_finalize() {
        let tmpdir = TempDir::new().unwrap();
        let backend = CasBackend::new(tmpdir.path().to_path_buf());

        // Create upload
        let upload_id = "test-upload-001";
        let upload_dir = backend.create_upload(upload_id).await.unwrap();

        // Write blob
        let blob_data = b"test blob content";
        fs::write(upload_dir.join("blob"), blob_data).await.unwrap();

        // Compute expected digest
        let mut hasher = Sha256::new();
        hasher.update(blob_data);
        let digest = format!("sha256:{:x}", hasher.finalize());

        // Finalize
        let cas_path = backend.finalize_upload(upload_id, &digest).await.unwrap();

        // Verify blob exists in CAS
        assert!(backend.blob_exists(&digest).await);
        assert_eq!(fs::read(&cas_path).await.unwrap(), blob_data);
    }
}
