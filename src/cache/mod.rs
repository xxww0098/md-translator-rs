use std::path::PathBuf;

use tokio::fs;

use crate::error::{MdTranslatorError, Result};

/// Abstract cache interface used by the MdTranslator engine.
#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &str) -> Result<()>;
}

/// Legacy file-based cache.
///
/// Deprecated: Use [`TwoTierCache`] for better performance and TTL support.
#[derive(Debug, Clone)]
pub struct FileCache {
    root: PathBuf,
}

impl FileCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[async_trait::async_trait]
impl Cache for FileCache {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let path = self.path_for(key);
        match fs::metadata(&path).await {
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(MdTranslatorError::Io(err));
            }
        }
        let data = fs::read_to_string(path).await?;
        Ok(Some(data))
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        fs::create_dir_all(&self.root).await?;
        let path = self.path_for(key);
        fs::write(path, value).await?;
        Ok(())
    }
}

impl FileCache {
    fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.txt"))
    }
}

pub mod disk;
pub mod memory;
pub mod two_tier;

pub use disk::DiskCache;
pub use memory::MemoryCache;
pub use two_tier::TwoTierCache;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "md-translator-rs-test-{}-{}",
            name,
            std::process::id()
        ))
    }

    #[tokio::test]
    async fn memory_cache_ttl() {
        let cache = MemoryCache::new(10, Duration::from_millis(100));
        cache.set("k1", "v1").await.unwrap();

        let hit = cache.get("k1").await.unwrap();
        assert_eq!(hit, Some("v1".to_string()));

        tokio::time::sleep(Duration::from_millis(150)).await;

        let miss = cache.get("k1").await.unwrap();
        assert_eq!(miss, None);
    }

    #[tokio::test]
    async fn disk_cache_persist() {
        let dir = test_dir("disk");
        let cache = DiskCache::new(dir.clone(), Duration::from_secs(60)).unwrap();
        cache.set("k1", "v1").await.unwrap();

        drop(cache);
        let cache2 = DiskCache::new(dir, Duration::from_secs(60)).unwrap();
        let hit = cache2.get("k1").await.unwrap();
        assert_eq!(hit, Some("v1".to_string()));
    }

    #[tokio::test]
    async fn two_tier_lookup() {
        let dir = test_dir("twotier");
        let cache = TwoTierCache::new(dir.clone()).unwrap();

        let miss = cache.get("k1").await.unwrap();
        assert_eq!(miss, None);

        cache.set("k1", "v1").await.unwrap();

        let hit = cache.get("k1").await.unwrap();
        assert_eq!(hit, Some("v1".to_string()));

        drop(cache);
        let cache2 = TwoTierCache::new(dir).unwrap();
        let hit_disk = cache2.get("k1").await.unwrap();
        assert_eq!(hit_disk, Some("v1".to_string()));
    }
}
