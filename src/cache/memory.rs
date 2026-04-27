use std::time::Duration;

use moka::future::Cache as MokaCache;

use crate::error::Result;

/// In-memory cache backed by `moka` with TTL support.
pub struct MemoryCache {
    inner: MokaCache<String, String>,
}

impl MemoryCache {
    pub fn new(max_capacity: u64, ttl: Duration) -> Self {
        let inner = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();
        Self { inner }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.inner.get(key).await)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        self.inner.insert(key.to_string(), value.to_string()).await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::cache::Cache for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        self.get(key).await
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        self.set(key, value).await
    }
}
