use std::collections::{BTreeSet, HashMap, HashSet};

use super::contracts::{McpBinding, McpHarnessScan, McpInventory, McpInventoryEntry, McpInventoryIssue};
use super::store::McpServerSpec;

pub fn build_inventory(
    managed_servers: &[McpServerSpec],
    specs: &[McpServerSpec],
    scans: &[McpHarnessScan],
    issues: Vec<McpInventoryIssue>,
) -> McpInventory {
    let mut bindings_by_name: HashMap<String, Vec<McpBinding>> = HashMap::new();
    for scan in scans {
        for entry in &scan.entries {
            bindings_by_name
                .entry(entry.name.clone())
                .or_default()
                .push(McpBinding {
                    harness: scan.harness.clone(),
                    state: entry.state.clone(),
                    drift_detail: entry.drift_detail.clone(),
                });
        }
    }

    let spec_by_name: HashMap<_, _> = specs.iter().map(|s| (s.name.clone(), s.clone())).collect();
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    let mut managed_sorted = managed_servers.to_vec();
    managed_sorted.sort_by_key(|s| s.display_name.to_lowercase());
    for server in managed_sorted {
        let spec = spec_by_name.get(&server.name).cloned();
        let bindings = bindings_by_name.remove(&server.name).unwrap_or_default();
        entries.push(McpInventoryEntry {
            name: server.name.clone(),
            display_name: server.display_name.clone(),
            spec,
            sightings: bindings,
            is_managed: true,
            can_enable: spec_by_name.contains_key(&server.name),
        });
        seen.insert(server.name.clone());
    }

    let mut unmanaged_names: BTreeSet<_> = bindings_by_name.keys().cloned().collect();
    unmanaged_names.retain(|name| !seen.contains(name));
    for name in unmanaged_names {
        entries.push(McpInventoryEntry {
            name: name.clone(),
            display_name: name.clone(),
            spec: spec_by_name.get(&name).cloned(),
            sightings: bindings_by_name.remove(&name).unwrap_or_default(),
            is_managed: false,
            can_enable: true,
        });
    }

    McpInventory {
        entries,
        issues,
    }
}
