use std::path::{Path, PathBuf};

use tokio::fs;
use uuid::Uuid;

use crate::{StorageError, StorageResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetKey {
    pack_public_id: String,
    filename: String,
}

#[derive(Clone, Debug)]
pub struct LocalAssetStore {
    root: PathBuf,
}

impl AssetKey {
    /// Creates a validated logical asset key.
    ///
    /// # Errors
    ///
    /// Returns an error when either component is empty or contains path separators,
    /// drive separators, parent directory segments, or NUL bytes.
    pub fn new(
        pack_public_id: impl Into<String>,
        filename: impl Into<String>,
    ) -> StorageResult<Self> {
        let pack_public_id = pack_public_id.into();
        let filename = filename.into();
        validate_component(&pack_public_id)?;
        validate_component(&filename)?;
        Ok(Self {
            pack_public_id,
            filename,
        })
    }

    #[must_use]
    pub fn pack_public_id(&self) -> &str {
        &self.pack_public_id
    }

    #[must_use]
    pub fn filename(&self) -> &str {
        &self.filename
    }
}

impl LocalAssetStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Writes asset bytes to the local asset store.
    ///
    /// # Errors
    ///
    /// Returns an error when directories cannot be created, bytes cannot be written,
    /// or the temporary file cannot be renamed into place.
    pub async fn write(&self, key: &AssetKey, bytes: &[u8]) -> StorageResult<()> {
        let dir = self.pack_dir(key);
        fs::create_dir_all(&dir).await?;

        let tmp_path = dir.join(format!(".{}.tmp", Uuid::new_v4()));
        fs::write(&tmp_path, bytes).await?;
        fs::rename(&tmp_path, self.path_for_test(key)).await?;
        Ok(())
    }

    /// Reads asset bytes from the local asset store.
    ///
    /// # Errors
    ///
    /// Returns an error when the asset does not exist or cannot be read.
    pub async fn read(&self, key: &AssetKey) -> StorageResult<Vec<u8>> {
        let path = self.path_for_test(key);
        match fs::read(&path).await {
            Ok(bytes) => Ok(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(StorageError::AssetNotFound { path })
            }
            Err(error) => Err(StorageError::Io(error)),
        }
    }

    /// Deletes an asset from the local asset store.
    ///
    /// # Errors
    ///
    /// Returns an error when the delete operation fails for a reason other than the file being absent.
    pub async fn delete(&self, key: &AssetKey) -> StorageResult<()> {
        match fs::remove_file(self.path_for_test(key)).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(StorageError::Io(error)),
        }
    }

    #[must_use]
    pub fn path_for_test(&self, key: &AssetKey) -> PathBuf {
        self.pack_dir(key).join(key.filename())
    }

    fn pack_dir(&self, key: &AssetKey) -> PathBuf {
        self.root
            .join("assets")
            .join("packs")
            .join(key.pack_public_id())
    }
}

fn validate_component(component: &str) -> StorageResult<()> {
    if component.is_empty() {
        return Err(invalid_asset_key(component, "component must not be empty"));
    }

    if component == "." || component == ".." {
        return Err(invalid_asset_key(
            component,
            "component must not be a relative path segment",
        ));
    }

    if component.contains(['/', '\\', ':', '\0']) {
        return Err(invalid_asset_key(
            component,
            "component must not contain path separators, drive separators, or NUL bytes",
        ));
    }

    if Path::new(component).components().count() != 1 {
        return Err(invalid_asset_key(
            component,
            "component must be a single path segment",
        ));
    }

    Ok(())
}

fn invalid_asset_key(component: &str, reason: &'static str) -> StorageError {
    StorageError::InvalidAssetKey {
        component: component.to_owned(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::{AssetKey, LocalAssetStore};

    #[tokio::test]
    async fn writes_reads_and_deletes_assets() {
        let temp = tempfile::tempdir().unwrap();
        let store = LocalAssetStore::new(temp.path());
        let key = AssetKey::new("pack", "sticker.webp").unwrap();

        store.write(&key, b"image-bytes").await.unwrap();
        assert_eq!(store.read(&key).await.unwrap(), b"image-bytes");

        store.delete(&key).await.unwrap();
        assert!(store.read(&key).await.is_err());
    }

    #[test]
    fn rejects_traversal_asset_keys() {
        assert!(AssetKey::new("..", "sticker.webp").is_err());
        assert!(AssetKey::new("pack/other", "sticker.webp").is_err());
        assert!(AssetKey::new("pack", "../sticker.webp").is_err());
        assert!(AssetKey::new("pack", r"folder\sticker.webp").is_err());
        assert!(AssetKey::new("pack", "C:sticker.webp").is_err());
        assert!(AssetKey::new("pack", "bad\0name.webp").is_err());
    }
}
