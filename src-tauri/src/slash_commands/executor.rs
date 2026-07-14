use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::codecs::render_slash_command;
use super::path_policy::SlashCommandPathPolicy;
use super::planner::{SlashCommandPlanner, SyncEntry};
use super::store::SlashCommand;
use super::sync_state::{hash_file, SlashCommandSyncRecord, SlashCommandSyncStateStore};
use super::targets::SlashTarget;

#[derive(Clone)]
pub struct SlashCommandSyncExecutor {
    sync_state: SlashCommandSyncStateStore,
    planner: SlashCommandPlanner,
    path_policy: SlashCommandPathPolicy,
}

impl SlashCommandSyncExecutor {
    pub fn new(
        sync_state: SlashCommandSyncStateStore,
        planner: SlashCommandPlanner,
        path_policy: SlashCommandPathPolicy,
    ) -> Self {
        Self {
            sync_state,
            planner,
            path_policy,
        }
    }

    pub fn sync_command(
        &self,
        command: &SlashCommand,
        selected: &[SlashTarget],
        all_targets: &[SlashTarget],
    ) -> Result<serde_json::Value, String> {
        let previous = self
            .sync_state
            .load()
            .get(&command.name)
            .cloned()
            .unwrap_or_default();
        let plan = self
            .planner
            .plan_sync(command, selected, &previous, all_targets);

        let mut results: Vec<SyncEntry> = plan.blocked;
        let mut next_records: HashMap<String, SlashCommandSyncRecord> = plan.keep;

        for write in plan.writes {
            match write_target(&write.path, command, &write.target) {
                Ok(()) => {
                    let content_hash = hash_file(&write.path).ok();
                    next_records.insert(
                        write.target.id.clone(),
                        SlashCommandSyncRecord {
                            target: write.target.id.clone(),
                            path: write.path.clone(),
                            content_hash,
                            render_format: write.target.render_format.clone(),
                        },
                    );
                    results.push(SyncEntry {
                        target: write.target.id.clone(),
                        path: write.path.clone(),
                        status: "synced".into(),
                        error: None,
                    });
                }
                Err(error) => {
                    if let Some(previous) = write.previous {
                        next_records.insert(write.target.id.clone(), previous);
                    }
                    results.push(SyncEntry {
                        target: write.target.id.clone(),
                        path: write.path,
                        status: "failed".into(),
                        error: Some(error),
                    });
                }
            }
        }

        for remove in plan.removes {
            match remove_target_file(&self.path_policy, &remove.target, &remove.record) {
                Ok(path) => results.push(SyncEntry {
                    target: remove.target.id.clone(),
                    path,
                    status: "removed".into(),
                    error: None,
                }),
                Err(error) => {
                    let path = remove.record.path.clone();
                    next_records.insert(remove.target.id.clone(), remove.record);
                    results.push(SyncEntry {
                        target: remove.target.id.clone(),
                        path,
                        status: "failed".into(),
                        error: Some(error),
                    });
                }
            }
        }

        self.sync_state
            .replace_for(&command.name, next_records)?;
        let ok = results.iter().all(|entry| {
            matches!(
                entry.status.as_str(),
                "synced" | "removed" | "not_selected"
            )
        });
        Ok(serde_json::json!({
            "ok": ok,
            "sync": results.iter().map(SyncEntry::to_json).collect::<Vec<_>>(),
        }))
    }

    pub fn remove_tracked_outputs(
        &self,
        records: &HashMap<String, SlashCommandSyncRecord>,
        all_targets: &[SlashTarget],
    ) -> Result<serde_json::Value, String> {
        let plan = self.planner.plan_delete(records, all_targets);
        if !plan.blocked.is_empty() {
            return Ok(serde_json::json!({
                "ok": false,
                "sync": plan.blocked.iter().map(SyncEntry::to_json).collect::<Vec<_>>(),
            }));
        }

        let mut results = Vec::new();
        for remove in plan.removes {
            match remove_target_file(&self.path_policy, &remove.target, &remove.record) {
                Ok(path) => results.push(SyncEntry {
                    target: remove.target.id.clone(),
                    path,
                    status: "removed".into(),
                    error: None,
                }),
                Err(error) => results.push(SyncEntry {
                    target: remove.target.id.clone(),
                    path: remove.record.path,
                    status: "failed".into(),
                    error: Some(error),
                }),
            }
        }

        let ok = !results.iter().any(|entry| entry.status == "failed");
        Ok(serde_json::json!({
            "ok": ok,
            "sync": results.iter().map(SyncEntry::to_json).collect::<Vec<_>>(),
        }))
    }
}

fn write_target(path: &PathBuf, command: &SlashCommand, target: &SlashTarget) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let rendered = render_slash_command(command, &target.render_format);
    let temp = path.with_extension("md.tmp");
    fs::write(&temp, rendered).map_err(|e| e.to_string())?;
    fs::rename(&temp, path).map_err(|e| e.to_string())?;
    Ok(())
}

fn remove_target_file(
    path_policy: &SlashCommandPathPolicy,
    target: &SlashTarget,
    record: &SlashCommandSyncRecord,
) -> Result<PathBuf, String> {
    let path = path_policy.tracked_path(target, &record.path)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(path)
}
