use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Content-addressable store for OCI blobs and raw manifests.
///
/// Layout under `base`:
/// - `sha256/{digest}` blob bytes
/// - `uploads/{uuid}/blob` in-progress upload
/// - `manifests/{repo}/sha256/{digest}` manifest bytes
/// - `manifests/{repo}/sha256/{digest}.type` media type
/// - `manifests/{repo}/tags/{tag}` digest the tag points at
pub struct CasBackend {
    base_path: PathBuf,
    blobs_path: PathBuf,
    uploads_path: PathBuf,
}

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    Invalid(String),
    DigestMismatch { expected: String, computed: String },
    Io(std::io::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found"),
            Self::Invalid(message) => write!(f, "{message}"),
            Self::DigestMismatch { expected, computed } => {
                write!(f, "Digest mismatch: expected {expected}, got {computed}")
            }
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

#[derive(Debug)]
pub struct ManifestRecord {
    pub bytes: Vec<u8>,
    pub media_type: String,
    pub digest: String,
}

const DEFAULT_MANIFEST_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";

impl CasBackend {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            blobs_path: base_path.join("sha256"),
            uploads_path: base_path.join("uploads"),
            base_path,
        }
    }

    pub async fn create_upload(&self, upload_id: &str) -> Result<PathBuf, StorageError> {
        validate_upload_id(upload_id)?;
        let upload_dir = self.uploads_path.join(upload_id);
        fs::create_dir_all(&upload_dir)
            .await
            .map_err(StorageError::Io)?;
        Ok(upload_dir)
    }

    /// Append `chunk` to the staging file. The upload directory must already exist.
    pub async fn append_chunk(&self, upload_id: &str, chunk: &[u8]) -> Result<u64, StorageError> {
        validate_upload_id(upload_id)?;
        let dir = self.uploads_path.join(upload_id);
        if !fs::try_exists(&dir).await.map_err(StorageError::Io)? {
            return Err(StorageError::NotFound);
        }
        let staging_file = dir.join("blob");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&staging_file)
            .await
            .map_err(StorageError::Io)?;
        file.write_all(chunk).await.map_err(StorageError::Io)?;
        file.flush().await.map_err(StorageError::Io)?;
        let len = file.metadata().await.map_err(StorageError::Io)?.len();
        Ok(len)
    }

    pub async fn upload_len(&self, upload_id: &str) -> Result<u64, StorageError> {
        validate_upload_id(upload_id)?;
        let dir = self.uploads_path.join(upload_id);
        if !fs::try_exists(&dir).await.map_err(StorageError::Io)? {
            return Err(StorageError::NotFound);
        }
        let staging_file = dir.join("blob");
        match fs::metadata(&staging_file).await {
            Ok(meta) => Ok(meta.len()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
            Err(err) => Err(StorageError::Io(err)),
        }
    }

    /// Hash the staging file and move it into `sha256/`.
    ///
    /// A digest mismatch leaves the staging file in place. An already-stored
    /// blob with the same digest is reused.
    pub async fn finalize_upload(
        &self,
        upload_id: &str,
        expected_digest: &str,
    ) -> Result<PathBuf, StorageError> {
        validate_upload_id(upload_id)?;
        let hex = digest_hex(expected_digest)?;
        let expected = format!("sha256:{hex}");
        let staging_file = self.uploads_path.join(upload_id).join("blob");

        let data = match fs::read(&staging_file).await {
            Ok(data) => data,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound);
            }
            Err(err) => return Err(StorageError::Io(err)),
        };

        let computed = sha256_digest(&data);
        if computed != expected {
            return Err(StorageError::DigestMismatch { expected, computed });
        }

        fs::create_dir_all(&self.blobs_path)
            .await
            .map_err(StorageError::Io)?;
        let cas_path = self.blobs_path.join(&hex);
        if fs::try_exists(&cas_path).await.map_err(StorageError::Io)? {
            let _ = fs::remove_file(&staging_file).await;
            let _ = fs::remove_dir(self.uploads_path.join(upload_id)).await;
            return Ok(cas_path);
        }

        fs::rename(&staging_file, &cas_path)
            .await
            .map_err(StorageError::Io)?;
        let _ = fs::remove_dir(self.uploads_path.join(upload_id)).await;
        Ok(cas_path)
    }

    pub async fn blob_exists(&self, digest: &str) -> bool {
        let Ok(hex) = digest_hex(digest) else {
            return false;
        };
        fs::try_exists(self.blobs_path.join(hex))
            .await
            .unwrap_or(false)
    }

    pub async fn blob_len(&self, digest: &str) -> Result<u64, StorageError> {
        let hex = digest_hex(digest)?;
        match fs::metadata(self.blobs_path.join(hex)).await {
            Ok(meta) => Ok(meta.len()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(StorageError::NotFound),
            Err(err) => Err(StorageError::Io(err)),
        }
    }

    pub async fn get_blob(&self, digest: &str) -> Result<Vec<u8>, StorageError> {
        let hex = digest_hex(digest)?;
        match fs::read(self.blobs_path.join(hex)).await {
            Ok(bytes) => Ok(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(StorageError::NotFound),
            Err(err) => Err(StorageError::Io(err)),
        }
    }

    pub async fn put_manifest(
        &self,
        repo: &str,
        reference: &str,
        media_type: &str,
        body: &[u8],
    ) -> Result<String, StorageError> {
        validate_repo(repo)?;
        let digest = sha256_digest(body);
        let hex = digest_hex(&digest)?;
        let media_type = if media_type.is_empty() {
            DEFAULT_MANIFEST_TYPE
        } else {
            media_type
        };
        if media_type.chars().any(|c| c.is_control()) {
            return Err(StorageError::Invalid("invalid media type".into()));
        }

        if reference.starts_with("sha256:") {
            let wanted = digest_hex(reference)?;
            if wanted != hex {
                return Err(StorageError::DigestMismatch {
                    expected: format!("sha256:{wanted}"),
                    computed: digest,
                });
            }
        } else {
            validate_tag(reference)?;
        }

        let sha_dir = self.manifest_dir(repo)?.join("sha256");
        fs::create_dir_all(&sha_dir)
            .await
            .map_err(StorageError::Io)?;
        let blob_path = sha_dir.join(&hex);
        let tmp_path = sha_dir.join(format!("{hex}.tmp"));
        fs::write(&tmp_path, body).await.map_err(StorageError::Io)?;
        fs::rename(&tmp_path, &blob_path)
            .await
            .map_err(StorageError::Io)?;
        fs::write(sha_dir.join(format!("{hex}.type")), media_type.as_bytes())
            .await
            .map_err(StorageError::Io)?;

        if !reference.starts_with("sha256:") {
            let tags = self.manifest_dir(repo)?.join("tags");
            fs::create_dir_all(&tags).await.map_err(StorageError::Io)?;
            fs::write(tags.join(reference), digest.as_bytes())
                .await
                .map_err(StorageError::Io)?;
        }

        Ok(digest)
    }

    pub async fn get_manifest(
        &self,
        repo: &str,
        reference: &str,
    ) -> Result<ManifestRecord, StorageError> {
        validate_repo(repo)?;
        let digest = self.resolve_manifest_digest(repo, reference).await?;
        let hex = digest_hex(&digest)?;
        let sha_dir = self.manifest_dir(repo)?.join("sha256");
        let bytes = match fs::read(sha_dir.join(&hex)).await {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound);
            }
            Err(err) => return Err(StorageError::Io(err)),
        };
        let media_type = match fs::read_to_string(sha_dir.join(format!("{hex}.type"))).await {
            Ok(value) => {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    DEFAULT_MANIFEST_TYPE.to_string()
                } else {
                    trimmed
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                DEFAULT_MANIFEST_TYPE.to_string()
            }
            Err(err) => return Err(StorageError::Io(err)),
        };
        Ok(ManifestRecord {
            bytes,
            media_type,
            digest,
        })
    }

    pub async fn delete_manifest(&self, repo: &str, reference: &str) -> Result<(), StorageError> {
        validate_repo(repo)?;
        if reference.starts_with("sha256:") {
            let digest = format!("sha256:{}", digest_hex(reference)?);
            let hex = digest_hex(&digest)?;
            let sha_dir = self.manifest_dir(repo)?.join("sha256");
            let blob_path = sha_dir.join(&hex);
            if !fs::try_exists(&blob_path).await.map_err(StorageError::Io)? {
                return Err(StorageError::NotFound);
            }
            fs::remove_file(&blob_path)
                .await
                .map_err(StorageError::Io)?;
            let _ = fs::remove_file(sha_dir.join(format!("{hex}.type"))).await;
            self.remove_tags_pointing_at(repo, &digest).await?;
            return Ok(());
        }

        validate_tag(reference)?;
        let tag_path = self.manifest_dir(repo)?.join("tags").join(reference);
        match fs::remove_file(&tag_path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(StorageError::NotFound),
            Err(err) => Err(StorageError::Io(err)),
        }
    }

    async fn resolve_manifest_digest(
        &self,
        repo: &str,
        reference: &str,
    ) -> Result<String, StorageError> {
        if reference.starts_with("sha256:") {
            let hex = digest_hex(reference)?;
            return Ok(format!("sha256:{hex}"));
        }
        validate_tag(reference)?;
        let tag_path = self.manifest_dir(repo)?.join("tags").join(reference);
        let text = match fs::read_to_string(&tag_path).await {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound);
            }
            Err(err) => return Err(StorageError::Io(err)),
        };
        let digest = text.trim();
        let hex = digest_hex(digest)?;
        Ok(format!("sha256:{hex}"))
    }

    async fn remove_tags_pointing_at(&self, repo: &str, digest: &str) -> Result<(), StorageError> {
        let tags = self.manifest_dir(repo)?.join("tags");
        let mut entries = match fs::read_dir(&tags).await {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(StorageError::Io(err)),
        };
        while let Some(entry) = entries.next_entry().await.map_err(StorageError::Io)? {
            let path = entry.path();
            if let Ok(text) = fs::read_to_string(&path).await {
                if text.trim() == digest {
                    let _ = fs::remove_file(&path).await;
                }
            }
        }
        Ok(())
    }

    fn manifest_dir(&self, repo: &str) -> Result<PathBuf, StorageError> {
        validate_repo(repo)?;
        let mut path = self.base_path.join("manifests");
        for segment in repo.split('/') {
            path = path.join(segment);
        }
        Ok(path)
    }
}

pub fn sha256_digest(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("sha256:{:x}", hasher.finalize())
}

pub fn validate_repo(name: &str) -> Result<(), StorageError> {
    if name.is_empty() || name.len() > 256 {
        return Err(StorageError::Invalid("invalid repository name".into()));
    }
    for segment in name.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(StorageError::Invalid("invalid repository name".into()));
        }
        if !segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
        {
            return Err(StorageError::Invalid("invalid repository name".into()));
        }
        if segment.len() > 128 {
            return Err(StorageError::Invalid("invalid repository name".into()));
        }
    }
    Ok(())
}

pub fn validate_tag(tag: &str) -> Result<(), StorageError> {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        return Err(StorageError::Invalid("invalid tag".into()));
    };
    if !(first.is_ascii_alphanumeric() || first == '_') || tag.len() > 128 {
        return Err(StorageError::Invalid("invalid tag".into()));
    }
    if !tag
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
    {
        return Err(StorageError::Invalid("invalid tag".into()));
    }
    Ok(())
}

pub fn validate_upload_id(id: &str) -> Result<(), StorageError> {
    if id.is_empty() || id.len() > 80 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(StorageError::Invalid("invalid upload id".into()));
    }
    Ok(())
}

/// Normalize `sha256:<64 hex>` (or a bare hex digest) to lowercase hex.
pub fn digest_hex(digest: &str) -> Result<String, StorageError> {
    let hex = digest.strip_prefix("sha256:").unwrap_or(digest);
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(StorageError::Invalid(format!("invalid digest: {digest}")));
    }
    Ok(hex.to_ascii_lowercase())
}

pub fn staging_blob(base: &Path, upload_id: &str) -> PathBuf {
    base.join("uploads").join(upload_id).join("blob")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cas_upload_finalize() {
        let dir = TempDir::new().unwrap();
        let cas = CasBackend::new(dir.path().to_path_buf());
        let id = "upload-1";
        cas.create_upload(id).await.unwrap();
        let payload = b"hello registry";
        let len = cas.append_chunk(id, payload).await.unwrap();
        assert_eq!(len, payload.len() as u64);

        let digest = sha256_digest(payload);
        let path = cas.finalize_upload(id, &digest).await.unwrap();
        assert!(path.exists());
        assert_eq!(cas.get_blob(&digest).await.unwrap(), payload);
        assert!(cas.blob_exists(&digest).await);
        assert_eq!(cas.blob_len(&digest).await.unwrap(), payload.len() as u64);
        assert!(!staging_blob(dir.path(), id).exists());
    }

    #[tokio::test]
    async fn test_append_concatenates_chunks() {
        let dir = TempDir::new().unwrap();
        let cas = CasBackend::new(dir.path().to_path_buf());
        cas.create_upload("u").await.unwrap();
        cas.append_chunk("u", b"hel").await.unwrap();
        cas.append_chunk("u", b"lo").await.unwrap();
        let digest = sha256_digest(b"hello");
        cas.finalize_upload("u", &digest).await.unwrap();
        assert_eq!(cas.get_blob(&digest).await.unwrap(), b"hello");
    }

    #[tokio::test]
    async fn test_digest_mismatch_keeps_staging() {
        let dir = TempDir::new().unwrap();
        let cas = CasBackend::new(dir.path().to_path_buf());
        cas.create_upload("u").await.unwrap();
        cas.append_chunk("u", b"hello registry").await.unwrap();
        let bogus = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        let err = cas.finalize_upload("u", bogus).await.unwrap_err();
        assert!(matches!(err, StorageError::DigestMismatch { .. }));
        assert!(staging_blob(dir.path(), "u").exists());
        assert!(!cas.blob_exists(bogus).await);
    }

    #[tokio::test]
    async fn test_finalize_creates_sha256_directory() {
        let dir = TempDir::new().unwrap();
        let cas = CasBackend::new(dir.path().to_path_buf());
        cas.create_upload("u").await.unwrap();
        cas.append_chunk("u", b"x").await.unwrap();
        let digest = sha256_digest(b"x");
        assert!(!dir.path().join("sha256").exists());
        cas.finalize_upload("u", &digest).await.unwrap();
        assert!(dir.path().join("sha256").is_dir());
    }

    #[tokio::test]
    async fn test_manifest_tag_and_digest_round_trip() {
        let dir = TempDir::new().unwrap();
        let cas = CasBackend::new(dir.path().to_path_buf());
        let body = br#"{"schemaVersion":2,"layers":[]}"#;
        let media = "application/vnd.oci.image.manifest.v1+json";
        let digest = cas
            .put_manifest("free/cache/demo", "v1.0", media, body)
            .await
            .unwrap();
        assert_eq!(digest, sha256_digest(body));

        let by_tag = cas.get_manifest("free/cache/demo", "v1.0").await.unwrap();
        assert_eq!(by_tag.bytes, body);
        assert_eq!(by_tag.media_type, media);
        assert_eq!(by_tag.digest, digest);

        let by_digest = cas.get_manifest("free/cache/demo", &digest).await.unwrap();
        assert_eq!(by_digest.bytes, body);

        cas.delete_manifest("free/cache/demo", "v1.0")
            .await
            .unwrap();
        assert!(matches!(
            cas.get_manifest("free/cache/demo", "v1.0")
                .await
                .unwrap_err(),
            StorageError::NotFound
        ));
        assert_eq!(
            cas.get_manifest("free/cache/demo", &digest)
                .await
                .unwrap()
                .bytes,
            body
        );
    }

    #[tokio::test]
    async fn test_rejects_path_traversal_repo() {
        let dir = TempDir::new().unwrap();
        let cas = CasBackend::new(dir.path().to_path_buf());
        let err = cas
            .put_manifest("free/../secrets", "latest", "", b"{}")
            .await
            .unwrap_err();
        assert!(matches!(err, StorageError::Invalid(_)));
    }
}
