use std::fs;
use std::path::PathBuf;

use super::codecs::{parse_slash_command_document, render_slash_command};
use super::path_policy::SlashCommandPathPolicy;
use super::planner::SyncEntry;
use super::queries::SlashCommandQueryService;
use super::store::{SlashCommandStore, validate_command_name};
use super::sync_state::{hash_file, SlashCommandSyncRecord, SlashCommandSyncStateStore};
use super::targets::SlashTarget;

#[derive(Clone)]
pub struct SlashCommandReviewResolver {
    store: SlashCommandStore,
    sync_state: SlashCommandSyncStateStore,
    queries: SlashCommandQueryService,
    path_policy: SlashCommandPathPolicy,
}

impl SlashCommandReviewResolver {
    pub fn new(
        store: SlashCommandStore,
        sync_state: SlashCommandSyncStateStore,
        queries: SlashCommandQueryService,
        path_policy: SlashCommandPathPolicy,
    ) -> Self {
        Self {
            store,
            sync_state,
            queries,
            path_policy,
        }
    }

    pub fn import_unmanaged_command(
        &self,
        target: &SlashTarget,
        name: &str,
    ) -> Result<serde_json::Value, String> {
        validate_command_name(name)?;
        if self.store.get_command(name).is_some() {
            return Err(format!(
                "slash command already exists: resolve {target_id}:{name} from review",
                target_id = target.id
            ));
        }
        let path = self.path_policy.output_path(target, name);
        if !path.is_file() {
            return Err(format!("slash command file not found: {}", path.display()));
        }

        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let command = parse_slash_command_document(name, &text, &target.render_format)?;
        self.store.create_command(&command)?;
        let record = SlashCommandSyncRecord {
            target: target.id.clone(),
            path: path.clone(),
            content_hash: hash_file(&path).ok(),
            render_format: target.render_format.clone(),
        };
        self.sync_state.add_target(name, record)?;
        Ok(serde_json::json!({
            "ok": true,
            "command": self.queries.get_command(name),
            "sync": [SyncEntry {
                target: target.id.clone(),
                path,
                status: "synced".into(),
                error: None,
            }.to_json()],
        }))
    }

    pub fn resolve_review_command(
        &self,
        target: &SlashTarget,
        name: &str,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        validate_command_name(name)?;
        match action {
            "restore_managed" => self.restore_managed(target, name),
            "adopt_target" => self.adopt_target(target, name),
            "remove_binding" => self.remove_binding(target, name),
            other => Err(format!("unknown slash command review action: {other}")),
        }
    }

    fn restore_managed(&self, target: &SlashTarget, name: &str) -> Result<serde_json::Value, String> {
        let command = self
            .store
            .get_command(name)
            .ok_or_else(|| format!("unknown slash command: {name}"))?;
        let previous = self
            .sync_state
            .load()
            .get(name)
            .and_then(|records| records.get(&target.id))
            .cloned()
            .ok_or_else(|| format!("slash command target is not managed: {}:{name}", target.id))?;
        let path = self.path_policy.tracked_path(target, &previous.path)?;
        let rendered = render_slash_command(&command, &target.render_format);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        crate::fsutil::atomic_write(&path, rendered.as_bytes())?;
        let record = SlashCommandSyncRecord {
            target: target.id.clone(),
            path: path.clone(),
            content_hash: hash_file(&path).ok(),
            render_format: target.render_format.clone(),
        };
        self.sync_state.add_target(name, record)?;
        Ok(self.mutation_payload(
            name,
            SyncEntry {
                target: target.id.clone(),
                path,
                status: "synced".into(),
                error: None,
            },
        ))
    }

    fn adopt_target(&self, target: &SlashTarget, name: &str) -> Result<serde_json::Value, String> {
        let path = self.review_file_path(target, name)?;
        if !path.is_file() {
            return Err(format!("slash command file not found: {}", path.display()));
        }
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let parsed = parse_slash_command_document(name, &text, &target.render_format)?;
        if self.store.get_command(name).is_none() {
            self.store.create_command(&parsed)?;
        } else {
            self.store
                .update_command(name, &parsed.description, &parsed.prompt)?;
        }
        let record = SlashCommandSyncRecord {
            target: target.id.clone(),
            path: path.clone(),
            content_hash: hash_file(&path).ok(),
            render_format: target.render_format.clone(),
        };
        self.sync_state.add_target(name, record)?;
        Ok(self.mutation_payload(
            name,
            SyncEntry {
                target: target.id.clone(),
                path,
                status: "synced".into(),
                error: None,
            },
        ))
    }

    fn remove_binding(&self, target: &SlashTarget, name: &str) -> Result<serde_json::Value, String> {
        let previous = self
            .sync_state
            .load()
            .get(name)
            .and_then(|records| records.get(&target.id))
            .cloned()
            .ok_or_else(|| format!("slash command target is not managed: {}:{name}", target.id))?;
        self.sync_state.remove_target(name, &target.id)?;
        Ok(self.mutation_payload(
            name,
            SyncEntry {
                target: target.id.clone(),
                path: previous.path,
                status: "removed".into(),
                error: None,
            },
        ))
    }

    fn review_file_path(&self, target: &SlashTarget, name: &str) -> Result<PathBuf, String> {
        if let Some(previous) = self
            .sync_state
            .load()
            .get(name)
            .and_then(|records| records.get(&target.id))
        {
            return self.path_policy.tracked_path(target, &previous.path);
        }
        Ok(self.path_policy.output_path(target, name))
    }

    fn mutation_payload(&self, name: &str, entry: SyncEntry) -> serde_json::Value {
        let ok = !matches!(
            entry.status.as_str(),
            "failed" | "blocked_manual_file" | "blocked_modified_file"
        );
        serde_json::json!({
            "ok": ok,
            "command": self.queries.get_command(name),
            "sync": [entry.to_json()],
        })
    }
}

