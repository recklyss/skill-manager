use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SlashCommandSyncRecord {
    pub target: String,
    pub path: PathBuf,
    pub content_hash: Option<String>,
    pub render_format: String,
}

pub type SlashCommandSyncState = HashMap<String, HashMap<String, SlashCommandSyncRecord>>;

#[derive(Clone)]
pub struct SlashCommandSyncStateStore {
    path: PathBuf,
}

impl SlashCommandSyncStateStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> SlashCommandSyncState {
        if !self.path.is_file() {
            return HashMap::new();
        }
        let Ok(text) = fs::read_to_string(&self.path) else {
            return HashMap::new();
        };
        let Ok(payload) = serde_json::from_str::<Value>(&text) else {
            return HashMap::new();
        };
        let commands = payload
            .get("commands")
            .or(Some(&payload))
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let mut state = HashMap::new();
        for (command_name, target_payload) in commands {
            let Some(target_map) = target_payload.as_object() else {
                continue;
            };
            let mut records = HashMap::new();
            for (target_id, raw_record) in target_map {
                if let Some(record) = parse_record(target_id, raw_record) {
                    records.insert(record.target.clone(), record);
                }
            }
            if !records.is_empty() {
                state.insert(command_name, records);
            }
        }
        state
    }

    pub fn write(&self, state: &SlashCommandSyncState) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut commands = serde_json::Map::new();
        let mut names: Vec<_> = state.keys().cloned().collect();
        names.sort();
        for name in names {
            let records = &state[&name];
            let mut target_map = serde_json::Map::new();
            let mut targets: Vec<_> = records.keys().cloned().collect();
            targets.sort();
            for target in targets {
                let record = &records[&target];
                target_map.insert(
                    target,
                    serde_json::json!({
                        "target": record.target,
                        "path": record.path.to_string_lossy(),
                        "contentHash": record.content_hash,
                        "renderFormat": record.render_format,
                    }),
                );
            }
            commands.insert(name, Value::Object(target_map));
        }
        let payload = serde_json::json!({
            "version": 2,
            "commands": Value::Object(commands),
        });
        let temp = self.path.with_extension("json.tmp");
        fs::write(&temp, serde_json::to_string_pretty(&payload).unwrap_or_default())
            .map_err(|e| e.to_string())?;
        fs::rename(&temp, &self.path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn replace_for(&self, name: &str, records: HashMap<String, SlashCommandSyncRecord>) -> Result<(), String> {
        let mut state = self.load();
        if records.is_empty() {
            state.remove(name);
        } else {
            state.insert(name.to_string(), records);
        }
        self.write(&state)
    }

    pub fn remove_command(&self, name: &str) -> HashMap<String, SlashCommandSyncRecord> {
        let mut state = self.load();
        let records = state.remove(name).unwrap_or_default();
        let _ = self.write(&state);
        records
    }

    pub fn add_target(&self, name: &str, record: SlashCommandSyncRecord) -> Result<(), String> {
        let mut state = self.load();
        let records = state.entry(name.to_string()).or_default();
        records.insert(record.target.clone(), record);
        self.write(&state)
    }

    pub fn remove_target(&self, name: &str, target: &str) -> Result<(), String> {
        let mut state = self.load();
        if let Some(records) = state.get_mut(name) {
            records.remove(target);
            if records.is_empty() {
                state.remove(name);
            }
        }
        self.write(&state)
    }
}

pub fn hash_file(path: &PathBuf) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let digest = Sha256::digest(bytes);
    Ok(format!("sha256:{digest:x}"))
}

fn parse_record(target_id: &str, raw_record: &Value) -> Option<SlashCommandSyncRecord> {
    let obj = raw_record.as_object()?;
    let path = obj.get("path")?.as_str()?;
    let render_format = obj.get("renderFormat")?.as_str()?;
    if render_format != "frontmatter_markdown" && render_format != "cursor_plaintext" {
        return None;
    }
    Some(SlashCommandSyncRecord {
        target: target_id.to_string(),
        path: PathBuf::from(path),
        content_hash: obj
            .get("contentHash")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        render_format: render_format.to_string(),
    })
}
