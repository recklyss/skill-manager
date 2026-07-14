use super::contracts::McpHarnessScan;
use super::env::is_env_var_reference;
use super::identity::HarnessSighting;
use super::redaction::{annotate_redacted_env, redact_payload, redacted_spec_dict};
use super::store::McpServerSpec;

#[derive(Clone)]
pub struct McpConfigChoice {
    pub id: String,
    pub source_kind: String,
    pub observed_harness: Option<String>,
    pub label: String,
    pub logo_key: Option<String>,
    pub config_path: Option<String>,
    pub payload_preview: serde_json::Value,
    pub spec: McpServerSpec,
    pub env: Vec<serde_json::Value>,
    pub recommended: bool,
}

impl McpConfigChoice {
    pub fn to_json(&self) -> serde_json::Value {
        let mut payload = serde_json::json!({
            "id": self.id,
            "sourceKind": self.source_kind,
            "observedHarness": self.observed_harness,
            "label": self.label,
            "logoKey": self.logo_key,
            "configPath": self.config_path,
            "payloadPreview": self.payload_preview,
            "spec": redacted_spec_dict(&self.spec),
            "env": self.env,
            "recommended": self.recommended,
        });
        if self.recommended {
            payload["recommended"] = serde_json::json!(true);
        }
        payload
    }
}

pub fn config_choices_payload(
    name: &str,
    managed_spec: &McpServerSpec,
    scans: &[McpHarnessScan],
) -> Vec<serde_json::Value> {
    let choices = config_choices(name, managed_spec, scans);
    let recommended_id = recommended_choice_id(&choices);
    choices
        .into_iter()
        .map(|choice| {
            let mut item = choice;
            item.recommended = recommended_id.as_deref() == Some(item.id.as_str());
            item.to_json()
        })
        .collect()
}

pub fn observed_spec_from_scans(
    name: &str,
    observed_harness: &str,
    scans: &[McpHarnessScan],
) -> Result<McpServerSpec, String> {
    for scan in scans {
        if scan.harness != observed_harness {
            continue;
        }
        for entry in &scan.entries {
            if entry.name != name {
                continue;
            }
            return entry.parsed_spec.clone().ok_or_else(|| {
                entry
                    .parse_issue
                    .clone()
                    .unwrap_or_else(|| format!("unable to parse '{name}' in {observed_harness}"))
            });
        }
    }
    Err(format!(
        "server '{name}' was not observed in harness '{observed_harness}'"
    ))
}

pub fn recommended_observed_harness(sightings: &[HarnessSighting]) -> Option<String> {
    if sightings.is_empty() {
        return None;
    }
    for sighting in sightings {
        if sighting.spec.transport == "stdio" && spec_has_env_ref(&sighting.spec) {
            return Some(sighting.harness.clone());
        }
    }
    for sighting in sightings {
        if sighting.spec.transport == "stdio" {
            return Some(sighting.harness.clone());
        }
    }
    for sighting in sightings {
        if sighting.spec.transport != "stdio" && !url_has_embedded_credential(sighting.spec.url.as_deref()) {
            return Some(sighting.harness.clone());
        }
    }
    sightings.first().map(|s| s.harness.clone())
}

fn config_choices(
    name: &str,
    managed_spec: &McpServerSpec,
    scans: &[McpHarnessScan],
) -> Vec<McpConfigChoice> {
    let mut choices = vec![McpConfigChoice {
        id: "managed".into(),
        source_kind: "managed".into(),
        observed_harness: None,
        label: "Managed config".into(),
        logo_key: None,
        config_path: None,
        payload_preview: redacted_spec_dict(managed_spec),
        spec: managed_spec.clone(),
        env: annotate_redacted_env(managed_spec.env.as_ref()),
        recommended: false,
    }];
    for scan in scans {
        for observed in &scan.entries {
            if observed.name != name || observed.state != "drifted" {
                continue;
            }
            let Some(parsed) = observed.parsed_spec.clone() else {
                continue;
            };
            let preview = observed
                .raw_payload
                .as_ref()
                .map(|v| redact_payload(v, ""))
                .unwrap_or(serde_json::Value::Null);
            choices.push(McpConfigChoice {
                id: format!("harness:{}", scan.harness),
                source_kind: "harness".into(),
                observed_harness: Some(scan.harness.clone()),
                label: format!("{} config", scan.label),
                logo_key: scan.logo_key.clone(),
                config_path: if scan.config_present {
                    Some(scan.config_path.to_string_lossy().into_owned())
                } else {
                    None
                },
                payload_preview: preview,
                spec: parsed,
                env: annotate_redacted_env(observed.parsed_spec.as_ref().and_then(|s| s.env.as_ref())),
                recommended: false,
            });
        }
    }
    choices
}

fn recommended_choice_id(choices: &[McpConfigChoice]) -> Option<String> {
    let harness_choices: Vec<_> = choices
        .iter()
        .filter(|c| c.source_kind == "harness")
        .collect();
    if harness_choices.is_empty() {
        return choices.first().map(|c| c.id.clone());
    }
    let has_env_ref = |choice: &&McpConfigChoice| {
        choice.env.iter().any(|entry| entry.get("isEnvRef").and_then(|v| v.as_bool()) == Some(true))
    };
    for choice in &harness_choices {
        if choice.spec.transport == "stdio" && has_env_ref(choice) {
            return Some(choice.id.clone());
        }
    }
    for choice in &harness_choices {
        if choice.spec.transport == "stdio" {
            return Some(choice.id.clone());
        }
    }
    for choice in &harness_choices {
        if choice.spec.transport != "stdio"
            && !url_has_embedded_credential(choice.spec.url.as_deref())
        {
            return Some(choice.id.clone());
        }
    }
    harness_choices.first().map(|c| c.id.clone())
}

fn url_has_embedded_credential(url: Option<&str>) -> bool {
    let Some(url) = url else {
        return false;
    };
    let lower = url.to_lowercase();
    ["api_key=", "api-key=", "token=", "secret=", "auth=", "authorization="]
        .iter()
        .any(|token| lower.contains(token))
}

fn spec_has_env_ref(spec: &McpServerSpec) -> bool {
    spec.env
        .as_ref()
        .map(|env| env.values().any(|v| is_env_var_reference(v)))
        .unwrap_or(false)
}
