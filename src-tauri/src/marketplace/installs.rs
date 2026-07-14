use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::skills::package::parse_skill_package;
use crate::skills::read_models::SkillsReadModelService;
use crate::skills::source_fetch::{FetchedSourcePackage, SourceFetchService};

use super::installation::find_managed_package_dir;
use super::tokens::resolve_install_token;

pub fn install_skill(
    install_token: &str,
    read_models: &SkillsReadModelService,
    source_fetcher: &SourceFetchService,
) -> ApiResult<Value> {
    let token = install_token.trim();
    if token.is_empty() {
        return Err(ApiError::bad_request("installToken is required"));
    }

    let (source_kind, source_locator) = resolve_install_token(token)
        .ok_or_else(|| ApiError::bad_request("unknown marketplace install token"))?;

    let temp_dir = tempfile::tempdir().map_err(|e| ApiError::internal(e.to_string()))?;
    let fetched = source_fetcher.fetch_package(&source_kind, &source_locator, temp_dir.path())?;

    install_fetched_package(read_models, &fetched, &source_kind, &source_locator)
}

pub(crate) fn install_fetched_package(
    read_models: &SkillsReadModelService,
    fetched: &FetchedSourcePackage,
    source_kind: &str,
    source_locator: &str,
) -> ApiResult<Value> {
    let package = parse_skill_package(
        &fetched.package_path,
        crate::skills::identity::SourceDescriptor::new(source_kind, source_locator),
    )
    .map_err(|e| ApiError::bad_request(e.to_string()))?;

    if let Some(package_dir) = find_managed_package_dir(read_models, source_kind, source_locator) {
        read_models
            .store
            .update(
                &package_dir,
                &fetched.package_path,
                fetched.source_ref.as_deref(),
                fetched.source_path.as_deref(),
            )
            .map_err(ApiError::conflict)?;
        read_models.invalidate();
        return Ok(json!({ "ok": true, "reinstalled": true }));
    }

    read_models
        .store
        .ingest(
            &fetched.package_path,
            &package.declared_name,
            source_kind,
            source_locator,
            fetched.source_ref.as_deref(),
            fetched.source_path.as_deref(),
            None,
        )
        .map_err(ApiError::conflict)?;

    read_models.invalidate();
    Ok(json!({ "ok": true, "reinstalled": false }))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::harness::{HarnessKernelService, HarnessSupportStore};
    use crate::paths::AppPaths;
    use crate::skills::read_models::SkillsReadModelService;
    use crate::skills::source_fetch::FetchedSourcePackage;
    use crate::skills::store::SkillStore;

    use super::*;

    fn read_models_with_seeded_skill(
        root: &std::path::Path,
        package_dir: &str,
        source_locator: &str,
    ) -> (SkillsReadModelService, PathBuf) {
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
            "---\nname: mode-switch\ndescription: Original.\n---\n\n# Mode Switch\n",
        )
        .expect("SKILL.md");
        fs::write(
            &paths.skills_store_manifest,
            serde_json::to_string_pretty(&serde_json::json!({
                "entries": [{
                    "packageDir": package_dir,
                    "declaredName": "mode-switch",
                    "sourceKind": "github",
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
        (SkillsReadModelService::new(store, kernel), skill_dir)
    }

    #[test]
    fn reinstall_existing_managed_skill_updates_without_conflict() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source_locator = "github:mode-io/skills/mode-switch";
        let (read_models, existing_dir) =
            read_models_with_seeded_skill(dir.path(), "mode-switch", source_locator);

        let updated_dir = dir.path().join("updated-package");
        fs::create_dir_all(&updated_dir).expect("updated dir");
        fs::write(
            updated_dir.join("SKILL.md"),
            "---\nname: mode-switch\ndescription: Updated copy.\n---\n\n# Mode Switch\n",
        )
        .expect("updated SKILL.md");

        let fetched = FetchedSourcePackage {
            package_path: updated_dir,
            source_ref: Some("main".to_string()),
            source_path: Some("skills/mode-switch".to_string()),
        };

        let response = install_fetched_package(
            &read_models,
            &fetched,
            "github",
            source_locator,
        )
        .expect("reinstall");

        assert_eq!(response["ok"], true);
        assert_eq!(response["reinstalled"], true);
        let on_disk = fs::read_to_string(existing_dir.join("SKILL.md")).expect("SKILL.md");
        assert!(on_disk.contains("Updated copy."));
    }
}
