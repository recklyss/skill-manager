use std::collections::{HashMap, HashSet};

use sha2::{Digest, Sha256};

use super::contracts::McpHarnessScan;
use super::store::{McpServerSpec};

#[derive(Clone)]
pub struct HarnessSighting {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub config_path: Option<String>,
    pub payload: HashMap<String, serde_json::Value>,
    pub spec: McpServerSpec,
}

#[derive(Clone)]
pub struct ServerIdentityGroup {
    pub name: String,
    pub identical: bool,
    pub canonical_spec: Option<McpServerSpec>,
    pub sightings: Vec<HarnessSighting>,
}

#[derive(Clone)]
pub struct AdoptionIssue {
    pub name: String,
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub config_path: Option<String>,
    pub reason: String,
    pub payload: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Clone)]
pub struct AdoptionPlan {
    pub groups: Vec<ServerIdentityGroup>,
    pub issues: Vec<AdoptionIssue>,
}

pub fn build_identity_plan(
    scans: &[McpHarnessScan],
    excluded_names: &[String],
) -> AdoptionPlan {
    let excluded: HashSet<_> = excluded_names.iter().cloned().collect();
    let mut by_name: HashMap<String, Vec<HarnessSighting>> = HashMap::new();
    let mut issues = Vec::new();

    for scan in scans {
        for entry in &scan.entries {
            if entry.state != "unmanaged" {
                continue;
            }
            if excluded.contains(&entry.name) {
                continue;
            }
            let Some(parsed) = entry.parsed_spec.clone() else {
                issues.push(AdoptionIssue {
                    name: entry.name.clone(),
                    harness: scan.harness.clone(),
                    label: scan.label.clone(),
                    logo_key: scan.logo_key.clone(),
                    config_path: if scan.config_present {
                        Some(scan.config_path.to_string_lossy().into_owned())
                    } else {
                        None
                    },
                    reason: entry
                        .parse_issue
                        .clone()
                        .unwrap_or_else(|| "unable to parse unmanaged MCP entry".into()),
                    payload: entry
                        .raw_payload
                        .as_ref()
                        .and_then(|v| v.as_object())
                        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                });
                continue;
            };
            by_name.entry(entry.name.clone()).or_default().push(HarnessSighting {
                harness: scan.harness.clone(),
                label: scan.label.clone(),
                logo_key: scan.logo_key.clone(),
                config_path: if scan.config_present {
                    Some(scan.config_path.to_string_lossy().into_owned())
                } else {
                    None
                },
                payload: entry
                    .raw_payload
                    .as_ref()
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                spec: parsed,
            });
        }
    }

    let mut groups = Vec::new();
    let mut names: Vec<_> = by_name.keys().cloned().collect();
    names.sort();
    for name in names {
        let sightings = by_name.remove(&name).unwrap_or_default();
        let keys: HashSet<_> = sightings.iter().map(|s| structural_key(&s.spec)).collect();
        let identical = keys.len() == 1;
        groups.push(ServerIdentityGroup {
            name,
            identical,
            canonical_spec: if identical {
                sightings.first().map(|s| s.spec.clone())
            } else {
                None
            },
            sightings,
        });
    }
    AdoptionPlan { groups, issues }
}

fn structural_key(spec: &McpServerSpec) -> String {
    let payload = serde_json::json!({
        "name": spec.name,
        "transport": spec.transport,
        "command": spec.command,
        "args": spec.args,
        "env": spec.env,
        "url": spec.url,
        "headers": spec.headers,
    });
    let digest = Sha256::digest(serde_json::to_string(&payload).unwrap_or_default().as_bytes());
    format!("{:x}", digest)
}
