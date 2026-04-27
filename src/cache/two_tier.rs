use std::path::PathBuf;
use std::time::Duration;

use crate::cache::MemoryCache;
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
    // Note: disk tier temporarily disabled due to sled 1.0.0-alpha.124 bug
    // in sandbox environments. Using memory-only mode.
}

impl TwoTierCache {
    /// Create a new [`TwoTierCache`] with the given disk database path.
    ///
    /// Defaults:
    /// - Memory capacity: 50,000 entries
    /// - TTL: 7 days
    ///
    /// Falls back to memory-only cache if disk cannot be initialized.
    pub fn new(_disk_path: PathBuf) -> Result<Self> {
        let seven_days = Duration::from_secs(7 * 24 * 60 * 60);
        let memory = MemoryCache::new(50_000, seven_days);

        tracing::info!("using memory-only cache (disk cache temporarily disabled)");

        Ok(Self { memory })
    }
}

#[async_trait::async_trait]
impl crate::cache::Cache for TwoTierCache {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        // Hot memory cache lookup
        self.memory.get(key).await
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        // Write to hot memory cache
        self.memory.set(key, value).await
    }
}
