use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerSpec {
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    pub source: McpSource,
    #[serde(default = "default_transport")]
    pub transport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub installed_at: String,
    #[serde(default)]
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSource {
    pub kind: String,
    pub locator: String,
}

impl McpSource {
    pub fn marketplace(qualified_name: &str) -> Self {
        Self {
            kind: "marketplace".into(),
            locator: qualified_name.to_string(),
        }
    }

    pub fn adopted(harness: &str, name: &str) -> Self {
        Self {
            kind: "adopted".into(),
            locator: format!("{harness}:{name}"),
        }
    }

    pub fn manual(name: &str) -> Self {
        Self {
            kind: "manual".into(),
            locator: name.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct McpManifestIssue {
    pub name: String,
    pub reason: String,
}

fn default_transport() -> String {
    "stdio".to_string()
}

#[derive(Clone)]
pub struct McpServerStore {
    manifest_path: PathBuf,
}

impl McpServerStore {
    pub fn new(manifest_path: PathBuf) -> Self {
        Self { manifest_path }
    }

    pub fn list_records(&self) -> Vec<McpServerSpec> {
        self.load_manifest().0
    }

    pub fn get_record(&self, name: &str) -> Option<McpServerSpec> {
        self.list_records()
            .into_iter()
            .find(|spec| spec.name == name)
    }

    pub fn manifest_issues(&self) -> Vec<McpManifestIssue> {
        self.load_manifest().1
    }

    pub fn upsert(&self, spec: McpServerSpec) -> Result<McpServerSpec, String> {
        let stamped = prepare_managed_spec(spec);
        let (mut entries, issues) = self.load_manifest();
        if !issues.is_empty() && entries.is_empty() {
            return Err("manifest has issues".to_string());
        }
        if let Some(existing) = entries.iter_mut().find(|e| e.name == stamped.name) {
            *existing = stamped.clone();
        } else {
            entries.push(stamped.clone());
        }
        self.write_manifest(&entries)?;
        Ok(stamped)
    }

    pub fn remove(&self, name: &str) -> Result<bool, String> {
        let (entries, _) = self.load_manifest();
        let new_entries: Vec<_> = entries.into_iter().filter(|e| e.name != name).collect();
        if new_entries.len() == self.load_manifest().0.len() {
            return Ok(false);
        }
        self.write_manifest(&new_entries)?;
        Ok(true)
    }

    fn load_manifest(&self) -> (Vec<McpServerSpec>, Vec<McpManifestIssue>) {
        if !self.manifest_path.is_file() {
            return (vec![], vec![]);
        }
        let text = match fs::read_to_string(&self.manifest_path) {
            Ok(t) => t,
            Err(_) => return (vec![], vec![]),
        };
        let payload: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                return (
                    vec![],
                    vec![McpManifestIssue {
                        name: "<manifest>".into(),
                        reason: e.to_string(),
                    }],
                );
            }
        };
        let Some(servers) = payload.get("servers").and_then(|v| v.as_array()) else {
            return (
                vec![],
                vec![McpManifestIssue {
                    name: "<manifest>".into(),
                    reason: "'servers' must be a list".into(),
                }],
            );
        };
        let mut entries = Vec::new();
        let mut issues = Vec::new();
        for item in servers {
            match parse_record(item) {
                Ok(spec) => entries.push(spec),
                Err(reason) => issues.push(McpManifestIssue {
                    name: item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unknown>")
                        .to_string(),
                    reason,
                }),
            }
        }
        (entries, issues)
    }

    fn write_manifest(&self, entries: &[McpServerSpec]) -> Result<(), String> {
        if let Some(parent) = self.manifest_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let payload = json!({
            "version": 6,
            "servers": entries.iter().map(spec_to_record).collect::<Vec<_>>(),
        });
        let temp = self.manifest_path.with_extension("json.tmp");
        fs::write(&temp, serde_json::to_string_pretty(&payload).unwrap_or_default())
            .map_err(|e| e.to_string())?;
        fs::rename(&temp, &self.manifest_path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn parse_record(item: &Value) -> Result<McpServerSpec, String> {
    let mut spec: McpServerSpec = serde_json::from_value(item.clone())
        .map_err(|e| e.to_string())?;
    if spec.display_name.is_empty() {
        spec.display_name = spec.name.clone();
    }
    Ok(spec)
}

fn spec_to_record(spec: &McpServerSpec) -> Value {
    serde_json::to_value(spec).unwrap_or_else(|_| json!({}))
}

pub fn prepare_managed_spec(mut spec: McpServerSpec) -> McpServerSpec {
    if spec.installed_at.is_empty() {
        spec.installed_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    }
    spec.revision = compute_revision(&spec);
    spec
}

fn compute_revision(spec: &McpServerSpec) -> String {
    let payload = json!({
        "name": spec.name,
        "transport": spec.transport,
        "command": spec.command,
        "args": spec.args,
        "env": spec.env,
        "url": spec.url,
        "headers": spec.headers,
    });
    let digest = Sha256::digest(serde_json::to_string(&payload).unwrap_or_default().as_bytes());
    format!("{:x}", digest)[..16].to_string()
}
