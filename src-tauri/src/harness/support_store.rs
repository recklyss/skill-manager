use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HarnessSupportPreferences {
    pub disabled_harnesses: Vec<String>,
}

impl HarnessSupportPreferences {
    pub fn is_enabled(&self, harness: &str) -> bool {
        !self.disabled_harnesses.iter().any(|item| item == harness)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SupportStorePayload {
    #[serde(rename = "disabledHarnesses", default)]
    disabled_harnesses: Vec<String>,
}

#[derive(Debug, Error)]
pub enum SupportStoreError {
    #[error("failed to read support preferences: {0}")]
    Read(String),
    #[error("failed to write support preferences: {0}")]
    Write(String),
}

#[derive(Clone)]
pub struct HarnessSupportStore {
    path: PathBuf,
    lock: std::sync::Arc<Mutex<()>>,
}

impl HarnessSupportStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: std::sync::Arc::new(Mutex::new(())),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<HarnessSupportPreferences, SupportStoreError> {
        if !self.path.is_file() {
            return Ok(HarnessSupportPreferences::default());
        }

        let raw = fs::read_to_string(&self.path)
            .map_err(|error| SupportStoreError::Read(error.to_string()))?;
        let payload: SupportStorePayload = serde_json::from_str(&raw)
            .map_err(|error| SupportStoreError::Read(error.to_string()))?;

        let mut disabled = BTreeSet::new();
        for item in payload.disabled_harnesses {
            if !item.is_empty() {
                disabled.insert(item);
            }
        }

        Ok(HarnessSupportPreferences {
            disabled_harnesses: disabled.into_iter().collect(),
        })
    }

    pub fn set_enabled(
        &self,
        harness: &str,
        enabled: bool,
    ) -> Result<HarnessSupportPreferences, SupportStoreError> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| SupportStoreError::Write("lock poisoned".into()))?;

        let mut disabled: BTreeSet<String> = self.load()?.disabled_harnesses.into_iter().collect();
        if enabled {
            disabled.remove(harness);
        } else {
            disabled.insert(harness.to_string());
        }

        let next = HarnessSupportPreferences {
            disabled_harnesses: disabled.into_iter().collect(),
        };
        self.write_locked(&next)?;
        Ok(next)
    }

    pub fn enabled_harnesses(
        &self,
        supported_harnesses: &[&str],
    ) -> Result<Vec<String>, SupportStoreError> {
        let preferences = self.load()?;
        Ok(supported_harnesses
            .iter()
            .filter(|harness| preferences.is_enabled(harness))
            .map(|harness| (*harness).to_string())
            .collect())
    }

    fn write_locked(&self, preferences: &HarnessSupportPreferences) -> Result<(), SupportStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| SupportStoreError::Write(error.to_string()))?;
        }

        let payload = SupportStorePayload {
            disabled_harnesses: preferences.disabled_harnesses.clone(),
        };
        let serialized = serde_json::to_string_pretty(&payload)
            .map_err(|error| SupportStoreError::Write(error.to_string()))?;
        let serialized = format!("{serialized}\n");

        let lock_path = self.path.with_extension("lock");
        let _lock_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)
            .map_err(|error| SupportStoreError::Write(error.to_string()))?;

        let temp_path = self.path.with_extension("tmp");
        {
            let mut file = fs::File::create(&temp_path)
                .map_err(|error| SupportStoreError::Write(error.to_string()))?;
            file.write_all(serialized.as_bytes())
                .map_err(|error| SupportStoreError::Write(error.to_string()))?;
            file.sync_all()
                .map_err(|error| SupportStoreError::Write(error.to_string()))?;
        }

        fs::rename(&temp_path, &self.path)
            .map_err(|error| SupportStoreError::Write(error.to_string()))?;
        Ok(())
    }
}
