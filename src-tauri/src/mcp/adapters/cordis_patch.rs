//! Cordis patch-list MCP adapter.
//!
//! DeepSeek Harness stores MCP servers as `dsh-mcp-client` plugin rows in a
//! `cordis.patch.yml` (a YAML *list* of rows), keyed by `config.serverName`.
//! This adapter reads/writes that list shape.

use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::harness::{
    dsh_home, is_executable_on_path, ConfigFileFormat, CordisPatchBindingProfile,
    HarnessDefinition, ResolutionContext,
};
use crate::mcp::adapters::{build_harness_scan, load_document, write_document, McpAdapter};
use crate::mcp::contracts::McpHarnessScan;
use crate::mcp::mappers::{get_mapper, TransportMapper};
use crate::mcp::store::McpServerSpec;

pub struct CordisPatchMcpAdapter {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub config_path: PathBuf,
    definition: &'static HarnessDefinition,
    profile: CordisPatchBindingProfile,
    context: ResolutionContext,
    mapper: &'static dyn TransportMapper,
}

impl CordisPatchMcpAdapter {
    pub fn new(
        definition: &'static HarnessDefinition,
        profile: CordisPatchBindingProfile,
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

    fn installed(&self) -> bool {
        if is_executable_on_path(&self.context, self.definition.install_probe) {
            return true;
        }
        dsh_home(&self.context).exists()
    }

    pub fn status(&self) -> (bool, bool, bool, Option<String>) {
        let installed = self.installed();
        let config_present = self.config_path.is_file();
        (installed, config_present, true, None)
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
        let mut document = self.load_document()?;
        let rows = document
            .as_array_mut()
            .ok_or_else(|| "deepseek cordis config must be a YAML list".to_string())?;
        rows.retain(|row| !self.is_server_row(row, &spec.name));
        let payload = self.mapper.spec_to_dict(spec);
        rows.push(json!({
            "id": format!("mcp-{}", spec.name),
            "name": self.profile.package_name,
            "config": payload,
        }));
        self.write_document(&document)
    }

    pub fn disable_server(&self, name: &str) -> Result<(), String> {
        if !self.config_path.is_file() {
            return Ok(());
        }
        let mut document = self.load_document()?;
        let rows = document
            .as_array_mut()
            .ok_or_else(|| "deepseek cordis config must be a YAML list".to_string())?;
        let before = rows.len();
        rows.retain(|row| !self.is_server_row(row, name));
        if rows.len() != before {
            self.write_document(&document)?;
        }
        Ok(())
    }

    fn is_server_row(&self, row: &Value, server_name: &str) -> bool {
        row.get("name").and_then(|v| v.as_str()) == Some(self.profile.package_name)
            && row
                .get("config")
                .and_then(|c| c.get("serverName"))
                .and_then(|v| v.as_str())
                == Some(server_name)
    }

    fn read_entries(&self) -> Result<Vec<(String, HashMap<String, Value>)>, String> {
        if !self.config_path.is_file() {
            return Ok(vec![]);
        }
        let document = self.load_document()?;
        let rows = document
            .as_array()
            .ok_or_else(|| "deepseek cordis config must be a YAML list".to_string())?;
        let mut entries = Vec::new();
        for row in rows {
            let Some(name) = row.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            if name != self.profile.package_name {
                continue;
            }
            let Some(config) = row.get("config").and_then(|v| v.as_object()) else {
                continue;
            };
            let Some(server_name) = config.get("serverName").and_then(|v| v.as_str()) else {
                continue;
            };
            let payload: HashMap<String, Value> =
                config.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            entries.push((server_name.to_string(), payload));
        }
        Ok(entries)
    }

    fn load_document(&self) -> Result<Value, String> {
        load_document(
            &self.config_path,
            ConfigFileFormat::Yaml,
            &self.harness,
            json!([]),
        )
    }

    fn write_document(&self, document: &Value) -> Result<(), String> {
        write_document(&self.config_path, ConfigFileFormat::Yaml, document)
    }
}

impl McpAdapter for CordisPatchMcpAdapter {
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
