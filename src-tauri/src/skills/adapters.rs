use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_dir as symlink;

use serde_json::Value;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::harness::{
    copilot_settings_skill_directories, BindingProfile, FamilyKey, FileTreeLayout,
    HarnessKernelService,
};
use super::identity::SourceDescriptor;
use super::observations::{SkillObservation, SkillsHarnessScan};
use super::package::{find_plugin_skill_containers, find_skill_roots, parse_skill_package, SkillParseError};

#[derive(Debug, Clone)]
pub struct SkillsHarnessAdapter {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    managed_root: PathBuf,
    discovery_roots: Vec<ResolvedRoot>,
    pub installed: bool,
    layout: FileTreeLayout,
    default_category: String,
}

#[derive(Debug, Clone)]
struct ResolvedRoot {
    scope: String,
    path: PathBuf,
}

#[derive(Debug, Clone)]
struct HermesScanPolicy {
    external_sources: HashMap<String, SourceDescriptor>,
    excluded_skill_names: HashSet<String>,
}

impl SkillsHarnessAdapter {
    pub fn installed(&self) -> bool {
        self.installed
    }

    pub fn scan(&self) -> SkillsHarnessScan {
        let hermes_policy = if self.harness == "hermes" {
            Some(hermes_scan_policy(&self.managed_root))
        } else {
            None
        };
        let excluded_for_scan = hermes_policy
            .as_ref()
            .map(|policy| policy.excluded_skill_names.clone())
            .unwrap_or_default();
        let (observations, mut skipped_skill_names) = scan_skill_roots(
            &self.harness,
            &self.label,
            &self.discovery_roots,
            &excluded_for_scan,
            hermes_policy.as_ref(),
            &self.default_category,
            self.layout,
        );
        if let Some(policy) = &hermes_policy {
            skipped_skill_names.extend(policy.excluded_skill_names.iter().cloned());
        }
        let mut excluded_skill_names: Vec<String> = skipped_skill_names.into_iter().collect();
        excluded_skill_names.sort();

        SkillsHarnessScan {
            harness: self.harness.clone(),
            label: self.label.clone(),
            logo_key: self.logo_key.clone(),
            installed: self.installed,
            skills: observations,
            excluded_skill_names,
        }
    }

    pub fn enable_shared_package(&self, package_path: &Path) -> ApiResult<()> {
        let resolved_target = package_path
            .canonicalize()
            .map_err(|e| ApiError::internal(e.to_string()))?;
        let link = self.binding_path(package_path.file_name().unwrap().to_string_lossy().as_ref());
        if link.is_symlink() {
            let current = link
                .canonicalize()
                .map_err(|e| ApiError::conflict(e.to_string()))?;
            if current == resolved_target {
                return Ok(());
            }
            return Err(ApiError::conflict(format!(
                "symlink already exists but points to {}, not {}",
                current.display(),
                resolved_target.display()
            )));
        }
        if link.is_dir() {
            return self.adopt_local_copy(&link, package_path);
        }
        if link.exists() {
            return Err(ApiError::conflict(format!(
                "non-directory file exists at {}; will not overwrite",
                link.display()
            )));
        }
        if let Some(parent) = link.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ApiError::internal(e.to_string()))?;
        }
        symlink(&resolved_target, &link).map_err(|e| ApiError::internal(e.to_string()))
    }

    pub fn disable_shared_package(&self, package_dir: &str) -> ApiResult<()> {
        let link = self.binding_path(package_dir);
        if !link.exists() && !link.is_symlink() {
            return Ok(());
        }
        if !link.is_symlink() {
            return Err(ApiError::conflict(format!(
                "not a symlink at {}; will not delete real directory",
                link.display()
            )));
        }
        std::fs::remove_file(&link).map_err(|e| ApiError::internal(e.to_string()))
    }

    pub fn adopt_local_copy(&self, existing_dir: &Path, package_path: &Path) -> ApiResult<()> {
        let resolved_target = package_path
            .canonicalize()
            .map_err(|e| ApiError::internal(e.to_string()))?;
        if !existing_dir.exists() && !existing_dir.is_symlink() {
            return Err(ApiError::bad_request(format!(
                "directory does not exist: {}",
                existing_dir.display()
            )));
        }
        if existing_dir.is_symlink() {
            let current = existing_dir
                .canonicalize()
                .map_err(|e| ApiError::conflict(e.to_string()))?;
            if current == resolved_target {
                return Ok(());
            }
            return Err(ApiError::conflict(format!(
                "symlink exists but points to {}, not {}",
                current.display(),
                resolved_target.display()
            )));
        }
        std::fs::remove_dir_all(existing_dir).map_err(|e| ApiError::internal(e.to_string()))?;
        symlink(&resolved_target, existing_dir).map_err(|e| ApiError::internal(e.to_string()))
    }

    pub fn has_binding(&self, package_dir: &str) -> bool {
        let candidate = self.binding_path(package_dir);
        candidate.exists() || candidate.is_symlink()
    }

    pub fn is_symlinked_to_shared(&self, package_dir: &str, package_path: &Path) -> bool {
        let binding = self.binding_path(package_dir);
        if !binding.is_symlink() {
            return false;
        }
        let Ok(current) = binding.canonicalize() else {
            return false;
        };
        let Ok(target) = package_path.canonicalize() else {
            return false;
        };
        current == target
    }

    pub fn prepare_materialize(&self, package_dir: &str, expected_target: &Path) -> ApiResult<()> {
        let existing_link = self.binding_path(package_dir);
        if !existing_link.exists() && !existing_link.is_symlink() {
            return Err(ApiError::bad_request(format!(
                "directory does not exist: {}",
                existing_link.display()
            )));
        }
        if !existing_link.is_symlink() {
            return Err(ApiError::conflict(format!(
                "not a symlink at {}; will not overwrite real directory",
                existing_link.display()
            )));
        }
        let resolved_target = expected_target
            .canonicalize()
            .map_err(|e| ApiError::internal(e.to_string()))?;
        let current = existing_link
            .canonicalize()
            .map_err(|e| ApiError::conflict(e.to_string()))?;
        if current != resolved_target {
            return Err(ApiError::conflict(format!(
                "symlink exists but points to {}, not {}",
                current.display(),
                resolved_target.display()
            )));
        }
        Ok(())
    }

    pub fn materialize_binding(&self, package_dir: &str, source_path: &Path) -> ApiResult<()> {
        let existing_link = self.binding_path(package_dir);
        let resolved_target = source_path
            .canonicalize()
            .map_err(|e| ApiError::internal(e.to_string()))?;
        self.prepare_materialize(package_dir, &resolved_target)?;

        let parent = existing_link.parent().unwrap_or(&existing_link);
        let temp_copy = parent.join(format!(
            ".{}.materialize-{}",
            existing_link.file_name().unwrap().to_string_lossy(),
            Uuid::new_v4().simple()
        ));
        let backup_link = parent.join(format!(
            ".{}.backup-{}",
            existing_link.file_name().unwrap().to_string_lossy(),
            Uuid::new_v4().simple()
        ));

        if let Err(error) = (|| -> Result<(), std::io::Error> {
            copy_dir_all(&resolved_target, &temp_copy)?;
            std::fs::rename(&existing_link, &backup_link)?;
            std::fs::rename(&temp_copy, &existing_link)?;
            Ok(())
        })() {
            if backup_link.exists() && !existing_link.exists() {
                let _ = std::fs::rename(&backup_link, &existing_link);
            }
            if temp_copy.exists() {
                let _ = std::fs::remove_dir_all(&temp_copy);
            }
            return Err(ApiError::internal(format!(
                "unable to restore local copy at {}: {}",
                existing_link.display(),
                error
            )));
        }
        if backup_link.exists() {
            let _ = std::fs::remove_file(&backup_link);
        }
        Ok(())
    }

    pub fn prepare_remove(&self, package_dir: &str) -> ApiResult<()> {
        let link = self.binding_path(package_dir);
        if !link.exists() && !link.is_symlink() {
            return Ok(());
        }
        if !link.is_symlink() {
            return Err(ApiError::conflict(format!(
                "not a symlink at {}; will not delete real directory",
                link.display()
            )));
        }
        Ok(())
    }

    pub fn remove_binding(&self, package_dir: &str) -> ApiResult<()> {
        self.disable_shared_package(package_dir)
    }

    fn binding_path(&self, package_dir: &str) -> PathBuf {
        let default = self.default_binding_path(package_dir);
        if default.exists() || default.is_symlink() {
            return default;
        }
        if self.layout != FileTreeLayout::Categorized || !self.managed_root.is_dir() {
            return default;
        }
        let Ok(entries) = std::fs::read_dir(&self.managed_root) else {
            return default;
        };
        let mut candidates: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir() && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')))
            .map(|category| category.join(package_dir))
            .filter(|p| p.is_symlink())
            .collect();
        candidates.sort();
        candidates.into_iter().next().unwrap_or(default)
    }

    fn default_binding_path(&self, package_dir: &str) -> PathBuf {
        if self.layout == FileTreeLayout::Categorized {
            self.managed_root.join(&self.default_category).join(package_dir)
        } else {
            self.managed_root.join(package_dir)
        }
    }
}

struct SkillRootRef {
    path: PathBuf,
    locator_name: String,
}

fn iter_skill_roots(root: &ResolvedRoot, layout: FileTreeLayout) -> Vec<SkillRootRef> {
    if root.scope == "installed-plugins" {
        return iter_plugin_skill_roots(root);
    }
    let mut results = Vec::new();
    if layout == FileTreeLayout::Flat {
        for skill_root in find_skill_roots(&root.path) {
            results.push(SkillRootRef {
                locator_name: skill_root.file_name().unwrap().to_string_lossy().to_string(),
                path: skill_root,
            });
        }
        return results;
    }
    if !root.path.is_dir() {
        return results;
    }
    let Ok(entries) = std::fs::read_dir(&root.path) else {
        return results;
    };
    let mut categories: Vec<_> = entries.flatten().collect();
    categories.sort_by_key(|e| e.file_name());
    for category in categories {
        let category_path = category.path();
        if !category_path.is_dir() || category_path.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
            continue;
        }
        let category_name = category_path.file_name().unwrap().to_string_lossy().to_string();
        for skill_root in find_skill_roots(&category_path) {
            let skill_name = skill_root.file_name().unwrap().to_string_lossy().to_string();
            results.push(SkillRootRef {
                locator_name: format!("{category_name}/{skill_name}"),
                path: skill_root,
            });
        }
    }
    results
}

fn iter_plugin_skill_roots(root: &ResolvedRoot) -> Vec<SkillRootRef> {
    let mut results = Vec::new();
    for skills_dir in find_plugin_skill_containers(&root.path) {
        let container_relative = skills_dir
            .strip_prefix(&root.path)
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();
        for skill_root in find_skill_roots(&skills_dir) {
            let skill_name = skill_root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            results.push(SkillRootRef {
                locator_name: if container_relative.is_empty() {
                    skill_name
                } else {
                    format!("{container_relative}/{skill_name}")
                },
                path: skill_root,
            });
        }
    }
    results
}

fn scan_skill_roots(
    harness: &str,
    label: &str,
    roots: &[ResolvedRoot],
    excluded_skill_names: &HashSet<String>,
    hermes_policy: Option<&HermesScanPolicy>,
    managed_category: &str,
    layout: FileTreeLayout,
) -> (Vec<SkillObservation>, HashSet<String>) {
    let mut observations = Vec::new();
    let mut skipped_skill_names = HashSet::new();

    for root in roots {
        for skill_root in iter_skill_roots(root, layout) {
            let package_dir = skill_root
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let mut hermes_source = hermes_external_source(
                hermes_policy,
                None,
                &package_dir,
                &skill_root.locator_name,
            );
            let is_skill_manager_binding = hermes_policy.is_some()
                && is_skill_manager_hermes_binding(
                    &skill_root.path,
                    &skill_root.locator_name,
                    managed_category,
                );
            if hermes_policy.is_some() && hermes_source.is_none() && !is_skill_manager_binding {
                record_excluded_skill(
                    &mut skipped_skill_names,
                    None,
                    &package_dir,
                    &skill_root.locator_name,
                );
                continue;
            }

            let default_source = SourceDescriptor::new(
                "harness-local",
                format!("{harness}:{}:{}", root.scope, skill_root.locator_name),
            );
            let source = hermes_source.clone().unwrap_or(default_source);
            let mut package = match parse_skill_package(&skill_root.path, source) {
                Ok(p) => p,
                Err(SkillParseError(_)) => continue,
            };

            if hermes_policy.is_some() {
                if !is_skill_manager_binding
                    && is_excluded_skill(
                        Some(&package.declared_name),
                        &package_dir,
                        &skill_root.locator_name,
                        excluded_skill_names,
                    )
                {
                    record_excluded_skill(
                        &mut skipped_skill_names,
                        Some(&package.declared_name),
                        &package_dir,
                        &skill_root.locator_name,
                    );
                    continue;
                }

                hermes_source = hermes_source.or_else(|| {
                    hermes_external_source(
                        hermes_policy,
                        Some(&package.declared_name),
                        &package_dir,
                        &skill_root.locator_name,
                    )
                });
                if let Some(source) = hermes_source {
                    if package.source.kind == "harness-local" {
                        package = match parse_skill_package(&skill_root.path, source) {
                            Ok(p) => p,
                            Err(SkillParseError(_)) => continue,
                        };
                    }
                }
            }

            observations.push(SkillObservation {
                harness: harness.to_string(),
                label: label.to_string(),
                scope: root.scope.clone(),
                package,
            });
        }
    }

    (observations, skipped_skill_names)
}

fn hermes_scan_policy(skills_root: &Path) -> HermesScanPolicy {
    let mut excluded_names = HashSet::new();
    let mut external_sources = HashMap::new();
    read_hermes_bundled_manifest(&skills_root.join(".bundled_manifest"), &mut excluded_names);
    read_hermes_hub_lock(
        &skills_root.join(".hub").join("lock.json"),
        &mut excluded_names,
        &mut external_sources,
    );
    HermesScanPolicy {
        external_sources,
        excluded_skill_names: excluded_names.into_iter().filter(|name| !name.is_empty()).collect(),
    }
}

fn read_hermes_bundled_manifest(path: &Path, names: &mut HashSet<String>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        let name = line.trim().split(':').next().unwrap_or("").trim();
        if !name.is_empty() {
            names.insert(name.to_string());
        }
    }
}

fn read_hermes_hub_lock(
    path: &Path,
    excluded_names: &mut HashSet<String>,
    external_sources: &mut HashMap<String, SourceDescriptor>,
) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<Value>(&content) else {
        return;
    };
    let Some(installed) = payload.get("installed").and_then(Value::as_object) else {
        return;
    };
    for (lock_name, raw_entry) in installed {
        let Some(raw_entry) = raw_entry.as_object() else {
            continue;
        };
        let install_path = raw_entry.get("install_path").and_then(Value::as_str);
        let names = hermes_lock_names(lock_name, install_path);
        if is_hermes_official_lock_entry(raw_entry) {
            excluded_names.extend(names);
            continue;
        }
        let source = raw_entry
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let source = if source.is_empty() { "hermes-hub" } else { source };
        let identifier = raw_entry
            .get("identifier")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let identifier = if identifier.is_empty() {
            lock_name.as_str()
        } else {
            identifier
        };
        let descriptor = SourceDescriptor::new(source, identifier);
        for name in names {
            external_sources.insert(name, descriptor.clone());
        }
    }
}

fn is_hermes_official_lock_entry(raw_entry: &serde_json::Map<String, Value>) -> bool {
    let source = raw_entry
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("");
    let identifier = raw_entry
        .get("identifier")
        .and_then(Value::as_str)
        .unwrap_or("");
    let trust_level = raw_entry
        .get("trust_level")
        .and_then(Value::as_str)
        .unwrap_or("");
    let metadata = raw_entry.get("metadata").and_then(Value::as_object);
    source == "official"
        || identifier.starts_with("official/")
        || trust_level == "builtin"
        || metadata
            .and_then(|m| m.get("backfilled_from"))
            .and_then(Value::as_str)
            == Some("optional-skills")
}

fn hermes_lock_names(lock_name: &str, install_path: Option<&str>) -> HashSet<String> {
    let mut names = HashSet::new();
    if !lock_name.is_empty() {
        names.insert(lock_name.to_string());
    }
    if let Some(install_path) = install_path.filter(|value| !value.is_empty()) {
        names.insert(install_path.to_string());
        if let Some(leaf) = Path::new(install_path).file_name().and_then(|n| n.to_str()) {
            names.insert(leaf.to_string());
        }
    }
    names
}

fn hermes_external_source(
    hermes_policy: Option<&HermesScanPolicy>,
    package_name: Option<&str>,
    package_dir: &str,
    locator_name: &str,
) -> Option<SourceDescriptor> {
    let policy = hermes_policy?;
    let locator_leaf = Path::new(locator_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(locator_name);
    for candidate in [package_name, Some(locator_name), Some(package_dir), Some(locator_leaf)] {
        if let Some(name) = candidate.filter(|value| !value.is_empty()) {
            if let Some(source) = policy.external_sources.get(name) {
                return Some(source.clone());
            }
        }
    }
    None
}

fn is_skill_manager_hermes_binding(
    skill_root: &Path,
    locator_name: &str,
    managed_category: &str,
) -> bool {
    skill_root.is_symlink() && locator_name.starts_with(&format!("{managed_category}/"))
}

fn record_excluded_skill(
    names: &mut HashSet<String>,
    package_name: Option<&str>,
    package_dir: &str,
    locator_name: &str,
) {
    let locator_leaf = Path::new(locator_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(locator_name);
    for candidate in [package_name, Some(package_dir), Some(locator_name), Some(locator_leaf)] {
        if let Some(name) = candidate.filter(|value| !value.is_empty()) {
            names.insert(name.to_string());
        }
    }
}

fn is_excluded_skill(
    package_name: Option<&str>,
    package_dir: &str,
    locator_name: &str,
    excluded_skill_names: &HashSet<String>,
) -> bool {
    if excluded_skill_names.is_empty() {
        return false;
    }
    let locator_leaf = Path::new(locator_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(locator_name);
    [package_name, Some(package_dir), Some(locator_name), Some(locator_leaf)]
        .into_iter()
        .flatten()
        .filter(|value| !value.is_empty())
        .any(|candidate| excluded_skill_names.contains(candidate))
}

pub fn build_skills_adapters(kernel: &HarnessKernelService) -> Vec<SkillsHarnessAdapter> {
    let statuses: HashMap<_, _> = kernel
        .harness_statuses()
        .into_iter()
        .map(|s| (s.harness.clone(), s.installed))
        .collect();

    kernel
        .bindings_for_family(FamilyKey::Skills)
        .into_iter()
        .filter_map(|binding| {
            let BindingProfile::FileTree(profile) = binding.profile else {
                return None;
            };
            let managed_root = profile.resolve_managed_root(&kernel.context);
            let mut resolved_roots = vec![ResolvedRoot {
                scope: "canonical".into(),
                path: managed_root.clone(),
            }];
            for root in profile.discovery_roots {
                resolved_roots.push(ResolvedRoot {
                    scope: root.scope.to_string(),
                    path: (root.path_resolver)(&kernel.context),
                });
            }
            if binding.definition.harness == "copilot" {
                for (index, path) in copilot_settings_skill_directories(&kernel.context)
                    .into_iter()
                    .enumerate()
                {
                    resolved_roots.push(ResolvedRoot {
                        scope: format!("skill-directories-{index}"),
                        path,
                    });
                }
            }
            let installed = statuses
                .get(binding.definition.harness)
                .copied()
                .unwrap_or(false);
            Some(SkillsHarnessAdapter {
                harness: binding.definition.harness.to_string(),
                label: binding.definition.label.to_string(),
                logo_key: binding.definition.logo_key.map(str::to_string),
                managed_root: managed_root.clone(),
                discovery_roots: dedupe_roots(resolved_roots),
                installed,
                layout: profile.layout,
                default_category: profile
                    .default_category
                    .unwrap_or("skill-manager")
                    .to_string(),
            })
        })
        .collect()
}

fn dedupe_roots(roots: Vec<ResolvedRoot>) -> Vec<ResolvedRoot> {
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    for root in roots {
        let path = root.path.canonicalize().unwrap_or(root.path);
        if seen.insert(path.clone()) {
            selected.push(ResolvedRoot { path, ..root });
        }
    }
    selected
}

pub fn scan_all_adapters(adapters: &[SkillsHarnessAdapter]) -> Vec<SkillsHarnessScan> {
    adapters.iter().map(|adapter| adapter.scan()).collect()
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}
