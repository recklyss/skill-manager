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
pub struct HarnessColumnResponse {
    pub harness: String,
    pub installed: bool,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logoKey: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HarnessCellResponse {
    pub harness: String,
    pub interactive: bool,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logoKey: Option<String>,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct SkillRowActionsResponse {
    pub canDelete: bool,
    pub canManage: bool,
    pub canStopManaging: bool,
}

#[derive(Debug, Serialize)]
pub struct SkillTableRowResponse {
    pub actions: SkillRowActionsResponse,
    pub cells: Vec<HarnessCellResponse>,
    pub description: String,
    pub displayStatus: String,
    pub name: String,
    pub skillRef: String,
}

#[derive(Debug, Serialize)]
pub struct SkillsSummaryResponse {
    pub managed: usize,
    pub unmanaged: usize,
}

#[derive(Debug, Serialize)]
pub struct SkillsPageResponse {
    #[serde(rename = "harnessColumns")]
    pub harness_columns: Vec<HarnessColumnResponse>,
    pub rows: Vec<SkillTableRowResponse>,
    pub summary: SkillsSummaryResponse,
}

#[derive(Debug, Serialize)]
pub struct SkillDetailActionsResponse {
    pub canManage: bool,
    pub stopManagingStatus: Option<String>,
    pub stopManagingHarnessLabels: Vec<String>,
    pub canDelete: bool,
    pub deleteHarnessLabels: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SkillLocationResponse {
    pub kind: String,
    pub harness: Option<String>,
    pub label: String,
    pub scope: Option<String>,
    pub path: Option<String>,
    pub revision: Option<String>,
    pub sourceKind: String,
    pub sourceLocator: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SkillSourceLinksResponse {
    pub repoLabel: String,
    pub repoUrl: String,
    pub folderUrl: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SkillDetailResponse {
    pub skillRef: String,
    pub name: String,
    pub description: String,
    pub displayStatus: String,
    pub attentionMessage: Option<String>,
    pub actions: SkillDetailActionsResponse,
    pub harnessCells: Vec<HarnessCellResponse>,
    pub locations: Vec<SkillLocationResponse>,
    pub sourceLinks: Option<SkillSourceLinksResponse>,
    pub documentMarkdown: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SkillSourceStatusResponse {
    pub updateStatus: Option<String>,
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
            skillRef: entry.skill_ref.clone(),
            name: entry.name.clone(),
            description: entry.description.clone(),
            displayStatus: display_status(entry).to_string(),
            attentionMessage: super::policy::attention_message(entry).map(str::to_string),
            actions: SkillDetailActionsResponse {
                canManage: can_manage(entry),
                stopManagingStatus: stop_managing_status(entry).map(str::to_string),
                stopManagingHarnessLabels: linked_labels.clone(),
                canDelete: can_delete(entry),
                deleteHarnessLabels: linked_labels,
            },
            harnessCells: inventory
                .columns
                .iter()
                .map(|column| cell_payload(entry, column))
                .collect(),
            locations: entry
                .detail_sightings()
                .into_iter()
                .map(sighting_payload)
                .collect(),
            sourceLinks: None,
            documentMarkdown: document_markdown,
        }
    }
}

fn column_payload(column: &InventoryColumn) -> HarnessColumnResponse {
    HarnessColumnResponse {
        harness: column.harness.clone(),
        label: column.label.clone(),
        logoKey: column.logo_key.clone(),
        installed: column.installed,
    }
}

fn row_payload(entry: &InventoryEntry, columns: &[InventoryColumn]) -> SkillTableRowResponse {
    SkillTableRowResponse {
        skillRef: entry.skill_ref.clone(),
        name: entry.name.clone(),
        description: entry.description.clone(),
        displayStatus: display_status(entry).to_string(),
        actions: SkillRowActionsResponse {
            canManage: can_manage(entry),
            canStopManaging: stop_managing_status(entry) == Some("available"),
            canDelete: can_delete(entry),
        },
        cells: columns
            .iter()
            .map(|column| cell_payload(entry, column))
            .collect(),
    }
}

fn cell_payload(entry: &InventoryEntry, column: &InventoryColumn) -> HarnessCellResponse {
    let state = cell_state(entry, &column.harness);
    let interactive = matches!(state, "enabled" | "disabled") && column.installed;
    HarnessCellResponse {
        harness: column.harness.clone(),
        label: column.label.clone(),
        logoKey: column.logo_key.clone(),
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
        sourceKind: sighting.source.kind.clone(),
        sourceLocator: sighting.source.locator.clone(),
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
