use std::path::PathBuf;
use std::time::Duration;

use crate::cache::{DiskCache, MemoryCache};
use crate::error::Result;

/// Two-tier cache: hot data in `moka` memory, warm data in sled disk.
///
/// Lookup order: memory -> disk -> miss.
/// On a disk hit, the value is promoted into memory.
///
/// Note: Falls back to memory-only cache when sled disk is unavailable
/// (e.g., in sandbox environments).
pub struct TwoTierCache {
    memory: MemoryCache,
    disk: Option<DiskCache>,
}

impl TwoTierCache {
    /// Create a new [`TwoTierCache`] with the given disk database path.
    ///
    /// Defaults:
    /// - Memory capacity: 50,000 entries
    /// - TTL: 7 days
    ///
    /// Falls back to memory-only cache if disk cannot be initialized.
    pub fn new(disk_path: PathBuf) -> Result<Self> {
        let seven_days = Duration::from_secs(7 * 24 * 60 * 60);
        let memory = MemoryCache::new(50_000, seven_days);
        let disk = match DiskCache::new(disk_path, seven_days) {
            Ok(cache) => Some(cache),
            Err(err) => {
                tracing::warn!(error = %err, "disk cache unavailable; using memory-only cache");
                None
            }
        };

        Ok(Self { memory, disk })
    }
}

#[async_trait::async_trait]
impl crate::cache::Cache for TwoTierCache {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        // Hot memory cache lookup
        if let Some(value) = self.memory.get(key).await? {
            return Ok(Some(value));
        }

        let Some(disk) = &self.disk else {
            return Ok(None);
        };

        let Some(value) = disk.get(key).await? else {
            return Ok(None);
        };

        self.memory.set(key, &value).await?;
        Ok(Some(value))
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        // Write to hot memory cache
        self.memory.set(key, value).await?;

        if let Some(disk) = &self.disk {
            disk.set(key, value).await?;
        }

        Ok(())
    }
}
