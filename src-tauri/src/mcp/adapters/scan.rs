//! Shared scan orchestration.
//!
//! Both layout adapters produce the same read-model shape (`McpHarnessScan`),
//! so the per-entry parsing and drift detection live here instead of being
//! duplicated.

use std::collections::HashMap;
use std::path::Path;

use serde_json::{json, Value};

use crate::mcp::contracts::{McpHarnessScan, McpObservedEntry};
use crate::mcp::mappers::TransportMapper;
use crate::mcp::store::{McpServerSpec, McpSource};

/// Turn raw server entries (name → payload map) into a harness scan, tagging
/// each entry as managed / drifted / unmanaged / missing against `specs`.
pub(crate) fn build_harness_scan(
    harness: &str,
    label: &str,
    logo_key: Option<&str>,
    config_path: &Path,
    installed: bool,
    config_present: bool,
    mcp_writable: bool,
    mcp_unavailable_reason: Option<String>,
    raw_entries: Result<Vec<(String, HashMap<String, Value>)>, String>,
    specs: &[McpServerSpec],
    mapper: &dyn TransportMapper,
) -> McpHarnessScan {
    let specs_by_name: HashMap<_, _> = specs.iter().map(|s| (s.name.clone(), s)).collect();
    let mut entries = Vec::new();
    let mut seen_names = std::collections::HashSet::new();
    let mut scan_issue = None;

    let raw_entries = match raw_entries {
        Ok(items) => items,
        Err(reason) => {
            scan_issue = Some(reason);
            vec![]
        }
    };

    for (name, payload) in raw_entries {
        seen_names.insert(name.clone());
        let mut parsed_spec = None;
        let mut parse_issue = None;
        match mapper.dict_to_spec(&name, &payload, Some(&McpSource::adopted(harness, &name))) {
            Ok(spec) => parsed_spec = Some(spec),
            Err(reason) => parse_issue = Some(reason),
        }

        let managed_spec = specs_by_name.get(&name);
        if managed_spec.is_none() {
            entries.push(McpObservedEntry {
                name,
                state: "unmanaged".into(),
                raw_payload: Some(json!(payload)),
                parsed_spec,
                drift_detail: None,
                parse_issue,
            });
            continue;
        }

        if let Some(reason) = parse_issue {
            entries.push(McpObservedEntry {
                name,
                state: "drifted".into(),
                raw_payload: Some(json!(payload)),
                parsed_spec,
                drift_detail: Some(reason.clone()),
                parse_issue: Some(reason),
            });
            continue;
        }

        let managed = managed_spec.unwrap();
        let expected = normalize_payload(&mapper.spec_to_dict(managed));
        let actual = normalize_payload(&payload);
        if expected == actual {
            entries.push(McpObservedEntry {
                name,
                state: "managed".into(),
                raw_payload: Some(json!(payload)),
                parsed_spec,
                drift_detail: None,
                parse_issue: None,
            });
        } else {
            entries.push(McpObservedEntry {
                name,
                state: "drifted".into(),
                raw_payload: Some(json!(payload)),
                parsed_spec,
                drift_detail: Some(drift_detail(&expected, &actual)),
                parse_issue: None,
            });
        }
    }

    for spec in specs {
        if !seen_names.contains(&spec.name) {
            entries.push(McpObservedEntry {
                name: spec.name.clone(),
                state: "missing".into(),
                raw_payload: None,
                parsed_spec: Some(spec.clone()),
                drift_detail: None,
                parse_issue: None,
            });
        }
    }

    McpHarnessScan {
        harness: harness.to_string(),
        label: label.to_string(),
        logo_key: logo_key.map(str::to_string),
        installed,
        config_present,
        config_path: config_path.to_path_buf(),
        mcp_writable,
        mcp_unavailable_reason,
        scan_issue,
        entries,
    }
}

fn normalize_payload(value: &HashMap<String, Value>) -> HashMap<String, Value> {
    let mut normalized = HashMap::new();
    for (key, item) in value {
        if is_semantic_default(key, item) {
            continue;
        }
        normalized.insert(key.clone(), normalize_value(item));
    }
    let mut keys: Vec<_> = normalized.keys().cloned().collect();
    keys.sort();
    keys.into_iter()
        .map(|k| (k.clone(), normalized[&k].clone()))
        .collect()
}

fn normalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if !is_semantic_default(k, v) {
                    out.insert(k.clone(), normalize_value(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_value).collect()),
        other => other.clone(),
    }
}

fn is_semantic_default(key: &str, value: &Value) -> bool {
    (key == "enabled" && value == &json!(true))
        || (key == "transport" && value == &json!("stdio"))
        || (matches!(key, "headers" | "env" | "environment" | "http_headers")
            && value.as_object().map(|m| m.is_empty()).unwrap_or(false))
}

fn drift_detail(expected: &HashMap<String, Value>, actual: &HashMap<String, Value>) -> String {
    let expected_keys: std::collections::HashSet<_> = expected.keys().collect();
    let actual_keys: std::collections::HashSet<_> = actual.keys().collect();
    let missing: Vec<_> = expected_keys
        .difference(&actual_keys)
        .map(|k| (*k).clone())
        .collect();
    let extra: Vec<_> = actual_keys
        .difference(&expected_keys)
        .map(|k| (*k).clone())
        .collect();
    let changed: Vec<_> = expected_keys
        .intersection(&actual_keys)
        .filter(|k| expected[k.as_str()] != actual[k.as_str()])
        .map(|k| (*k).clone())
        .collect();
    let mut parts = Vec::new();
    if !missing.is_empty() {
        parts.push(format!("missing={}", missing.join(",")));
    }
    if !extra.is_empty() {
        parts.push(format!("extra={}", extra.join(",")));
    }
    if !changed.is_empty() {
        parts.push(format!("changed={}", changed.join(",")));
    }
    if parts.is_empty() {
        "value mismatch".into()
    } else {
        parts.join("; ")
    }
}
