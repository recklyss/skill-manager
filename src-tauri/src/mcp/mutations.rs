use std::collections::HashSet;

use serde_json::{json, Value};

use super::adapters::McpReadModelService;
use super::availability::{availability_cache_key, McpAvailabilityProbe};
use super::config_choice::observed_spec_from_scans;
use super::harness_application::{harnesses_in_states, McpHarnessApplication};
use super::planner::McpAdoptionPlanner;
use super::redaction::redacted_spec_dict;
use super::store::{prepare_managed_spec, McpServerSpec, McpServerStore, McpSource};
use crate::error::{ApiError, ApiResult};
use crate::marketplace::McpMarketplaceService;
use super::availability::McpEnrichmentService;

#[derive(Clone)]
pub struct McpMutationService {
    store: McpServerStore,
    read_models: McpReadModelService,
    planner: McpAdoptionPlanner,
    marketplace: McpMarketplaceService,
    enrichment: McpEnrichmentService,
    harness_application: McpHarnessApplication,
    availability_probe: McpAvailabilityProbe,
    availability_cache: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<(String, String), super::availability::McpAvailabilityResult>>>,
}

impl McpMutationService {
    pub fn new(
        store: McpServerStore,
        read_models: McpReadModelService,
        planner: McpAdoptionPlanner,
        marketplace: McpMarketplaceService,
        enrichment: McpEnrichmentService,
    ) -> Self {
        let harness_application = McpHarnessApplication::new(read_models.clone());
        Self {
            store,
            read_models,
            planner,
            marketplace,
            enrichment,
            harness_application,
            availability_probe: McpAvailabilityProbe::default(),
            availability_cache: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn install_from_marketplace(&self, qualified_name: &str) -> ApiResult<Value> {
        if qualified_name.trim().is_empty() {
            return Err(ApiError::bad_request("qualifiedName is required"));
        }
        let detail = self
            .marketplace
            .install_detail(qualified_name)
            .await?
            .ok_or_else(|| ApiError::not_found(format!("server not found in marketplace: {qualified_name}")))?;
        let mut spec = spec_from_marketplace_detail(&detail, qualified_name)?;
        let reinstalled = if let Some(existing) = self.managed_for_marketplace(qualified_name) {
            spec.name = existing.name;
            true
        } else if self.store.get_record(&spec.name).is_some() {
            return Err(ApiError::conflict(format!(
                "a server named '{}' is already installed",
                spec.name
            )));
        } else {
            false
        };
        let stored = self.store.upsert(spec).map_err(ApiError::bad_request)?;
        self.read_models.invalidate();
        if let Ok(mut cache) = self.availability_cache.lock() {
            cache.insert(
                availability_cache_key(&stored.name, &stored),
                self.availability_probe.probe(&stored),
            );
        }
        Ok(json!({
            "ok": true,
            "reinstalled": reinstalled,
            "server": redacted_spec_dict(&stored)
        }))
    }

    pub fn uninstall_server(&self, name: &str) -> ApiResult<Value> {
        if self.store.get_record(name).is_none() {
            return Err(ApiError::not_found(format!("unknown server: {name}")));
        }
        let bound = harnesses_in_states(&self.read_models, name, &["managed", "drifted"], false);
        let result = self.harness_application.disable_many(name, &bound, false);
        if result.ok() {
            self.store.remove(name).map_err(ApiError::internal)?;
        }
        Ok(result.to_json())
    }

    pub fn enable_server(&self, name: &str, harness: &str, _config: Option<Value>) -> ApiResult<Value> {
        let spec = self
            .store
            .get_record(name)
            .ok_or_else(|| ApiError::not_found(format!("unknown server: {name}")))?;
        let adapter = self
            .read_models
            .require_enabled_adapter(harness)
            .map_err(ApiError::bad_request)?;
        if adapter.has_binding(name) {
            return Ok(json!({ "ok": true }));
        }
        let result = self.harness_application.enable_one(adapter, &spec);
        if !result.failed.is_empty() {
            return Err(ApiError::bad_request(
                result.failed[0]
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("enable failed")
                    .to_string(),
            ));
        }
        Ok(json!({ "ok": true }))
    }

    pub fn disable_server(&self, name: &str, harness: &str) -> ApiResult<Value> {
        if self.store.get_record(name).is_none() {
            return Err(ApiError::not_found(format!("unknown server: {name}")));
        }
        let adapter = self
            .read_models
            .require_enabled_adapter(harness)
            .map_err(ApiError::bad_request)?;
        adapter.disable_server(name).map_err(ApiError::bad_request)?;
        self.read_models.invalidate();
        Ok(json!({ "ok": true }))
    }

    pub fn set_server_all_harnesses(
        &self,
        name: &str,
        target: &str,
        _config: Option<Value>,
    ) -> ApiResult<Value> {
        if target != "enabled" && target != "disabled" {
            return Err(ApiError::bad_request("target must be 'enabled' or 'disabled'"));
        }
        let spec = self
            .store
            .get_record(name)
            .ok_or_else(|| ApiError::not_found(format!("unknown server: {name}")))?;
        let bound_now = harnesses_in_states(&self.read_models, name, &["managed", "drifted"], false);
        let result = if target == "enabled" {
            self.harness_application.enable_many(
                &spec,
                &self.read_models.enabled_harnesses(),
                true,
                &bound_now,
            )
        } else {
            self.harness_application
                .disable_many(name, &bound_now, true)
        };
        Ok(result.to_json())
    }

    pub fn reconcile_server(
        &self,
        name: &str,
        source_kind: &str,
        observed_harness: Option<String>,
        harnesses: Option<Vec<String>>,
    ) -> ApiResult<Value> {
        if self.store.get_record(name).is_none() {
            return Err(ApiError::not_found(format!("unknown server: {name}")));
        }
        let target_harnesses: HashSet<String> = if let Some(list) = harnesses {
            list.into_iter().collect()
        } else {
            harnesses_in_states(&self.read_models, name, &["managed", "drifted"], true)
                .into_iter()
                .collect()
        };
        let current = self
            .store
            .get_record(name)
            .ok_or_else(|| ApiError::not_found(format!("unknown server: {name}")))?;
        let source_spec = match source_kind {
            "managed" => current.clone(),
            "harness" => {
                let observed = observed_harness.ok_or_else(|| {
                    ApiError::bad_request("observedHarness is required when sourceKind is 'harness'")
                })?;
                let snapshot = self.read_models.snapshot();
                let mut observed_spec =
                    observed_spec_from_scans(name, &observed, &snapshot.harness_scans)
                        .map_err(ApiError::bad_request)?;
                observed_spec.name = current.name.clone();
                observed_spec.display_name = current.display_name.clone();
                observed_spec.source = current.source.clone();
                observed_spec
            }
            _ => return Err(ApiError::bad_request("sourceKind must be 'managed' or 'harness'")),
        };
        let result = self.harness_application.enable_many(
            &source_spec,
            &target_harnesses.into_iter().collect::<Vec<_>>(),
            false,
            &[],
        );
        if !result.succeeded.is_empty() && source_spec.revision != current.revision {
            self.store.upsert(source_spec.clone()).map_err(ApiError::bad_request)?;
        }
        let stored = self.store.get_record(name).unwrap_or(source_spec);
        let mut payload = result.to_json();
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("server".into(), redacted_spec_dict(&stored));
        }
        Ok(payload)
    }

    pub fn adopt(
        &self,
        name: &str,
        observed_harness: Option<&str>,
        harnesses: Option<Vec<String>>,
    ) -> ApiResult<Value> {
        if self.store.get_record(name).is_some() {
            return Err(ApiError::conflict(format!(
                "a managed server named '{name}' already exists"
            )));
        }
        let group = self
            .planner
            .require_group(name)
            .map_err(|error| ApiError::not_found(error))?;
        let target_spec = if let Some(harness) = observed_harness {
            group
                .sightings
                .iter()
                .find(|s| s.harness == harness)
                .map(|s| s.spec.clone())
                .ok_or_else(|| {
                    ApiError::bad_request(format!(
                        "server '{name}' was not observed in harness '{harness}'"
                    ))
                })?
        } else {
            group.canonical_spec.clone().ok_or_else(|| {
                ApiError::conflict(format!(
                    "server '{name}' has different configs across harnesses; choose an observedHarness to adopt"
                ))
            })?
        };
        let mut target_spec = if target_spec.name != name {
            McpServerSpec {
                name: name.to_string(),
                ..target_spec
            }
        } else {
            target_spec
        };
        target_spec = self.apply_enrichment(target_spec);
        let target_harnesses: HashSet<String> = if let Some(list) = harnesses {
            list.into_iter().collect()
        } else {
            group.sightings.iter().map(|s| s.harness.clone()).collect()
        };
        let result = self.harness_application.enable_many(
            &target_spec,
            &target_harnesses.into_iter().collect::<Vec<_>>(),
            false,
            &[],
        );
        if !result.succeeded.is_empty() {
            self.store
                .upsert(prepare_managed_spec(target_spec.clone()))
                .map_err(ApiError::bad_request)?;
        }
        let stored = self.store.get_record(name).unwrap_or(target_spec);
        let mut payload = result.to_json();
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("server".into(), redacted_spec_dict(&stored));
        }
        Ok(payload)
    }

    fn apply_enrichment(&self, mut spec: McpServerSpec) -> McpServerSpec {
        let Some(link) = self.enrichment.lookup(&spec.name) else {
            return spec;
        };
        spec.display_name = if link.display_name.is_empty() {
            spec.display_name
        } else {
            link.display_name
        };
        spec.source = McpSource::marketplace(&link.qualified_name);
        spec
    }

    fn managed_for_marketplace(&self, qualified_name: &str) -> Option<McpServerSpec> {
        self.store.list_records().into_iter().find(|server| {
            server.source.kind == "marketplace" && server.source.locator == qualified_name
        })
    }
}

fn spec_from_marketplace_detail(detail: &Value, qualified_name: &str) -> Result<McpServerSpec, ApiError> {
    let name = detail
        .get("managedName")
        .or_else(|| detail.get("qualifiedName"))
        .or_else(|| detail.get("displayName"))
        .and_then(|v| v.as_str())
        .unwrap_or(qualified_name)
        .to_string();
    let connection = detail.get("connection");
    let transport = connection
        .and_then(|v| v.get("kind"))
        .or_else(|| connection.and_then(|v| v.get("type")))
        .and_then(|v| v.as_str())
        .unwrap_or("stdio")
        .to_string();
    Ok(McpServerSpec {
        name: name.clone(),
        display_name: detail
            .get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or(&name)
            .to_string(),
        source: McpSource::marketplace(qualified_name),
        transport: transport.clone(),
        command: Some("npx".into()),
        args: Some(vec!["-y".into(), format!("{qualified_name}@latest")]),
        env: None,
        url: if transport == "http" || transport == "sse" {
            connection
                .and_then(|v| v.get("deploymentUrl"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        } else {
            None
        },
        headers: None,
        installed_at: String::new(),
        revision: String::new(),
    })
}
