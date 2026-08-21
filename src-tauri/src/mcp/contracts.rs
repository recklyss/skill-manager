use std::path::PathBuf;

use super::store::McpServerSpec;

#[derive(Debug, Clone)]
pub struct McpObservedEntry {
    pub name: String,
    pub state: String,
    pub raw_payload: Option<serde_json::Value>,
    pub parsed_spec: Option<McpServerSpec>,
    pub drift_detail: Option<String>,
    pub parse_issue: Option<String>,
}

#[derive(Debug, Clone)]
pub struct McpHarnessScan {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub installed: bool,
    pub config_present: bool,
    pub config_path: PathBuf,
    pub mcp_writable: bool,
    pub mcp_unavailable_reason: Option<String>,
    pub scan_issue: Option<String>,
    pub entries: Vec<McpObservedEntry>,
}

#[derive(Debug, Clone)]
pub struct McpBinding {
    pub harness: String,
    pub state: String,
    pub drift_detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct McpInventoryEntry {
    pub name: String,
    pub display_name: String,
    pub spec: Option<McpServerSpec>,
    pub sightings: Vec<McpBinding>,
    pub is_managed: bool,
    pub can_enable: bool,
}

impl McpInventoryEntry {
    pub fn kind(&self) -> &'static str {
        if self.is_managed {
            "managed"
        } else {
            "unmanaged"
        }
    }
}

#[derive(Debug, Clone)]
pub struct McpInventoryIssue {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct McpInventory {
    pub entries: Vec<McpInventoryEntry>,
    pub issues: Vec<McpInventoryIssue>,
}

#[derive(Debug, Clone)]
pub struct McpReadModelSnapshot {
    pub harness_scans: Vec<McpHarnessScan>,
}
