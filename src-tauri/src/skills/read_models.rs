use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::error::{ApiError, ApiResult};
use crate::harness::{FamilyKey, HarnessKernelService};

use super::adapters::{build_skills_adapters, scan_all_adapters, SkillsHarnessAdapter};
use super::inventory::{InventoryColumn, InventoryEntry, SkillInventory};
use super::observations::{SkillStoreScan, SkillsHarnessScan};
use super::policy::{
    can_delete, can_manage, cell_state, display_status, stop_managing_status,
};
use super::store::SkillStore;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessColumnResponse {
    pub harness: String,
    pub installed: bool,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_key: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessCellResponse {
    pub harness: String,
    pub interactive: bool,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_key: Option<String>,
    pub state: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRowActionsResponse {
    pub can_delete: bool,
    pub can_manage: bool,
    pub can_stop_managing: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillTableRowResponse {
    pub actions: SkillRowActionsResponse,
    pub cells: Vec<HarnessCellResponse>,
    pub description: String,
    pub display_status: String,
    pub name: String,
    pub skill_ref: String,
}

#[derive(Debug, Serialize)]
pub struct SkillsSummaryResponse {
    pub managed: usize,
    pub unmanaged: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsPageResponse {
    pub harness_columns: Vec<HarnessColumnResponse>,
    pub rows: Vec<SkillTableRowResponse>,
    pub summary: SkillsSummaryResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetailActionsResponse {
    pub can_manage: bool,
    pub stop_managing_status: Option<String>,
    pub stop_managing_harness_labels: Vec<String>,
    pub can_delete: bool,
    pub delete_harness_labels: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillLocationResponse {
    pub kind: String,
    pub harness: Option<String>,
    pub label: String,
    pub scope: Option<String>,
    pub path: Option<String>,
    pub revision: Option<String>,
    pub source_kind: String,
    pub source_locator: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSourceLinksResponse {
    pub repo_label: String,
    pub repo_url: String,
    pub folder_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetailResponse {
    pub skill_ref: String,
    pub name: String,
    pub description: String,
    pub display_status: String,
    pub attention_message: Option<String>,
    pub actions: SkillDetailActionsResponse,
    pub harness_cells: Vec<HarnessCellResponse>,
    pub locations: Vec<SkillLocationResponse>,
    pub source_links: Option<SkillSourceLinksResponse>,
    pub document_markdown: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSourceStatusResponse {
    pub update_status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SkillsReadModelSnapshot {
    pub store_scan: SkillStoreScan,
    pub harness_scans: Vec<SkillsHarnessScan>,
}

struct CachedSnapshot {
    snapshot: SkillsReadModelSnapshot,
    captured_at: Instant,
}

#[derive(Clone)]
pub struct SkillsReadModelService {
    pub store: SkillStore,
    adapters: Arc<Vec<SkillsHarnessAdapter>>,
    kernel: HarnessKernelService,
    cache: Arc<Mutex<Option<CachedSnapshot>>>,
    snapshot_ttl: Duration,
}

impl SkillsReadModelService {
    pub fn new(store: SkillStore, kernel: HarnessKernelService) -> Self {
        let adapters = Arc::new(build_skills_adapters(&kernel));
        Self {
            store,
            adapters,
            kernel,
            cache: Arc::new(Mutex::new(None)),
            snapshot_ttl: Duration::from_secs_f64(1.0),
        }
    }

    pub fn find_adapter(&self, harness: &str) -> Option<&SkillsHarnessAdapter> {
        self.adapters.iter().find(|a| a.harness == harness)
    }

    pub fn enabled_harnesses(&self) -> Vec<String> {
        self.kernel.enabled_harness_ids_for_family(FamilyKey::Skills)
    }

    pub fn enabled_adapters(&self) -> Vec<&SkillsHarnessAdapter> {
        let enabled: std::collections::HashSet<_> = self.enabled_harnesses().into_iter().collect();
        self.adapters
            .iter()
            .filter(|a| enabled.contains(&a.harness))
            .collect()
    }

    pub fn enabled_installed_adapters(&self) -> Vec<&SkillsHarnessAdapter> {
        self.enabled_adapters()
            .into_iter()
            .filter(|a| a.installed())
            .collect()
    }

    pub fn all_adapters(&self) -> &[SkillsHarnessAdapter] {
        &self.adapters
    }

    pub fn require_enabled_adapter(&self, harness: &str) -> ApiResult<&SkillsHarnessAdapter> {
        let adapter = self
            .find_adapter(harness)
            .ok_or_else(|| ApiError::bad_request(format!("unknown harness: {harness}")))?;
        if !self.enabled_harnesses().iter().any(|h| h == harness) {
            return Err(ApiError::bad_request(format!(
                "harness support is disabled: {harness}"
            )));
        }
        if !adapter.installed() {
            return Err(ApiError::bad_request(format!(
                "{} is not installed or not available on PATH",
                adapter.label
            )));
        }
        Ok(adapter)
    }

    pub fn visible_scans(&self, snapshot: &SkillsReadModelSnapshot) -> Vec<SkillsHarnessScan> {
        let visible: std::collections::HashSet<_> = self.enabled_harnesses().into_iter().collect();
        snapshot
            .harness_scans
            .iter()
            .filter(|scan| visible.contains(&scan.harness))
            .cloned()
            .collect()
    }

    pub fn snapshot(&self) -> SkillsReadModelSnapshot {
        if let Ok(guard) = self.cache.lock() {
            if let Some(cached) = guard.as_ref() {
                if cached.captured_at.elapsed() < self.snapshot_ttl {
                    return cached.snapshot.clone();
                }
            }
        }

        let snapshot = SkillsReadModelSnapshot {
            store_scan: self.store.scan(),
            harness_scans: scan_all_adapters(&self.adapters),
        };

        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some(CachedSnapshot {
                snapshot: snapshot.clone(),
                captured_at: Instant::now(),
            });
        }
        snapshot
    }

    pub fn invalidate(&self) {
        if let Ok(mut guard) = self.cache.lock() {
            *guard = None;
        }
    }

    pub fn inventory(&self) -> SkillInventory {
        let snapshot = self.snapshot();
        SkillInventory::from_snapshot(&snapshot.store_scan, &self.visible_scans(&snapshot))
    }

    pub fn page_response(&self) -> SkillsPageResponse {
        let inventory = self.inventory();
        let managed = inventory
            .entries
            .iter()
            .filter(|e| display_status(e) == "Managed")
            .count();
        let unmanaged = inventory
            .entries
            .iter()
            .filter(|e| display_status(e) == "Unmanaged")
            .count();

        SkillsPageResponse {
            harness_columns: inventory
                .columns
                .iter()
                .map(column_payload)
                .collect(),
            rows: inventory
                .entries
                .iter()
                .map(|entry| row_payload(entry, &inventory.columns))
                .collect(),
            summary: SkillsSummaryResponse { managed, unmanaged },
        }
    }

    pub fn detail_response(&self, entry: &InventoryEntry) -> SkillDetailResponse {
        let inventory = self.inventory();
        let package_root = resolve_detail_package_root(entry);
        let document_markdown = package_root
            .as_ref()
            .map(|p| p.join("SKILL.md"))
            .and_then(|p| std::fs::read_to_string(p).ok());

        let linked_labels = linked_harness_labels(entry, &inventory.columns);

        SkillDetailResponse {
            skill_ref: entry.skill_ref.clone(),
            name: entry.name.clone(),
            description: entry.description.clone(),
            display_status: display_status(entry).to_string(),
            attention_message: super::policy::attention_message(entry).map(str::to_string),
            actions: SkillDetailActionsResponse {
                can_manage: can_manage(entry),
                stop_managing_status: stop_managing_status(entry).map(str::to_string),
                stop_managing_harness_labels: linked_labels.clone(),
                can_delete: can_delete(entry),
                delete_harness_labels: linked_labels,
            },
            harness_cells: inventory
                .columns
                .iter()
                .map(|column| cell_payload(entry, column))
                .collect(),
            locations: entry
                .detail_sightings()
                .into_iter()
                .map(sighting_payload)
                .collect(),
            source_links: None,
            document_markdown: document_markdown,
        }
    }
}

fn column_payload(column: &InventoryColumn) -> HarnessColumnResponse {
    HarnessColumnResponse {
        harness: column.harness.clone(),
        label: column.label.clone(),
        logo_key: column.logo_key.clone(),
        installed: column.installed,
    }
}

fn row_payload(entry: &InventoryEntry, columns: &[InventoryColumn]) -> SkillTableRowResponse {
    SkillTableRowResponse {
        skill_ref: entry.skill_ref.clone(),
        name: entry.name.clone(),
        description: entry.description.clone(),
        display_status: display_status(entry).to_string(),
        actions: SkillRowActionsResponse {
            can_manage: can_manage(entry),
            can_stop_managing: stop_managing_status(entry) == Some("available"),
            can_delete: can_delete(entry),
        },
        cells: columns
            .iter()
            .map(|column| cell_payload(entry, column))
            .collect(),
    }
}

fn cell_payload(entry: &InventoryEntry, column: &InventoryColumn) -> HarnessCellResponse {
    let state = cell_state(entry, &column.harness);
    let interactive = column.installed
        && (matches!(state, "enabled" | "disabled")
            || (state == "found" && entry.kind == "managed"));
    HarnessCellResponse {
        harness: column.harness.clone(),
        label: column.label.clone(),
        logo_key: column.logo_key.clone(),
        state: state.to_string(),
        interactive,
    }
}

fn sighting_payload(sighting: &super::observations::InventorySighting) -> SkillLocationResponse {
    SkillLocationResponse {
        kind: sighting.kind.clone(),
        harness: sighting.harness.clone(),
        label: sighting.label.clone(),
        scope: sighting.scope.clone(),
        path: sighting.path.as_ref().map(|p| p.display().to_string()),
        revision: sighting.revision.clone(),
        source_kind: sighting.source.kind.clone(),
        source_locator: sighting.source.locator.clone(),
        detail: if sighting.detail.is_empty() {
            None
        } else {
            Some(sighting.detail.clone())
        },
    }
}

fn linked_harness_labels(entry: &InventoryEntry, columns: &[InventoryColumn]) -> Vec<String> {
    let linked = entry.linked_harnesses();
    columns
        .iter()
        .filter(|c| linked.contains(&c.harness))
        .map(|c| c.label.clone())
        .collect()
}

fn resolve_detail_package_root(entry: &InventoryEntry) -> Option<std::path::PathBuf> {
    if let Some(path) = &entry.package_path {
        if path.join("SKILL.md").is_file() {
            return Some(path.clone());
        }
    }
    for sighting in entry.detail_sightings() {
        if let Some(path) = &sighting.path {
            if path.join("SKILL.md").is_file() {
                return Some(path.clone());
            }
        }
    }
    None
}
