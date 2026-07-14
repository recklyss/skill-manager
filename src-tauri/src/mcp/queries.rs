use std::collections::HashMap;

use serde_json::{json, Value};

use super::adapters::McpReadModelService;
use super::availability::{
    availability_cache_key, MarketplaceLink, McpAvailabilityProbe, McpAvailabilityResult,
    McpEnrichmentService,
};
use super::config_choice::recommended_observed_harness;
use super::contracts::McpInventoryIssue;
use super::inventory::build_inventory;
use super::managed_state::{detail_extras_payload, entry_payload, inventory_payload};
use super::planner::McpAdoptionPlanner;
use super::redaction::{annotate_redacted_env, redact_payload, redacted_spec_dict};
use super::store::McpServerStore;

#[derive(Clone)]
pub struct McpQueryService {
    read_models: McpReadModelService,
    planner: McpAdoptionPlanner,
    enrichment: McpEnrichmentService,
    availability_probe: McpAvailabilityProbe,
    availability_cache: std::sync::Arc<std::sync::Mutex<HashMap<(String, String), McpAvailabilityResult>>>,
}

impl McpQueryService {
    pub fn new(
        read_models: McpReadModelService,
        planner: McpAdoptionPlanner,
        enrichment: McpEnrichmentService,
    ) -> Self {
        Self {
            read_models,
            planner,
            enrichment,
            availability_probe: McpAvailabilityProbe::default(),
            availability_cache: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn cache_snapshot(&self) -> HashMap<(String, String), McpAvailabilityResult> {
        self.availability_cache
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn list_servers(&self) -> Value {
        let snapshot = self.read_models.snapshot();
        let inventory = self.inventory(&snapshot.harness_scans);
        let visible = self.read_models.visible_scans(&snapshot);
        inventory_payload(&inventory, &visible, &self.cache_snapshot())
    }

    pub fn get_server(&self, name: &str) -> Option<Value> {
        let snapshot = self.read_models.snapshot();
        let inventory = self.inventory(&snapshot.harness_scans);
        let visible = self.read_models.visible_scans(&snapshot);
        let entry = inventory.entries.iter().find(|e| e.name == name)?;
        let mut payload = entry_payload(entry, &visible, &self.cache_snapshot());
        if let Some(spec) = &entry.spec {
            if let Some(obj) = payload.as_object_mut() {
                let extras = detail_extras_payload(name, spec, &visible);
                if let Some(env) = extras.get("env") {
                    obj.insert("env".into(), env.clone());
                }
                if let Some(choices) = extras.get("configChoices") {
                    obj.insert("configChoices".into(), choices.clone());
                }
            }
            if let Some(link) = self.enrichment.lookup(name) {
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("marketplaceLink".into(), link.to_json());
                }
            }
        }
        Some(payload)
    }

    pub fn check_availability(&self, name: &str) -> Option<Value> {
        let snapshot = self.read_models.snapshot();
        let inventory = self.inventory(&snapshot.harness_scans);
        let entry = inventory.entries.iter().find(|e| e.name == name)?;
        let spec = entry.spec.as_ref()?;
        let result = self.availability_probe.probe(spec);
        if let Ok(mut cache) = self.availability_cache.lock() {
            cache.insert(availability_cache_key(name, spec), result.clone());
        }
        Some(json!({
            "ok": true,
            "name": name,
            "availabilityStatus": result.status,
            "availabilityReason": result.reason,
        }))
    }

    pub fn list_unmanaged_by_server(&self) -> Value {
        let snapshot = self.read_models.snapshot();
        let plan = self.planner.plan();
        let visible = self.read_models.visible_scans(&snapshot);
        let visible_harnesses: std::collections::HashSet<_> =
            visible.iter().map(|s| s.harness.clone()).collect();

        let harness_meta: Vec<_> = visible
            .iter()
            .map(|scan| {
                json!({
                    "harness": scan.harness,
                    "label": scan.label,
                    "logoKey": scan.logo_key,
                    "installed": scan.installed,
                    "configPresent": scan.config_present,
                    "configPath": scan.config_path.to_string_lossy(),
                    "mcpWritable": scan.mcp_writable,
                    "mcpUnavailableReason": scan.mcp_unavailable_reason,
                })
            })
            .collect();

        let mut issues_payload: Vec<Value> = visible
            .iter()
            .filter_map(|scan| {
                scan.scan_issue.as_ref().map(|reason| {
                    json!({
                        "harness": scan.harness,
                        "label": scan.label,
                        "logoKey": scan.logo_key,
                        "name": format!("{} config", scan.label),
                        "configPath": scan.config_path.to_string_lossy(),
                        "payloadPreview": Value::Null,
                        "reason": reason,
                    })
                })
            })
            .collect();

        issues_payload.extend(plan.issues.iter().filter_map(|issue| {
            if !visible_harnesses.contains(&issue.harness) {
                return None;
            }
            Some(json!({
                "harness": issue.harness,
                "label": issue.label,
                "logoKey": issue.logo_key,
                "name": issue.name,
                "configPath": issue.config_path,
                "payloadPreview": issue.payload.as_ref().map(|payload| {
                    redact_payload(&json!(payload), "")
                }),
                "reason": issue.reason,
            }))
        }));

        let mut servers_payload = Vec::new();
        for group in plan.groups {
            let sightings: Vec<_> = group
                .sightings
                .iter()
                .filter(|s| visible_harnesses.contains(&s.harness))
                .collect();
            if sightings.is_empty() {
                continue;
            }
            let recommended = recommended_observed_harness(
                &group
                    .sightings
                    .iter()
                    .filter(|s| visible_harnesses.contains(&s.harness))
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            let sightings_payload: Vec<_> = sightings
                .iter()
                .map(|s| {
                    json!({
                        "harness": s.harness,
                        "label": s.label,
                        "logoKey": s.logo_key,
                        "configPath": s.config_path,
                        "payloadPreview": redact_payload(&json!(s.payload), ""),
                        "spec": redacted_spec_dict(&s.spec),
                        "env": annotate_redacted_env(s.spec.env.as_ref()),
                        "recommended": recommended.as_deref() == Some(s.harness.as_str()),
                    })
                })
                .collect();
            let link = self.enrichment.lookup(&group.name);
            servers_payload.push(json!({
                "name": group.name,
                "identical": group.identical,
                "canonicalSpec": group.canonical_spec.as_ref().map(redacted_spec_dict),
                "sightings": sightings_payload,
                "marketplaceLink": link.as_ref().map(MarketplaceLink::to_json),
            }));
        }

        json!({
            "harnesses": harness_meta,
            "servers": servers_payload,
            "issues": issues_payload,
        })
    }

    fn inventory(&self, scans: &[super::contracts::McpHarnessScan]) -> super::contracts::McpInventory {
        let mut issues: Vec<McpInventoryIssue> = self
            .read_models
            .store()
            .manifest_issues()
            .into_iter()
            .map(|issue| McpInventoryIssue {
                name: issue.name,
                reason: issue.reason,
            })
            .collect();
        issues.extend(scans.iter().filter_map(|scan| {
            scan.scan_issue.as_ref().map(|reason| McpInventoryIssue {
                name: format!("{} config", scan.label),
                reason: reason.clone(),
            })
        }));
        build_inventory(
            &self.read_models.store().list_records(),
            &self.read_models.store().list_records(),
            scans,
            issues,
        )
    }
}
