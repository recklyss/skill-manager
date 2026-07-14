use std::path::PathBuf;

use super::identity::SourceDescriptor;
use super::package::SkillPackage;

#[derive(Debug, Clone)]
pub struct SkillObservation {
    pub harness: String,
    pub label: String,
    pub scope: String,
    pub package: SkillPackage,
}

#[derive(Debug, Clone)]
pub struct StorePackageObservation {
    pub package: SkillPackage,
    pub recorded_revision: Option<String>,
    pub recorded_source_ref: Option<String>,
    pub recorded_source_path: Option<String>,
    pub origin_harness: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SkillsHarnessScan {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub installed: bool,
    pub skills: Vec<SkillObservation>,
    pub excluded_skill_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SkillStoreScan {
    pub packages: Vec<StorePackageObservation>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InventorySighting {
    pub kind: String,
    pub harness: Option<String>,
    pub label: String,
    pub scope: Option<String>,
    pub path: Option<PathBuf>,
    pub revision: Option<String>,
    pub source: SourceDescriptor,
    pub detail: String,
}
