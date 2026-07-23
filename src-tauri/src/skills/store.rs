use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::fsutil::copy_dir_all;
use super::package::{find_skill_roots, fingerprint_package, parse_skill_package, SkillParseError};
use super::identity::SourceDescriptor;
use super::observations::{SkillStoreScan, StorePackageObservation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStoreEntry {
    #[serde(rename = "packageDir")]
    pub package_dir: String,
    #[serde(rename = "declaredName")]
    pub declared_name: String,
    #[serde(rename = "sourceKind")]
    pub source_kind: String,
    #[serde(rename = "sourceLocator")]
    pub source_locator: String,
    pub revision: String,
    #[serde(rename = "sourceRef", skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(rename = "sourcePath", skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(rename = "originHarness", skip_serializing_if = "Option::is_none")]
    pub origin_harness: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillStoreManifest {
    pub entries: Vec<SkillStoreEntry>,
}

pub fn load_skill_store_manifest(path: &Path) -> SkillStoreManifest {
    if !path.is_file() {
        return SkillStoreManifest::default();
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return SkillStoreManifest::default(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn write_skill_store_manifest(path: &Path, manifest: &SkillStoreManifest) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(manifest)? + "\n";
    fs::write(path, payload)
}

/// Filesystem-based skill store backed by `data_dir/shared/` and `manifest.json`.
#[derive(Clone)]
pub struct SkillStore {
    root: std::path::PathBuf,
    manifest_path: std::path::PathBuf,
}

impl SkillStore {
    pub fn new(root: std::path::PathBuf, manifest_path: std::path::PathBuf) -> Self {
        Self { root, manifest_path }
    }

    pub fn from_paths(paths: &crate::paths::AppPaths) -> Self {
        Self::new(
            paths.skills_store_root.clone(),
            paths.skills_store_manifest.clone(),
        )
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)
    }

    pub fn scan(&self) -> SkillStoreScan {
        let manifest = load_skill_store_manifest(&self.manifest_path);
        let manifest_index: std::collections::HashMap<_, _> = manifest
            .entries
            .iter()
            .map(|e| (e.package_dir.clone(), e))
            .collect();

        let mut packages = Vec::new();
        for path in find_skill_roots(&self.root) {
            let entry = manifest_index.get(&path.file_name().unwrap().to_string_lossy().to_string());
            let default_source = if let Some(e) = entry {
                SourceDescriptor::new(e.source_kind.clone(), e.source_locator.clone())
            } else {
                SourceDescriptor::new(
                    "shared-store",
                    format!("shared-store:{}", path.file_name().unwrap().to_string_lossy()),
                )
            };
            let package = match parse_skill_package(&path, default_source) {
                Ok(p) => p,
                Err(SkillParseError(_)) => continue,
            };
            packages.push(StorePackageObservation {
                package,
                recorded_revision: entry.map(|e| e.revision.clone()),
                recorded_source_ref: entry.and_then(|e| e.source_ref.clone()),
                recorded_source_path: entry.and_then(|e| e.source_path.clone()),
                origin_harness: entry.and_then(|e| e.origin_harness.clone()),
            });
        }

        SkillStoreScan {
            packages,
            issues: self.check_integrity(),
        }
    }

    pub fn ingest(
        &self,
        source_path: &Path,
        declared_name: &str,
        source_kind: &str,
        source_locator: &str,
        source_ref: Option<&str>,
        source_path_hint: Option<&str>,
        origin_harness: Option<&str>,
    ) -> Result<std::path::PathBuf, String> {
        self.init().map_err(|e| e.to_string())?;
        let dest = self.root.join(
            source_path
                .file_name()
                .ok_or_else(|| "invalid source path".to_string())?
                .to_string_lossy()
                .as_ref(),
        );
        if dest.exists() {
            return Err(format!(
                "package directory already exists in store: {}",
                dest.file_name().unwrap().to_string_lossy()
            ));
        }
        copy_dir_all(source_path, &dest).map_err(|e| e.to_string())?;
        let (fingerprint, _) = fingerprint_package(&dest).map_err(|e| e.to_string())?;
        let mut manifest = load_skill_store_manifest(&self.manifest_path);
        manifest.entries.push(SkillStoreEntry {
            package_dir: dest.file_name().unwrap().to_string_lossy().to_string(),
            declared_name: declared_name.to_string(),
            source_kind: source_kind.to_string(),
            source_locator: source_locator.to_string(),
            revision: fingerprint,
            source_ref: source_ref.map(str::to_string),
            source_path: source_path_hint.map(str::to_string),
            origin_harness: origin_harness.map(str::to_string),
        });
        write_skill_store_manifest(&self.manifest_path, &manifest).map_err(|e| e.to_string())?;
        Ok(dest)
    }

    pub fn update(
        &self,
        package_dir: &str,
        source_path: &Path,
        source_ref: Option<&str>,
        source_path_hint: Option<&str>,
    ) -> Result<(std::path::PathBuf, bool), String> {
        let dest = self.root.join(package_dir);
        if !dest.is_dir() {
            return Err(format!("package not in store: {package_dir}"));
        }
        let (new_fp, _) = fingerprint_package(source_path).map_err(|e| e.to_string())?;
        let (old_fp, _) = fingerprint_package(&dest).map_err(|e| e.to_string())?;
        if new_fp == old_fp {
            return Ok((dest, false));
        }
        fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
        copy_dir_all(source_path, &dest).map_err(|e| e.to_string())?;
        let mut manifest = load_skill_store_manifest(&self.manifest_path);
        for entry in &mut manifest.entries {
            if entry.package_dir == package_dir {
                entry.revision = new_fp;
                if let Some(source_ref) = source_ref {
                    entry.source_ref = Some(source_ref.to_string());
                }
                if let Some(source_path_hint) = source_path_hint {
                    entry.source_path = Some(source_path_hint.to_string());
                }
                break;
            }
        }
        write_skill_store_manifest(&self.manifest_path, &manifest).map_err(|e| e.to_string())?;
        Ok((dest, true))
    }

    pub fn delete(&self, package_dir: &str) -> Result<(), String> {
        self.ensure_deletable(package_dir)?;
        let dest = self.root.join(package_dir);
        fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
        let mut manifest = load_skill_store_manifest(&self.manifest_path);
        manifest.entries.retain(|e| e.package_dir != package_dir);
        write_skill_store_manifest(&self.manifest_path, &manifest).map_err(|e| e.to_string())
    }

    pub fn ensure_deletable(&self, package_dir: &str) -> Result<(), String> {
        let dest = self.root.join(package_dir);
        if !dest.is_dir() {
            return Err(format!("package not in store: {package_dir}"));
        }
        let manifest = load_skill_store_manifest(&self.manifest_path);
        if !manifest.entries.iter().any(|e| e.package_dir == package_dir) {
            return Err(format!("package missing from manifest: {package_dir}"));
        }
        Ok(())
    }

    fn check_integrity(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if !self.root.exists() {
            return issues;
        }
        let Ok(entries) = fs::read_dir(&self.root) else {
            return issues;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.join("SKILL.md").is_file() {
                issues.push(format!(
                    "Shared package is missing SKILL.md: {}",
                    path.file_name().unwrap().to_string_lossy()
                ));
            }
        }
        issues
    }
}


