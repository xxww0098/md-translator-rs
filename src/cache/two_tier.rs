use std::path::PathBuf;
use std::time::Duration;

use crate::cache::{Cache, DiskCache, MemoryCache};
use crate::error::Result;

/// Two-tier cache: hot data in `moka` memory, warm data in `sled` disk.
///
/// Lookup order: memory -> disk -> miss.
/// On a disk hit, the value is promoted into memory.
pub struct TwoTierCache {
    memory: MemoryCache,
    disk: DiskCache,
}

impl TwoTierCache {
    /// Create a new [`TwoTierCache`] with the given disk database path.
    ///
    /// Defaults:
    /// - Memory capacity: 50,000 entries
    /// - TTL: 7 days
    pub fn new(disk_path: PathBuf) -> Result<Self> {
        let seven_days = Duration::from_secs(7 * 24 * 60 * 60);
        let memory = MemoryCache::new(50_000, seven_days);
        let disk = DiskCache::new(disk_path, seven_days)?;
        Ok(Self { memory, disk })
    }
}

#[async_trait::async_trait]
impl Cache for TwoTierCache {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        if let Some(value) = self.memory.get(key).await? {
            return Ok(Some(value));
        }
        if let Some(value) = self.disk.get(key).await? {
            self.memory.set(key, &value).await?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        self.memory.set(key, value).await?;
        self.disk.set(key, value).await?;
        Ok(())
    }
}
