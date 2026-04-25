use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sled::Db;

use crate::error::{MdTranslatorError, Result};

/// Persistent disk cache backed by `sled` with TTL support.
pub struct DiskCache {
    db: Mutex<Db>,
    ttl: Duration,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DiskEntry {
    value: String,
    expires_at: u64,
}

fn poison_err<E: std::fmt::Display>(e: E) -> MdTranslatorError {
    MdTranslatorError::Provider(format!("sled mutex poison: {e}"))
}

impl DiskCache {
    pub fn new(path: PathBuf, ttl: Duration) -> Result<Self> {
        let db = sled::open(&path)
            .map_err(|e| MdTranslatorError::Provider(format!("sled open error: {e}")))?;
        Ok(Self {
            db: Mutex::new(db),
            ttl,
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let raw = {
            let db = self.db.lock().map_err(poison_err)?;
            db.get(key.as_bytes())
                .map_err(|e| MdTranslatorError::Provider(format!("sled get error: {e}")))?
        };

        let raw = match raw {
            Some(v) => v,
            None => return Ok(None),
        };

        let entry: DiskEntry = serde_json::from_slice(&raw)
            .map_err(|e| MdTranslatorError::Provider(format!("sled deserialize error: {e}")))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if entry.expires_at < now {
            let db = self.db.lock().map_err(poison_err)?;
            let _ = db.remove(key.as_bytes());
            return Ok(None);
        }

        Ok(Some(entry.value))
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + self.ttl.as_secs();

        let entry = DiskEntry {
            value: value.to_string(),
            expires_at,
        };

        let raw = serde_json::to_vec(&entry)
            .map_err(|e| MdTranslatorError::Provider(format!("sled serialize error: {e}")))?;

        {
            let db = self.db.lock().map_err(poison_err)?;
            db.insert(key.as_bytes(), raw)
                .map_err(|e| MdTranslatorError::Provider(format!("sled insert error: {e}")))?;
            db.flush()
                .map_err(|e| MdTranslatorError::Provider(format!("sled flush error: {e}")))?;
        }

        Ok(())
    }
}
