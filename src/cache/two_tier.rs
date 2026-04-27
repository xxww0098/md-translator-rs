use std::path::PathBuf;
use std::time::Duration;

use crate::cache::{Cache, MemoryCache};
use crate::error::Result;

pub struct TwoTierCache {
    memory: MemoryCache,
}

impl TwoTierCache {
    pub fn new(_disk_path: PathBuf) -> Result<Self> {
        let seven_days = Duration::from_secs(7 * 24 * 60 * 60);
        let memory = MemoryCache::new(50_000, seven_days);
        Ok(Self { memory })
    }
}

#[async_trait::async_trait]
impl Cache for TwoTierCache {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        self.memory.get(key).await
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        self.memory.set(key, value).await
    }
}
