use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::path_policy::SlashCommandPathPolicy;
use super::store::SlashCommand;
use super::sync_state::{hash_file, SlashCommandSyncRecord};
use super::targets::SlashTarget;

#[derive(Debug, Clone)]
pub struct PlannedWrite {
    pub target: SlashTarget,
    pub previous: Option<SlashCommandSyncRecord>,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PlannedRemove {
    pub target: SlashTarget,
    pub record: SlashCommandSyncRecord,
}

#[derive(Debug, Clone)]
pub struct SlashSyncPlan {
    pub writes: Vec<PlannedWrite>,
    pub removes: Vec<PlannedRemove>,
    pub blocked: Vec<SyncEntry>,
    pub keep: HashMap<String, SlashCommandSyncRecord>,
}

#[derive(Debug, Clone)]
pub struct SyncEntry {
    pub target: String,
    pub path: PathBuf,
    pub status: String,
    pub error: Option<String>,
}

impl SyncEntry {
    pub fn to_json(&self) -> serde_json::Value {
        let mut payload = serde_json::json!({
            "target": self.target,
            "path": self.path.to_string_lossy(),
            "status": self.status,
        });
        if let Some(error) = &self.error {
            payload["error"] = serde_json::Value::String(error.clone());
        }
        payload
    }
}

#[derive(Clone)]
pub struct SlashCommandPlanner {
    path_policy: SlashCommandPathPolicy,
}

impl SlashCommandPlanner {
    pub fn new(path_policy: SlashCommandPathPolicy) -> Self {
        Self { path_policy }
    }

    pub fn plan_sync(
        &self,
        command: &SlashCommand,
        selected: &[SlashTarget],
        previous: &HashMap<String, SlashCommandSyncRecord>,
        all_targets: &[SlashTarget],
    ) -> SlashSyncPlan {
        let selected_ids: HashSet<_> = selected.iter().map(|target| target.id.as_str()).collect();
        let mut writes = Vec::new();
        let mut removes = Vec::new();
        let mut blocked = Vec::new();
        let mut keep = HashMap::new();

        for target in selected {
            let record = previous.get(&target.id);
            let path = match record {
                Some(record) => match self.path_policy.tracked_path(target, &record.path) {
                    Ok(path) => path,
                    Err(error) => {
                        blocked.push(SyncEntry {
                            target: target.id.clone(),
                            path: record.path.clone(),
                            status: "failed".into(),
                            error: Some(error),
                        });
                        keep.insert(target.id.clone(), record.clone());
                        continue;
                    }
                },
                None => self.path_policy.output_path(target, &command.name),
            };

            if let Some(block) = self.write_block(target, &path, record) {
                blocked.push(block);
                if let Some(record) = record {
                    keep.insert(target.id.clone(), record.clone());
                }
                continue;
            }
            writes.push(PlannedWrite {
                target: target.clone(),
                previous: record.cloned(),
                path,
            });
        }

        for (target_id, record) in previous {
            if selected_ids.contains(target_id.as_str()) {
                continue;
            }
            let Some(target) = all_targets.iter().find(|candidate| candidate.id == *target_id) else {
                keep.insert(record.target.clone(), record.clone());
                continue;
            };
            if let Some(block) = self.remove_block(target, record) {
                blocked.push(block);
                keep.insert(record.target.clone(), record.clone());
                continue;
            }
            removes.push(PlannedRemove {
                target: target.clone(),
                record: record.clone(),
            });
        }

        SlashSyncPlan {
            writes,
            removes,
            blocked,
            keep,
        }
    }

    pub fn plan_delete(
        &self,
        records: &HashMap<String, SlashCommandSyncRecord>,
        all_targets: &[SlashTarget],
    ) -> SlashSyncPlan {
        let mut removes = Vec::new();
        let mut blocked = Vec::new();
        let mut keep = HashMap::new();

        for (target_id, record) in records {
            let Some(target) = all_targets.iter().find(|candidate| candidate.id == *target_id) else {
                keep.insert(record.target.clone(), record.clone());
                continue;
            };
            if let Some(block) = self.remove_block(target, record) {
                blocked.push(block);
                keep.insert(record.target.clone(), record.clone());
            } else {
                removes.push(PlannedRemove {
                    target: target.clone(),
                    record: record.clone(),
                });
            }
        }

        SlashSyncPlan {
            writes: Vec::new(),
            removes,
            blocked,
            keep,
        }
    }

    fn write_block(
        &self,
        target: &SlashTarget,
        path: &PathBuf,
        record: Option<&SlashCommandSyncRecord>,
    ) -> Option<SyncEntry> {
        if record.is_none() {
            if path.exists() {
                return Some(SyncEntry {
                    target: target.id.clone(),
                    path: path.clone(),
                    status: "blocked_manual_file".into(),
                    error: Some(format!("refusing to overwrite manual file: {}", path.display())),
                });
            }
            return None;
        }

        let record = record?;
        let record_path = match self.path_policy.tracked_path(target, &record.path) {
            Ok(path) => path,
            Err(error) => {
                return Some(SyncEntry {
                    target: target.id.clone(),
                    path: record.path.clone(),
                    status: "failed".into(),
                    error: Some(error),
                });
            }
        };
        if record_path.exists() {
            if let (Some(expected), Ok(actual)) = (&record.content_hash, hash_file(&record_path)) {
                if &actual != expected {
                    let display = record_path.display().to_string();
                    return Some(SyncEntry {
                        target: target.id.clone(),
                        path: record_path,
                        status: "blocked_modified_file".into(),
                        error: Some(format!(
                            "refusing to overwrite modified managed file: {display}"
                        )),
                    });
                }
            }
        }
        None
    }

    fn remove_block(
        &self,
        target: &SlashTarget,
        record: &SlashCommandSyncRecord,
    ) -> Option<SyncEntry> {
        let record_path = match self.path_policy.tracked_path(target, &record.path) {
            Ok(path) => path,
            Err(error) => {
                return Some(SyncEntry {
                    target: target.id.clone(),
                    path: record.path.clone(),
                    status: "failed".into(),
                    error: Some(error),
                });
            }
        };
        if record_path.exists() {
            if let (Some(expected), Ok(actual)) = (&record.content_hash, hash_file(&record_path)) {
                if &actual != expected {
                    let display = record_path.display().to_string();
                    return Some(SyncEntry {
                        target: target.id.clone(),
                        path: record_path,
                        status: "blocked_modified_file".into(),
                        error: Some(format!(
                            "refusing to delete modified managed file: {display}"
                        )),
                    });
                }
            }
        }
        None
    }
}
