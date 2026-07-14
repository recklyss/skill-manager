use super::codecs::parse_slash_command_document;
use super::path_policy::SlashCommandPathPolicy;
use super::store::{SlashCommand, SlashCommandStore};
use super::sync_state::{hash_file, SlashCommandSyncRecord, SlashCommandSyncStateStore};
use super::targets::SlashTarget;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct SlashCommandQueryService {
    store: SlashCommandStore,
    sync_state: SlashCommandSyncStateStore,
    store_path: PathBuf,
    sync_state_path: PathBuf,
    targets: Vec<SlashTarget>,
    path_policy: SlashCommandPathPolicy,
}

impl SlashCommandQueryService {
    pub fn new(
        store: SlashCommandStore,
        sync_state: SlashCommandSyncStateStore,
        store_path: PathBuf,
        sync_state_path: PathBuf,
        targets: Vec<SlashTarget>,
        path_policy: SlashCommandPathPolicy,
    ) -> Self {
        Self {
            store,
            sync_state,
            store_path,
            sync_state_path,
            targets,
            path_policy,
        }
    }

    pub fn list_commands(&self) -> Value {
        let commands = self.store.list_commands();
        let state = self.sync_state.load();
        json!({
            "storePath": self.store_path.to_string_lossy(),
            "syncStatePath": self.sync_state_path.to_string_lossy(),
            "targets": self.targets.iter().map(target_to_json).collect::<Vec<_>>(),
            "defaultTargets": super::targets::default_target_ids(&self.targets),
            "commands": commands.iter().map(|c| self.command_payload(c, state.get(&c.name))).collect::<Vec<_>>(),
            "reviewCommands": self.review_commands(&commands, &state),
        })
    }

    pub fn get_command(&self, name: &str) -> Option<Value> {
        let command = self.store.get_command(name)?;
        let state = self.sync_state.load();
        Some(self.command_payload(&command, state.get(name)))
    }

    fn command_payload(
        &self,
        command: &SlashCommand,
        records: Option<&HashMap<String, SlashCommandSyncRecord>>,
    ) -> Value {
        json!({
            "name": command.name,
            "description": command.description,
            "prompt": command.prompt,
            "syncTargets": self.sync_entries(&command.name, records),
        })
    }

    fn sync_entries(
        &self,
        command_name: &str,
        records: Option<&HashMap<String, SlashCommandSyncRecord>>,
    ) -> Vec<Value> {
        self.targets
            .iter()
            .map(|target| {
                let record = records.and_then(|m| m.get(&target.id));
                if let Some(record) = record {
                    match self.path_policy.tracked_path(target, &record.path) {
                        Ok(path) => {
                            if !path.exists() {
                                return json!({
                                    "target": target.id,
                                    "path": path.to_string_lossy(),
                                    "status": "missing",
                                    "error": "Managed slash command file is missing",
                                });
                            }
                            if let (Some(expected), Ok(actual)) =
                                (&record.content_hash, hash_file(&path))
                            {
                                if &actual != expected {
                                    return json!({
                                        "target": target.id,
                                        "path": path.to_string_lossy(),
                                        "status": "drifted",
                                        "error": "Managed slash command file changed outside Skill Manager",
                                    });
                                }
                            }
                            json!({
                                "target": target.id,
                                "path": path.to_string_lossy(),
                                "status": "synced",
                            })
                        }
                        Err(error) => json!({
                            "target": target.id,
                            "path": record.path.to_string_lossy(),
                            "status": "failed",
                            "error": error,
                        }),
                    }
                } else {
                    let path = self.path_policy.output_path(target, command_name);
                    json!({
                        "target": target.id,
                        "path": path.to_string_lossy(),
                        "status": "not_selected",
                    })
                }
            })
            .collect()
    }

    fn review_commands(
        &self,
        commands: &[SlashCommand],
        state: &super::sync_state::SlashCommandSyncState,
    ) -> Vec<Value> {
        let command_names: HashSet<_> = commands.iter().map(|c| c.name.as_str()).collect();
        let mut rows = Vec::new();
        rows.extend(self.tracked_review_rows(&command_names, state));
        rows.extend(self.unmanaged_review_rows(&command_names, state));
        rows
    }

    fn tracked_review_rows(
        &self,
        command_names: &HashSet<&str>,
        state: &super::sync_state::SlashCommandSyncState,
    ) -> Vec<Value> {
        let mut rows = Vec::new();
        for (command_name, records) in state {
            for record in records.values() {
                let Some(target) = self.target(&record.target) else {
                    continue;
                };
                match self.path_policy.tracked_path(target, &record.path) {
                    Ok(path) => {
                        if !path.exists() {
                            rows.push(review_row_json(
                                "missing",
                                target,
                                command_name,
                                &path,
                                "",
                                "",
                                command_names.contains(command_name.as_str()),
                                vec!["restore_managed", "remove_binding"],
                                Some("Managed slash command file is missing".into()),
                            ));
                            continue;
                        }
                        if let (Some(expected), Ok(actual)) =
                            (&record.content_hash, hash_file(&path))
                        {
                            if &actual != expected {
                                rows.push(self.parsed_review_row(
                                    "drifted",
                                    target,
                                    command_name,
                                    &path,
                                    command_names.contains(command_name.as_str()),
                                    vec!["restore_managed", "adopt_target", "remove_binding"],
                                    Some(
                                        "Managed slash command file changed outside Skill Manager"
                                            .into(),
                                    ),
                                ));
                            }
                        }
                    }
                    Err(error) => rows.push(review_row_json(
                        "drifted",
                        target,
                        command_name,
                        &record.path,
                        "",
                        "",
                        command_names.contains(command_name.as_str()),
                        vec!["remove_binding"],
                        Some(error),
                    )),
                }
            }
        }
        rows
    }

    fn unmanaged_review_rows(
        &self,
        command_names: &HashSet<&str>,
        state: &super::sync_state::SlashCommandSyncState,
    ) -> Vec<Value> {
        let known_paths: HashSet<_> = state
            .values()
            .flat_map(|records| records.values())
            .map(|record| self.path_policy.path_identity(&record.path))
            .collect();
        let mut rows = Vec::new();
        for target in &self.targets {
            if !target.output_dir.is_dir() {
                continue;
            }
            let Ok(entries) = fs::read_dir(&target.output_dir) else {
                continue;
            };
            let mut paths: Vec<PathBuf> = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| matches_glob(path, &target.file_glob))
                .collect();
            paths.sort();
            for path in paths {
                if known_paths.contains(&self.path_policy.path_identity(&path)) {
                    continue;
                }
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let command_exists = command_names.contains(name.as_str());
                let actions = if command_exists {
                    vec!["adopt_target"]
                } else {
                    vec!["import"]
                };
                rows.push(self.parsed_review_row(
                    "unmanaged",
                    target,
                    &name,
                    &path,
                    command_exists,
                    actions,
                    None,
                ));
            }
        }
        rows
    }

    fn parsed_review_row(
        &self,
        kind: &str,
        target: &SlashTarget,
        name: &str,
        path: &Path,
        command_exists: bool,
        actions: Vec<&str>,
        error: Option<String>,
    ) -> Value {
        match fs::read_to_string(path)
            .ok()
            .and_then(|text| parse_slash_command_document(name, &text, &target.render_format).ok())
        {
            Some(parsed) => review_row_json(
                kind,
                target,
                name,
                path,
                &parsed.description,
                &parsed.prompt,
                command_exists,
                actions,
                error,
            ),
            None => {
                let parse_error = fs::read_to_string(path)
                    .ok()
                    .and_then(|text| {
                        parse_slash_command_document(name, &text, &target.render_format).err()
                    })
                    .unwrap_or_else(|| "failed to read slash command file".into());
                review_row_json(
                    kind,
                    target,
                    name,
                    path,
                    "",
                    "",
                    command_exists,
                    Vec::new(),
                    Some(parse_error),
                )
            }
        }
    }

    fn target(&self, target_id: &str) -> Option<&SlashTarget> {
        self.targets.iter().find(|target| target.id == target_id)
    }
}

fn review_row_json(
    kind: &str,
    target: &SlashTarget,
    name: &str,
    path: &Path,
    description: &str,
    prompt: &str,
    command_exists: bool,
    actions: Vec<&str>,
    error: Option<String>,
) -> Value {
    let can_import = actions.contains(&"import") && error.is_none();
    json!({
        "reviewRef": format!("{kind}:{}:{name}", target.id),
        "kind": kind,
        "target": target.id,
        "targetLabel": target.label,
        "name": name,
        "path": path.to_string_lossy(),
        "description": description,
        "prompt": prompt,
        "commandExists": command_exists,
        "canImport": can_import,
        "actions": actions,
        "error": error,
    })
}

fn target_to_json(target: &SlashTarget) -> Value {
    json!({
        "id": target.id,
        "label": target.label,
        "rootPath": target.root_path.to_string_lossy(),
        "outputDir": target.output_dir.to_string_lossy(),
        "invocationPrefix": target.invocation_prefix,
        "renderFormat": target.render_format,
        "fileGlob": target.file_glob,
        "enabled": target.enabled,
        "available": target.available,
        "defaultSelected": target.default_selected,
    })
}

fn matches_glob(path: &Path, glob: &str) -> bool {
    match glob {
        "*.md" => path.extension().and_then(|ext| ext.to_str()) == Some("md"),
        _ => path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(glob.trim_start_matches('*'))),
    }
}
