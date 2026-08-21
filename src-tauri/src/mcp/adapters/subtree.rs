//! Subtree-layout MCP adapter.
//!
//! Covers the majority of harnesses (Codex, Claude, Cursor, OpenCode, Hermes,
//! OpenClaw, Copilot, Pi) where MCP servers live as a map under a config key
//! such as `mcpServers` / `mcp_servers` / `mcp.servers`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{json, Value};

use crate::harness::{
    is_executable_on_path, resolve_executable_path, ConfigSubtreeBindingProfile,
    HarnessDefinition, ResolutionContext,
};
use crate::mcp::adapters::{build_harness_scan, load_document, write_document, McpAdapter};
use crate::mcp::contracts::McpHarnessScan;
use crate::mcp::mappers::{get_mapper, value_to_payload_map, TransportMapper};
use crate::mcp::store::McpServerSpec;

#[derive(Clone)]
pub struct FileBackedMcpAdapter {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub config_path: PathBuf,
    definition: &'static HarnessDefinition,
    profile: ConfigSubtreeBindingProfile,
    context: ResolutionContext,
    mapper: &'static dyn TransportMapper,
}

impl FileBackedMcpAdapter {
    pub fn new(
        definition: &'static HarnessDefinition,
        profile: ConfigSubtreeBindingProfile,
        context: ResolutionContext,
    ) -> Self {
        let mapper = get_mapper(profile.codec);
        Self {
            harness: definition.harness.to_string(),
            label: definition.label.to_string(),
            logo_key: definition.logo_key.map(str::to_string),
            config_path: profile.resolve_config_path(&context),
            definition,
            profile,
            context,
            mapper,
        }
    }

    pub fn status(&self) -> (bool, bool, bool, Option<String>) {
        let installed = is_executable_on_path(&self.context, self.definition.install_probe);
        let config_present = self.config_path.is_file();
        let (mcp_writable, unavailable_reason) = self.mcp_write_capability(installed);
        (installed, config_present, mcp_writable, unavailable_reason)
    }

    pub fn scan(&self, specs: &[McpServerSpec]) -> McpHarnessScan {
        let (installed, config_present, mcp_writable, mcp_unavailable_reason) = self.status();
        let raw_entries = if config_present {
            self.read_entries()
        } else {
            Ok(vec![])
        };
        build_harness_scan(
            &self.harness,
            &self.label,
            self.logo_key.as_deref(),
            &self.config_path,
            installed,
            config_present,
            mcp_writable,
            mcp_unavailable_reason,
            raw_entries,
            specs,
            self.mapper,
        )
    }

    pub fn has_binding(&self, name: &str) -> bool {
        self.read_entries()
            .map(|entries| entries.iter().any(|(n, _)| n == name))
            .unwrap_or(false)
    }

    pub fn enable_server(&self, spec: &McpServerSpec) -> Result<(), String> {
        self.require_mcp_writable()?;
        let mut document = self.load_document()?;
        let payload = self.mapper.spec_to_dict(spec);
        set_subtree_entry(
            &mut document,
            self.profile.subtree_path,
            &spec.name,
            payload,
        )?;
        self.write_document(&document)
    }

    pub fn disable_server(&self, name: &str) -> Result<(), String> {
        if !self.config_path.is_file() {
            return Ok(());
        }
        let mut document = self.load_document()?;
        if remove_subtree_entry(&mut document, self.profile.subtree_path, name) {
            self.write_document(&document)?;
        }
        Ok(())
    }

    fn load_document(&self) -> Result<Value, String> {
        load_document(
            &self.config_path,
            self.profile.file_format,
            &self.harness,
            json!({}),
        )
    }

    fn write_document(&self, document: &Value) -> Result<(), String> {
        write_document(&self.config_path, self.profile.file_format, document)
    }

    fn require_mcp_writable(&self) -> Result<(), String> {
        let (_, _, writable, reason) = self.status();
        if writable {
            return Ok(());
        }
        Err(reason.unwrap_or_else(|| format!("{} MCP config is not writable", self.label)))
    }

    fn mcp_write_capability(&self, installed: bool) -> (bool, Option<String>) {
        let Some(probe) = self.profile.capability_probe else {
            return (true, None);
        };
        let reason = self
            .profile
            .capability_unavailable_reason
            .map(str::to_string)
            .unwrap_or_else(|| format!("{} MCP support is unavailable", self.label));
        if probe == "openclaw-mcp-command" {
            let executable = resolve_executable_path(&self.context, self.definition.install_probe);
            let Some(executable) = executable else {
                return (false, Some(reason));
            };
            let output = Command::new(executable).args(["mcp", "--help"]).output();
            match output {
                Ok(result) if result.status.success() => (true, None),
                _ => (false, Some(reason)),
            }
        } else {
            (installed, if installed { None } else { Some(reason) })
        }
    }

    fn read_entries(&self) -> Result<Vec<(String, HashMap<String, Value>)>, String> {
        if !self.config_path.is_file() {
            return Ok(vec![]);
        }
        let document = self.load_document()?;
        let subtree = read_subtree(&document, self.profile.subtree_path)?;
        Ok(subtree
            .into_iter()
            .filter_map(|(name, value)| {
                value_to_payload_map(&value).map(|payload| (name, payload))
            })
            .collect())
    }
}

impl McpAdapter for FileBackedMcpAdapter {
    fn harness(&self) -> &str {
        &self.harness
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn status(&self) -> (bool, bool, bool, Option<String>) {
        self.status()
    }

    fn scan(&self, specs: &[McpServerSpec]) -> McpHarnessScan {
        self.scan(specs)
    }

    fn has_binding(&self, name: &str) -> bool {
        self.has_binding(name)
    }

    fn enable_server(&self, spec: &McpServerSpec) -> Result<(), String> {
        self.enable_server(spec)
    }

    fn disable_server(&self, name: &str) -> Result<(), String> {
        self.disable_server(name)
    }
}

fn read_subtree(document: &Value, subtree_path: &[&str]) -> Result<HashMap<String, Value>, String> {
    let mut cursor = document;
    for key in subtree_path {
        cursor = cursor.get(*key).ok_or_else(|| format!("missing subtree '{key}'"))?;
    }
    let obj = cursor
        .as_object()
        .ok_or_else(|| "subtree must be an object".to_string())?;
    Ok(obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

fn set_subtree_entry(
    document: &mut Value,
    subtree_path: &[&str],
    name: &str,
    payload: HashMap<String, Value>,
) -> Result<(), String> {
    ensure_subtree(document, subtree_path);
    let mut cursor = document;
    for key in subtree_path {
        cursor = cursor.get_mut(*key).unwrap();
    }
    let obj = cursor.as_object_mut().unwrap();
    obj.insert(name.to_string(), json!(payload));
    Ok(())
}

fn remove_subtree_entry(document: &mut Value, subtree_path: &[&str], name: &str) -> bool {
    let mut cursor = &mut *document;
    for key in subtree_path {
        let Some(next) = cursor.get_mut(*key) else {
            return false;
        };
        cursor = next;
    }
    let Some(obj) = cursor.as_object_mut() else {
        return false;
    };
    let removed = obj.remove(name).is_some();
    if obj.is_empty() && subtree_path.len() > 1 {
        let mut parent = &mut *document;
        for key in &subtree_path[..subtree_path.len() - 1] {
            parent = parent.get_mut(*key).unwrap();
        }
        parent
            .as_object_mut()
            .unwrap()
            .remove(*subtree_path.last().unwrap());
    }
    removed
}

fn ensure_subtree(document: &mut Value, subtree_path: &[&str]) {
    let mut cursor = document;
    if !cursor.is_object() {
        *cursor = json!({});
    }
    for key in subtree_path {
        if !cursor.get(key).map(|v| v.is_object()).unwrap_or(false) {
            cursor.as_object_mut().unwrap().insert(key.to_string(), json!({}));
        }
        cursor = cursor.get_mut(*key).unwrap();
    }
}
