use serde_json::{json, Value};

use super::availability::{availability_cache_key, McpAvailabilityResult};
use super::config_choice::config_choices_payload;
use super::contracts::{McpHarnessScan, McpInventory, McpInventoryEntry};
use super::redaction::{annotate_redacted_env, redacted_spec_dict};
use super::store::McpServerSpec;

pub fn inventory_payload(
    inventory: &McpInventory,
    scans: &[McpHarnessScan],
    availability_cache: &std::collections::HashMap<(String, String), McpAvailabilityResult>,
) -> Value {
    let visible_harnesses: std::collections::HashSet<_> = scans.iter().map(|s| s.harness.clone()).collect();
    json!({
        "columns": scans.iter().map(|scan| json!({
            "harness": scan.harness,
            "label": scan.label,
            "logoKey": scan.logo_key,
            "installed": scan.installed,
            "configPresent": scan.config_present,
            "mcpWritable": scan.mcp_writable,
            "mcpUnavailableReason": scan.mcp_unavailable_reason,
        })).collect::<Vec<_>>(),
        "entries": inventory.entries.iter().filter_map(|entry| {
            if entry.is_managed || entry.sightings.iter().any(|b| visible_harnesses.contains(&b.harness)) {
                Some(entry_payload(entry, scans, availability_cache))
            } else {
                None
            }
        }).collect::<Vec<_>>(),
        "issues": inventory.issues.iter().map(|issue| json!({
            "name": issue.name,
            "reason": issue.reason,
        })).collect::<Vec<_>>(),
    })
}

pub fn entry_payload(
    entry: &McpInventoryEntry,
    scans: &[McpHarnessScan],
    availability_cache: &std::collections::HashMap<(String, String), McpAvailabilityResult>,
) -> Value {
    let visible_harnesses: std::collections::HashSet<_> = scans.iter().map(|s| s.harness.clone()).collect();
    let addressable = addressable_harnesses(scans);
    let availability = entry
        .spec
        .as_ref()
        .map(|spec| availability_cache.get(&availability_cache_key(&entry.name, spec)).cloned())
        .flatten();
    let effective = entry_effective_availability(availability.as_ref());
    let config_status = install_config_status();
    json!({
        "name": entry.name,
        "displayName": entry.display_name,
        "kind": entry.kind(),
        "spec": entry.spec.as_ref().map(redacted_spec_dict),
        "canEnable": entry.can_enable,
        "enabledStatus": entry_enabled_status(entry, &addressable),
        "availabilityStatus": effective.status,
        "availabilityReason": effective.reason,
        "mcpStatus": mcp_status(availability.as_ref(), &config_status),
        "installConfigStatus": config_status,
        "sightings": entry.sightings.iter().filter_map(|binding| {
            if visible_harnesses.contains(&binding.harness) {
                Some(binding_to_json(binding))
            } else {
                None
            }
        }).collect::<Vec<_>>(),
    })
}

pub fn detail_extras_payload(name: &str, spec: &McpServerSpec, scans: &[McpHarnessScan]) -> Value {
    json!({
        "env": annotate_redacted_env(spec.env.as_ref()),
        "configChoices": config_choices_payload(name, spec, scans),
    })
}

fn binding_to_json(binding: &super::contracts::McpBinding) -> Value {
    let mut payload = json!({
        "harness": binding.harness,
        "state": binding.state,
    });
    if let Some(detail) = &binding.drift_detail {
        payload["driftDetail"] = json!(detail);
    }
    payload
}

fn addressable_harnesses(scans: &[McpHarnessScan]) -> std::collections::HashSet<String> {
    scans
        .iter()
        .filter(|scan| scan.mcp_writable && (scan.installed || scan.config_present))
        .map(|scan| scan.harness.clone())
        .collect()
}

fn entry_enabled_status(entry: &McpInventoryEntry, addressable: &std::collections::HashSet<String>) -> &'static str {
    for binding in &entry.sightings {
        if addressable.contains(&binding.harness) && binding.state == "managed" {
            return "enabled";
        }
    }
    "disabled"
}

fn entry_effective_availability(availability: Option<&McpAvailabilityResult>) -> McpAvailabilityResult {
    availability.cloned().unwrap_or(McpAvailabilityResult {
        status: "unavailable".into(),
        reason: None,
    })
}

fn install_config_status() -> Value {
    json!({
        "missingRequired": [],
        "provided": [],
        "status": "ready",
    })
}

fn mcp_status(availability: Option<&McpAvailabilityResult>, install_config_status: &Value) -> Value {
    if install_config_status
        .get("missingRequired")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
    {
        return json!({ "kind": "needs_config", "reason": Value::Null });
    }
    let Some(availability) = availability else {
        return json!({ "kind": "unchecked", "reason": Value::Null });
    };
    if availability.status == "available" {
        return json!({ "kind": "available", "reason": Value::Null });
    }
    if availability.reason.is_none() {
        return json!({ "kind": "unchecked", "reason": Value::Null });
    }
    json!({
        "kind": "connection_issue",
        "reason": availability.reason,
    })
}
