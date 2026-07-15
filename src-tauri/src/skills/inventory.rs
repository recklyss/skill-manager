use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::identity::{stable_id, SourceDescriptor};
use super::observations::{
    InventorySighting, SkillObservation, SkillStoreScan, SkillsHarnessScan,
};
use super::package::SkillPackage;
use super::policy;

#[derive(Debug, Clone)]
pub struct InventoryColumn {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub installed: bool,
}

#[derive(Debug, Clone)]
pub struct InventoryEntry {
    pub skill_ref: String,
    pub name: String,
    pub description: String,
    pub kind: String,
    pub source: SourceDescriptor,
    pub current_revision: Option<String>,
    pub recorded_revision: Option<String>,
    pub source_ref: Option<String>,
    pub source_path: Option<String>,
    pub package_dir: Option<String>,
    pub package_path: Option<PathBuf>,
    pub origin_harness: Option<String>,
    pub sightings: Vec<InventorySighting>,
}

impl InventoryEntry {
    pub fn detail_sightings(&self) -> Vec<&InventorySighting> {
        let mut sightings: Vec<_> = self.sightings.iter().collect();
        sightings.sort_by_key(|s| {
            let kind_order = if s.kind == "shared" { 0 } else { 1 };
            (
                kind_order,
                s.harness.clone().unwrap_or_default(),
                s.scope.clone().unwrap_or_default(),
                s.label.clone(),
                s.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
            )
        });
        sightings
    }

    pub fn linked_harnesses(&self) -> HashSet<String> {
        self.sightings
            .iter()
            .filter(|s| {
                s.kind == "harness"
                    && s.harness.is_some()
                    && s.scope.as_deref() == Some("canonical")
            })
            .filter_map(|s| s.harness.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct SkillInventory {
    pub columns: Vec<InventoryColumn>,
    pub harness_scans: Vec<SkillsHarnessScan>,
    pub store_issues: Vec<String>,
    pub entries: Vec<InventoryEntry>,
    by_ref: HashMap<String, usize>,
}

impl SkillInventory {
    pub fn from_snapshot(store_scan: &SkillStoreScan, harness_scans: &[SkillsHarnessScan]) -> Self {
        let columns: Vec<InventoryColumn> = harness_scans
            .iter()
            .map(|scan| InventoryColumn {
                harness: scan.harness.clone(),
                label: scan.label.clone(),
                logo_key: scan.logo_key.clone(),
                installed: scan.installed,
            })
            .collect();

        let mut entries: Vec<InventoryEntry> = Vec::new();
        let mut shared_path_index: HashMap<PathBuf, usize> = HashMap::new();
        let mut shared_match_index: HashMap<String, usize> = HashMap::new();
        let mut managed_name_index: HashMap<String, usize> = HashMap::new();
        let excluded_hermes_names = excluded_hermes_names(harness_scans);

        for store_package in &store_scan.packages {
            let package = &store_package.package;
            if is_excluded_hermes_store_package(
                &package.declared_name,
                package.root_path.file_name().unwrap().to_string_lossy().as_ref(),
                store_package.origin_harness.as_deref(),
                &package.source.kind,
                &excluded_hermes_names,
            ) {
                continue;
            }
            let entry = InventoryEntry {
                skill_ref: format!("shared:{}", package.root_path.file_name().unwrap().to_string_lossy()),
                name: package.declared_name.clone(),
                description: package.description.clone(),
                kind: "managed".into(),
                source: package.source.clone(),
                current_revision: Some(package.revision.clone()),
                recorded_revision: store_package.recorded_revision.clone(),
                source_ref: store_package.recorded_source_ref.clone(),
                source_path: store_package.recorded_source_path.clone(),
                package_dir: Some(package.root_path.file_name().unwrap().to_string_lossy().to_string()),
                package_path: Some(package.root_path.clone()),
                origin_harness: store_package.origin_harness.clone(),
                sightings: vec![InventorySighting {
                    kind: "shared".into(),
                    harness: None,
                    label: "Shared Store".into(),
                    scope: None,
                    path: Some(package.root_path.clone()),
                    revision: Some(package.revision.clone()),
                    source: package.source.clone(),
                    detail: String::new(),
                }],
            };
            let idx = entries.len();
            shared_path_index.insert(package.resolved_path.clone(), idx);
            shared_match_index.insert(managed_entry_key(&entry), idx);
            managed_name_index.insert(package.declared_name.to_ascii_lowercase(), idx);
            entries.push(entry);
        }

        let mut unmanaged_entries: HashMap<String, usize> = HashMap::new();

        for scan in harness_scans {
            for observation in &scan.skills {
                let shared_entry = shared_path_index.get(&observation.package.resolved_path).copied();
                let sighting = observation_to_sighting(observation);
                if let Some(idx) = shared_entry {
                    entries[idx].sightings.push(sighting);
                    continue;
                }
                let match_key = observation_match_key(&observation.package);
                if let Some(idx) = shared_match_index.get(&match_key).copied() {
                    entries[idx].sightings.push(sighting);
                    continue;
                }
                if let Some(idx) = managed_name_index
                    .get(&observation.package.declared_name.to_ascii_lowercase())
                    .copied()
                {
                    entries[idx].sightings.push(sighting);
                    continue;
                }

                let key = unmanaged_entry_key(
                    &observation.package.declared_name,
                    &observation.package.source,
                    &observation.package.revision,
                );
                if let Some(idx) = unmanaged_entries.get(&key).copied() {
                    entries[idx].sightings.push(sighting);
                } else {
                    let entry = InventoryEntry {
                        skill_ref: format!("unmanaged:{key}"),
                        name: observation.package.declared_name.clone(),
                        description: observation.package.description.clone(),
                        kind: "unmanaged".into(),
                        source: observation.package.source.clone(),
                        current_revision: Some(observation.package.revision.clone()),
                        recorded_revision: None,
                        source_ref: None,
                        source_path: None,
                        package_dir: None,
                        package_path: None,
                        origin_harness: None,
                        sightings: vec![sighting],
                    };
                    let idx = entries.len();
                    unmanaged_entries.insert(key, idx);
                    entries.push(entry);
                }
            }
        }

        policy::sort_entries(&mut entries);
        let by_ref = entries
            .iter()
            .enumerate()
            .map(|(i, e)| (e.skill_ref.clone(), i))
            .collect();

        Self {
            columns,
            harness_scans: harness_scans.to_vec(),
            store_issues: store_scan.issues.clone(),
            entries,
            by_ref,
        }
    }

    pub fn find(&self, skill_ref: &str) -> Option<&InventoryEntry> {
        self.by_ref.get(skill_ref).map(|&i| &self.entries[i])
    }
}

fn observation_to_sighting(observation: &SkillObservation) -> InventorySighting {
    InventorySighting {
        kind: "harness".into(),
        harness: Some(observation.harness.clone()),
        label: observation.label.clone(),
        scope: Some(observation.scope.clone()),
        path: Some(observation.package.root_path.clone()),
        revision: Some(observation.package.revision.clone()),
        source: observation.package.source.clone(),
        detail: String::new(),
    }
}

fn excluded_hermes_names(harness_scans: &[SkillsHarnessScan]) -> HashSet<String> {
    let mut names = HashSet::new();
    for scan in harness_scans {
        if scan.harness == "hermes" {
            names.extend(scan.excluded_skill_names.iter().cloned());
        }
    }
    names
}

fn is_excluded_hermes_store_package(
    name: &str,
    package_dir: &str,
    origin_harness: Option<&str>,
    source_kind: &str,
    excluded_hermes_names: &HashSet<String>,
) -> bool {
    if origin_harness != Some("hermes") {
        return false;
    }
    if excluded_hermes_names.contains(name) || excluded_hermes_names.contains(package_dir) {
        return true;
    }
    source_kind == "centralized"
}

fn unmanaged_entry_key(name: &str, source: &SourceDescriptor, revision: &str) -> String {
    if source.is_source_backed() {
        stable_id(&["unmanaged", &source.kind, &source.locator, name, revision])
    } else {
        stable_id(&["unmanaged", name, revision])
    }
}

fn managed_entry_key(entry: &InventoryEntry) -> String {
    if entry.source.kind == "centralized" {
        stable_id(&[
            "managed-centralized",
            &entry.name,
            entry.current_revision.as_deref().unwrap_or(""),
        ])
    } else {
        stable_id(&[
            "managed",
            &entry.source.kind,
            &entry.source.locator,
            &entry.name,
            entry.current_revision.as_deref().unwrap_or(""),
        ])
    }
}

fn observation_match_key(package: &SkillPackage) -> String {
    if package.source.is_source_backed() {
        stable_id(&[
            "managed",
            &package.source.kind,
            &package.source.locator,
            &package.declared_name,
            &package.revision,
        ])
    } else {
        stable_id(&[
            "managed-centralized",
            &package.declared_name,
            &package.revision,
        ])
    }
}
