use std::collections::HashMap;

use super::adapters::{FileBackedMcpAdapter, McpReadModelService};
use super::store::McpServerSpec;

#[derive(Clone)]
pub struct McpHarnessApplicationResult {
    pub succeeded: Vec<String>,
    pub failed: Vec<serde_json::Value>,
}

impl McpHarnessApplicationResult {
    pub fn ok(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "ok": self.ok(),
            "succeeded": self.succeeded,
            "failed": self.failed,
        })
    }
}

#[derive(Clone)]
pub struct McpHarnessApplication {
    read_models: McpReadModelService,
}

impl McpHarnessApplication {
    pub fn new(read_models: McpReadModelService) -> Self {
        Self { read_models }
    }

    pub fn enable_one(
        &self,
        adapter: &FileBackedMcpAdapter,
        spec: &McpServerSpec,
    ) -> McpHarnessApplicationResult {
        match adapter.enable_server(spec) {
            Ok(()) => {
                self.read_models.invalidate();
                McpHarnessApplicationResult {
                    succeeded: vec![adapter.harness.clone()],
                    failed: vec![],
                }
            }
            Err(error) => McpHarnessApplicationResult {
                succeeded: vec![],
                failed: vec![serde_json::json!({ "harness": adapter.harness, "error": error })],
            },
        }
    }

    pub fn enable_many(
        &self,
        spec: &McpServerSpec,
        harnesses: &[String],
        writable_only: bool,
        skip_harnesses: &[String],
    ) -> McpHarnessApplicationResult {
        let targets: std::collections::HashSet<_> = harnesses.iter().cloned().collect();
        let skipped: std::collections::HashSet<_> = skip_harnesses.iter().cloned().collect();
        let adapters: Vec<_> = if writable_only {
            self.read_models.enabled_writable_adapters()
        } else {
            self.read_models.enabled_adapters()
        };
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();
        for adapter in adapters {
            if !targets.contains(&adapter.harness) || skipped.contains(&adapter.harness) {
                continue;
            }
            match adapter.enable_server(spec) {
                Ok(()) => succeeded.push(adapter.harness.clone()),
                Err(error) => failed.push(serde_json::json!({ "harness": adapter.harness, "error": error })),
            }
        }
        if !succeeded.is_empty() {
            self.read_models.invalidate();
        }
        McpHarnessApplicationResult { succeeded, failed }
    }

    pub fn disable_many(
        &self,
        name: &str,
        harnesses: &[String],
        addressable_only: bool,
    ) -> McpHarnessApplicationResult {
        let targets: std::collections::HashSet<_> = harnesses.iter().cloned().collect();
        let adapters: Vec<_> = if addressable_only {
            self.read_models.enabled_addressable_adapters()
        } else {
            self.read_models.enabled_adapters()
        };
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();
        for adapter in adapters {
            if !targets.contains(&adapter.harness) {
                continue;
            }
            match adapter.disable_server(name) {
                Ok(()) => succeeded.push(adapter.harness.clone()),
                Err(error) => failed.push(serde_json::json!({ "harness": adapter.harness, "error": error })),
            }
        }
        if !succeeded.is_empty() {
            self.read_models.invalidate();
        }
        McpHarnessApplicationResult { succeeded, failed }
    }
}

pub fn harnesses_in_states(
    read_models: &McpReadModelService,
    name: &str,
    states: &[&str],
    addressable_only: bool,
) -> Vec<String> {
    let allowed: std::collections::HashSet<_> = states.iter().copied().collect();
    let addressable: std::collections::HashSet<_> = if addressable_only {
        read_models
            .enabled_addressable_adapters()
            .into_iter()
            .map(|a| a.harness.clone())
            .collect()
    } else {
        read_models.enabled_harnesses().into_iter().collect()
    };
    let snapshot = read_models.snapshot();
    let mut result = Vec::new();
    for scan in &snapshot.harness_scans {
        if !addressable.contains(&scan.harness) {
            continue;
        }
        for entry in &scan.entries {
            if entry.name == name && allowed.contains(entry.state.as_str()) {
                result.push(scan.harness.clone());
            }
        }
    }
    result
}
