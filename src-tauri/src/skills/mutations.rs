use crate::error::{ApiError, ApiResult};

use super::adapters::SkillsHarnessAdapter;
use super::inventory::InventoryEntry;
use super::policy::{can_delete, can_manage, can_stop_managing, display_status};
use super::queries::SkillsQueryService;
use super::read_models::SkillsReadModelService;
use super::source_fetch::SourceFetchService;

#[derive(Clone)]
pub struct SkillsMutationService {
    read_models: SkillsReadModelService,
    queries: SkillsQueryService,
    source_fetcher: SourceFetchService,
}

impl SkillsMutationService {
    pub fn new(
        read_models: SkillsReadModelService,
        queries: SkillsQueryService,
        source_fetcher: SourceFetchService,
    ) -> Self {
        Self {
            read_models,
            queries,
            source_fetcher,
        }
    }

    pub fn enable_skill(&self, skill_ref: &str, harness: &str) -> ApiResult<serde_json::Value> {
        let entry = self.queries.require_entry(skill_ref)?;
        if entry.kind != "managed" {
            return Err(ApiError::bad_request(format!(
                "only managed skills can be toggled; this is {}",
                display_status(&entry)
            )));
        }
        let package_path = entry
            .package_path
            .as_ref()
            .ok_or_else(|| ApiError::internal("managed skill is missing its shared package path"))?;
        let adapter = self.read_models.require_enabled_adapter(harness)?;
        adapter.enable_shared_package(package_path)?;
        self.read_models.invalidate();
        Ok(serde_json::json!({ "ok": true }))
    }

    pub fn disable_skill(&self, skill_ref: &str, harness: &str) -> ApiResult<serde_json::Value> {
        let entry = self.queries.require_entry(skill_ref)?;
        if entry.kind != "managed" {
            return Err(ApiError::bad_request(format!(
                "only managed skills can be toggled; this is {}",
                display_status(&entry)
            )));
        }
        let package_dir = entry
            .package_dir
            .as_deref()
            .ok_or_else(|| ApiError::internal("managed skill is missing its package directory name"))?;
        let adapter = self.read_models.require_enabled_adapter(harness)?;
        adapter.disable_shared_package(package_dir)?;
        self.read_models.invalidate();
        Ok(serde_json::json!({ "ok": true }))
    }

    pub fn set_skill_all_harnesses(
        &self,
        skill_ref: &str,
        target: &str,
    ) -> ApiResult<serde_json::Value> {
        if target != "enabled" && target != "disabled" {
            return Err(ApiError::bad_request("target must be 'enabled' or 'disabled'"));
        }
        let entry = self.queries.require_entry(skill_ref)?;
        if entry.kind != "managed" {
            return Err(ApiError::bad_request(format!(
                "only managed skills can be toggled; this is {}",
                display_status(&entry)
            )));
        }
        let package_dir = entry
            .package_dir
            .as_deref()
            .ok_or_else(|| ApiError::internal("managed skill is missing its package directory name"))?;
        let package_path = entry.package_path.as_deref();
        if target == "enabled" && package_path.is_none() {
            return Err(ApiError::internal("managed skill is missing its shared package path"));
        }

        let mut succeeded = Vec::new();
        let mut failures = Vec::new();
        let mut flipped_any = false;

        for adapter in self.read_models.enabled_installed_adapters() {
            let has_binding = adapter.has_binding(package_dir);
            if target == "enabled" && has_binding {
                continue;
            }
            if target == "disabled" && !has_binding {
                continue;
            }
            let result = if target == "enabled" {
                adapter.enable_shared_package(package_path.unwrap())
            } else {
                adapter.disable_shared_package(package_dir)
            };
            match result {
                Ok(()) => {
                    succeeded.push(adapter.harness.clone());
                    flipped_any = true;
                }
                Err(error) => failures.push(serde_json::json!({
                    "harness": adapter.harness,
                    "error": error.message,
                })),
            }
        }

        if flipped_any {
            self.read_models.invalidate();
        }

        Ok(serde_json::json!({
            "ok": failures.is_empty(),
            "succeeded": succeeded,
            "failed": failures,
        }))
    }

    pub fn manage_skill(&self, skill_ref: &str) -> ApiResult<serde_json::Value> {
        let entry = self.queries.require_entry(skill_ref)?;
        if entry.kind != "unmanaged" {
            return Err(ApiError::bad_request(format!(
                "only unmanaged skills can be managed; this is {}",
                display_status(&entry)
            )));
        }
        self.manage_entry(&entry)?;
        self.read_models.invalidate();
        Ok(serde_json::json!({ "ok": true }))
    }

    pub fn manage_all_skills(&self) -> ApiResult<serde_json::Value> {
        let inventory = self.read_models.inventory();
        let mut managed_count = 0usize;
        let mut skipped_count = 0usize;
        let mut failures = Vec::new();

        for entry in &inventory.entries {
            if !can_manage(entry) {
                skipped_count += 1;
                continue;
            }
            if let Err(error) = self.manage_entry(entry) {
                failures.push(serde_json::json!({
                    "skillRef": entry.skill_ref,
                    "name": entry.name,
                    "error": error.message,
                }));
            } else {
                managed_count += 1;
            }
        }

        if managed_count > 0 {
            self.read_models.invalidate();
        }

        Ok(serde_json::json!({
            "ok": failures.is_empty(),
            "managedCount": managed_count,
            "skippedCount": skipped_count,
            "failures": failures,
        }))
    }

    pub fn update_skill(&self, skill_ref: &str) -> ApiResult<serde_json::Value> {
        let entry = self.queries.require_entry(skill_ref)?;
        if !super::policy::can_update(&entry) {
            if super::policy::has_local_changes(&entry) {
                return Err(ApiError::bad_request(
                    "Local changes detected. Source updates are disabled.",
                ));
            }
            return Err(ApiError::bad_request("skill cannot be updated from its source"));
        }
        let package_dir = entry
            .package_dir
            .as_deref()
            .ok_or_else(|| ApiError::internal("managed skill is missing its package directory name"))?;
        let work_dir = tempfile::tempdir().map_err(|e| ApiError::internal(e.to_string()))?;
        let fetched = self.source_fetcher.fetch_package(
            &entry.source.kind,
            &entry.source.locator,
            work_dir.path(),
        )?;
        self.read_models
            .store
            .update(
                package_dir,
                &fetched.package_path,
                fetched.source_ref.as_deref(),
                fetched.source_path.as_deref(),
            )
            .map_err(ApiError::conflict)?;
        self.read_models.invalidate();
        Ok(serde_json::json!({ "ok": true }))
    }

    pub fn unmanage_skill(&self, skill_ref: &str) -> ApiResult<serde_json::Value> {
        let entry = self.queries.require_entry(skill_ref)?;
        if !can_stop_managing(&entry) {
            return Err(ApiError::bad_request(format!(
                "only managed shared-store skills can be moved back to unmanaged; this is {}",
                display_status(&entry)
            )));
        }
        let package_dir = entry
            .package_dir
            .as_deref()
            .ok_or_else(|| ApiError::internal("managed skill is missing its package directory name"))?;
        let package_path = entry
            .package_path
            .as_ref()
            .ok_or_else(|| ApiError::internal("managed skill is missing its shared package metadata"))?;

        let (enabled_bindings, disabled_bindings) = self.partition_bound_adapters(package_dir);
        if !disabled_bindings.is_empty() {
            return Err(ApiError::conflict(format!(
                "cannot stop managing while disabled harnesses still have bindings: {}; re-enable support or clean them manually",
                describe_harnesses(&disabled_bindings)
            )));
        }
        if enabled_bindings.is_empty() {
            return Err(ApiError::bad_request(
                "turn on at least one harness before stopping management",
            ));
        }

        self.read_models
            .store
            .ensure_deletable(package_dir)
            .map_err(ApiError::conflict)?;

        for adapter in &enabled_bindings {
            adapter.prepare_materialize(package_dir, package_path)?;
        }
        for adapter in &enabled_bindings {
            adapter.materialize_binding(package_dir, package_path)?;
        }
        self.read_models
            .store
            .delete(package_dir)
            .map_err(ApiError::conflict)?;
        self.read_models.invalidate();
        Ok(serde_json::json!({ "ok": true }))
    }

    pub fn delete_skill(&self, skill_ref: &str) -> ApiResult<serde_json::Value> {
        let entry = self.queries.require_entry(skill_ref)?;
        if !can_delete(&entry) {
            return Err(ApiError::bad_request(format!(
                "only managed shared-store skills can be deleted; this is {}",
                display_status(&entry)
            )));
        }
        let package_dir = entry
            .package_dir
            .as_deref()
            .ok_or_else(|| ApiError::internal("managed skill is missing its package directory name"))?;

        let (enabled_bindings, disabled_bindings) = self.partition_bound_adapters(package_dir);
        if !disabled_bindings.is_empty() {
            return Err(ApiError::conflict(format!(
                "cannot delete while disabled harnesses still have bindings: {}; re-enable support or clean them manually",
                describe_harnesses(&disabled_bindings)
            )));
        }

        self.read_models
            .store
            .ensure_deletable(package_dir)
            .map_err(ApiError::conflict)?;

        for adapter in &enabled_bindings {
            adapter.prepare_remove(package_dir)?;
        }
        for adapter in &enabled_bindings {
            adapter.remove_binding(package_dir)?;
        }
        self.read_models
            .store
            .delete(package_dir)
            .map_err(ApiError::conflict)?;
        self.read_models.invalidate();
        Ok(serde_json::json!({ "ok": true }))
    }

    fn manage_entry(&self, entry: &InventoryEntry) -> ApiResult<()> {
        let harness_sightings: Vec<_> = entry
            .sightings
            .iter()
            .filter(|s| s.kind == "harness" && s.path.is_some())
            .collect();
        if harness_sightings.is_empty() {
            return Err(ApiError::bad_request("no local skill copy found to manage"));
        }

        let first = harness_sightings[0];
        let source_path = first.path.as_ref().unwrap();
        let (source_kind, source_locator) = if first.source.is_source_backed() {
            (first.source.kind.clone(), first.source.locator.clone())
        } else {
            (
                "centralized".to_string(),
                format!("centralized:{}", entry.name),
            )
        };

        let ingested = self
            .read_models
            .store
            .ingest(
                source_path,
                &entry.name,
                &source_kind,
                &source_locator,
                None,
                None,
                origin_harness_for_entry(&harness_sightings).as_deref(),
            )
            .map_err(ApiError::conflict)?;

        let mut canonical_bound = std::collections::HashSet::new();
        for sighting in &harness_sightings {
            let adapter = self
                .read_models
                .require_enabled_adapter(sighting.harness.as_deref().unwrap())?;
            if sighting.scope.as_deref() == Some("canonical") {
                adapter.adopt_local_copy(sighting.path.as_ref().unwrap(), &ingested)?;
                canonical_bound.insert(sighting.harness.clone().unwrap());
            }
        }
        for sighting in &harness_sightings {
            let harness = sighting.harness.as_deref().unwrap();
            if canonical_bound.contains(harness) {
                continue;
            }
            let adapter = self.read_models.require_enabled_adapter(harness)?;
            adapter.enable_shared_package(&ingested)?;
            canonical_bound.insert(harness.to_string());
        }
        Ok(())
    }

    fn partition_bound_adapters(
        &self,
        package_dir: &str,
    ) -> (Vec<&SkillsHarnessAdapter>, Vec<&SkillsHarnessAdapter>) {
        let enabled: std::collections::HashSet<_> =
            self.read_models.enabled_harnesses().into_iter().collect();
        let mut enabled_bindings = Vec::new();
        let mut disabled_bindings = Vec::new();
        for adapter in self.read_models.all_adapters() {
            if !adapter.has_binding(package_dir) {
                continue;
            }
            if enabled.contains(&adapter.harness) {
                enabled_bindings.push(adapter);
            } else {
                disabled_bindings.push(adapter);
            }
        }
        (enabled_bindings, disabled_bindings)
    }
}

fn describe_harnesses(bindings: &[&SkillsHarnessAdapter]) -> String {
    bindings
        .iter()
        .map(|a| a.label.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn origin_harness_for_entry(
    harness_sightings: &[&super::observations::InventorySighting],
) -> Option<String> {
    for sighting in harness_sightings {
        if sighting.scope.as_deref() == Some("canonical") {
            return sighting.harness.clone();
        }
    }
    harness_sightings
        .first()
        .and_then(|s| s.harness.clone())
}
