use serde_json::{json, Value};

use crate::skills::inventory::InventoryEntry;
use crate::skills::read_models::SkillsReadModelService;

use super::tokens::resolve_install_token;

pub fn installation_state(
    read_models: &SkillsReadModelService,
    source_kind: &str,
    source_locator: &str,
) -> Value {
    if let Some(skill_ref) = installed_skill_ref(read_models, source_kind, source_locator) {
        json!({
            "status": "installed",
            "installedSkillRef": skill_ref,
        })
    } else {
        json!({
            "status": "installable",
            "installedSkillRef": Value::Null,
        })
    }
}

pub fn installed_skill_ref(
    read_models: &SkillsReadModelService,
    source_kind: &str,
    source_locator: &str,
) -> Option<String> {
    read_models
        .inventory()
        .entries
        .iter()
        .find(|entry| source_matches(entry, source_kind, source_locator))
        .map(|entry| entry.skill_ref.clone())
}

pub fn find_managed_package_dir(
    read_models: &SkillsReadModelService,
    source_kind: &str,
    source_locator: &str,
) -> Option<String> {
    read_models
        .inventory()
        .entries
        .iter()
        .find(|entry| source_matches(entry, source_kind, source_locator))
        .and_then(|entry| entry.package_dir.clone())
}

fn source_matches(entry: &InventoryEntry, source_kind: &str, source_locator: &str) -> bool {
    if entry.kind != "managed" {
        return false;
    }

    if entry.source.kind == source_kind && locators_equivalent(&entry.source.locator, source_locator) {
        return true;
    }

    if source_kind != "github" {
        return false;
    }

    let Some(marketplace_skill_id) = github_marketplace_skill_id(source_locator) else {
        return false;
    };

    if entry
        .package_dir
        .as_deref()
        .is_some_and(|package_dir| identifiers_equivalent(package_dir, marketplace_skill_id))
    {
        return true;
    }

    if entry.source.kind == "centralized" {
        let centralized_name = entry
            .source
            .locator
            .strip_prefix("centralized:")
            .unwrap_or(&entry.source.locator);
        if identifiers_equivalent(centralized_name, marketplace_skill_id)
            || identifiers_equivalent(&entry.name, marketplace_skill_id)
        {
            return true;
        }
    }

    if entry.source.kind == "github" {
        return github_marketplace_skill_id(&entry.source.locator)
            .is_some_and(|installed_skill_id| {
                identifiers_equivalent(installed_skill_id, marketplace_skill_id)
            });
    }

    false
}

fn github_marketplace_skill_id(source_locator: &str) -> Option<&str> {
    let stripped = source_locator.strip_prefix("github:").unwrap_or(source_locator);
    let segments: Vec<&str> = stripped.split('/').filter(|segment| !segment.is_empty()).collect();
    if segments.len() < 3 {
        return None;
    }
    segments.last().copied()
}

fn locators_equivalent(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    if left.starts_with("github:") && right.starts_with("github:") {
        return left.eq_ignore_ascii_case(right);
    }
    false
}

fn identifiers_equivalent(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

pub fn enrich_skill_marketplace_payload(
    read_models: &SkillsReadModelService,
    mut payload: Value,
) -> Value {
    if let Some(items) = payload.get_mut("items").and_then(|v| v.as_array_mut()) {
        for item in items.iter_mut() {
            enrich_skill_item(read_models, item);
        }
    }
    payload
}

pub fn enrich_skill_item(read_models: &SkillsReadModelService, item: &mut Value) {
    let Some(token) = item.get("installToken").and_then(|v| v.as_str()) else {
        return;
    };
    let Some((source_kind, source_locator)) = resolve_install_token(token) else {
        return;
    };
    item["installation"] = installation_state(read_models, &source_kind, &source_locator);
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::harness::{HarnessKernelService, HarnessSupportStore};
    use crate::paths::AppPaths;
    use crate::skills::read_models::SkillsReadModelService;
    use crate::skills::store::SkillStore;

    use super::*;

    fn read_models_with_manifest(
        root: &std::path::Path,
        package_dir: &str,
        declared_name: &str,
        source_kind: &str,
        source_locator: &str,
    ) -> SkillsReadModelService {
        let paths = AppPaths::from_dirs(
            root.join("config"),
            root.join("data"),
            root.join("state"),
        );
        fs::create_dir_all(&paths.skills_store_root).expect("store root");
        let skill_dir = paths.skills_store_root.join(package_dir);
        fs::create_dir_all(&skill_dir).expect("skill dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {declared_name}\ndescription: Test skill.\n---\n\n# {declared_name}\n"),
        )
        .expect("SKILL.md");
        fs::write(
            &paths.skills_store_manifest,
            serde_json::to_string_pretty(&serde_json::json!({
                "entries": [{
                    "packageDir": package_dir,
                    "declaredName": declared_name,
                    "sourceKind": source_kind,
                    "sourceLocator": source_locator,
                    "revision": "abc123",
                }]
            }))
            .unwrap(),
        )
        .expect("manifest");
        let support_store = HarnessSupportStore::new(paths.settings_path.clone());
        let kernel = HarnessKernelService::from_environment(None, support_store);
        let store = SkillStore::from_paths(&paths);
        store.init().expect("store init");
        SkillsReadModelService::new(store, kernel)
    }

    #[test]
    fn installation_state_marks_matching_managed_skill_as_installed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let read_models = read_models_with_manifest(
            dir.path(),
            "mode-switch",
            "mode-switch",
            "github",
            "github:mode-io/skills/mode-switch",
        );
        let state = installation_state(
            &read_models,
            "github",
            "github:mode-io/skills/mode-switch",
        );
        assert_eq!(state["status"], "installed");
        assert_eq!(
            state["installedSkillRef"].as_str(),
            Some("shared:mode-switch")
        );
    }

    #[test]
    fn installation_state_matches_github_locator_case_insensitively() {
        let dir = tempfile::tempdir().expect("tempdir");
        let read_models = read_models_with_manifest(
            dir.path(),
            "ponytail",
            "ponytail",
            "github",
            "github:dietrichgebert/ponytail/ponytail",
        );
        let state = installation_state(
            &read_models,
            "github",
            "github:DietrichGebert/ponytail/ponytail",
        );
        assert_eq!(state["status"], "installed");
        assert_eq!(
            state["installedSkillRef"].as_str(),
            Some("shared:ponytail")
        );
    }

    #[test]
    fn installation_state_matches_centralized_skill_by_marketplace_skill_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let read_models = read_models_with_manifest(
            dir.path(),
            "find-skills",
            "find-skills",
            "centralized",
            "centralized:find-skills",
        );
        let state = installation_state(
            &read_models,
            "github",
            "github:vercel-labs/skills/find-skills",
        );
        assert_eq!(state["status"], "installed");
        assert_eq!(
            state["installedSkillRef"].as_str(),
            Some("shared:find-skills")
        );
    }

    #[test]
    fn installation_state_defaults_to_installable_when_not_managed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let read_models = read_models_with_manifest(
            dir.path(),
            "mode-switch",
            "mode-switch",
            "github",
            "github:mode-io/skills/mode-switch",
        );
        let state = installation_state(
            &read_models,
            "github",
            "github:other/repo/other-skill",
        );
        assert_eq!(state["status"], "installable");
        assert!(state["installedSkillRef"].is_null());
    }
}
