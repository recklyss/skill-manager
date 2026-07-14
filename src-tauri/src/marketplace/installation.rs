use serde_json::{json, Value};

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
        .find(|entry| {
            entry.kind == "managed"
                && entry.source.kind == source_kind
                && entry.source.locator == source_locator
        })
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
        .find(|entry| {
            entry.kind == "managed"
                && entry.source.kind == source_kind
                && entry.source.locator == source_locator
        })
        .and_then(|entry| entry.package_dir.clone())
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

    fn read_models_with_seeded_skill(
        root: &std::path::Path,
    ) -> SkillsReadModelService {
        let paths = AppPaths::from_dirs(
            root.join("config"),
            root.join("data"),
            root.join("state"),
        );
        fs::create_dir_all(&paths.skills_store_root).expect("store root");
        let skill_dir = paths.skills_store_root.join("mode-switch");
        fs::create_dir_all(&skill_dir).expect("skill dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: mode-switch\ndescription: Switch modes.\n---\n\n# Mode Switch\n",
        )
        .expect("SKILL.md");
        fs::write(
            &paths.skills_store_manifest,
            serde_json::to_string_pretty(&serde_json::json!({
                "entries": [{
                    "packageDir": "mode-switch",
                    "declaredName": "mode-switch",
                    "sourceKind": "github",
                    "sourceLocator": "github:mode-io/skills/mode-switch",
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
        let read_models = read_models_with_seeded_skill(dir.path());
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
    fn installation_state_defaults_to_installable_when_not_managed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let read_models = read_models_with_seeded_skill(dir.path());
        let state = installation_state(
            &read_models,
            "github",
            "github:other/repo/other-skill",
        );
        assert_eq!(state["status"], "installable");
        assert!(state["installedSkillRef"].is_null());
    }
}
