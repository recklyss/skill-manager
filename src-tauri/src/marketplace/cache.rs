use serde_json::Value;
use sha1::{Digest, Sha1};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct MarketplaceCache {
    root: PathBuf,
}

impl MarketplaceCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn read(&self, namespace: &str, key: &str, ttl_seconds: u64) -> Option<Value> {
        let stored = self.load(namespace, key)?;
        if stored.age_seconds <= ttl_seconds {
            return Some(stored.payload);
        }
        None
    }

    pub fn load(&self, namespace: &str, key: &str) -> Option<StoredPayload> {
        let path = self.path_for(namespace, key)?;
        let text = fs::read_to_string(&path).ok()?;
        let payload: Value = serde_json::from_str(&text).ok()?;
        let fetched_at = payload.get("fetchedAt")?.as_f64()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs_f64();
        let age = now - fetched_at;
        Some(StoredPayload {
            payload: payload.get("payload")?.clone(),
            age_seconds: age.max(0.0) as u64,
        })
    }

    pub fn write(&self, namespace: &str, key: &str, payload: &Value) {
        let Some(path) = self.path_for(namespace, key) else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        let encoded = serde_json::json!({
            "fetchedAt": now,
            "payload": payload,
        });
        let temp = path.with_extension("json.tmp");
        if fs::write(&temp, serde_json::to_string_pretty(&encoded).unwrap_or_default()).is_ok() {
            let _ = fs::rename(temp, path);
        }
    }

    fn path_for(&self, namespace: &str, key: &str) -> Option<PathBuf> {
        let digest = format!("{:x}", Sha1::digest(key.as_bytes()));
        Some(self.root.join(namespace).join(format!("{digest}.json")))
    }
}

pub struct StoredPayload {
    pub payload: Value,
    pub age_seconds: u64,
}
