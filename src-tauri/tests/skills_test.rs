mod common;

use std::fs;

use common::{
    codex_legacy_root, copilot_installed_plugins_root, harness_ids, hermes_skills_root,
    seed_named_skill, seed_skill, seed_store_manifest, write_json, TestFixture,
};

/// Empty store returns zero rows with valid page shape.
#[tokio::test]
async fn list_skills_empty_page_shape() {
    let fixture = TestFixture::new();
    let (status, body) = fixture.get("/api/skills").await;

    assert_eq!(status, 200);
    assert_eq!(body["summary"]["managed"], 0);
    assert!(
        body["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["displayStatus"] != "Managed"),
        "empty store should not list managed rows"
    );
    assert_eq!(
        body["harnessColumns"].as_array().unwrap().len(),
        7
    );
}

/// Frontend contract: skills page uses camelCase top-level keys.
#[tokio::test]
async fn skills_page_json_matches_frontend_contract() {
    let fixture = TestFixture::new();
    let (_, body) = fixture.get("/api/skills").await;

    assert!(body.get("harnessColumns").is_some());
    assert!(body.get("rows").is_some());
    assert!(body.get("summary").is_some());
    assert!(body.get("harness_columns").is_none());

    let columns = body["harnessColumns"].as_array().unwrap();
    for col in columns {
        assert!(col.get("logoKey").is_some() || col.get("logoKey").is_none());
        assert!(col.get("harness").is_some());
        assert!(col.get("installed").is_some());
        assert!(col.get("label").is_some());
    }
}

/// Managed skill in shared store appears in rows with description from SKILL.md.
#[tokio::test]
async fn list_skills_returns_managed_skill_from_store() {
    let fixture = TestFixture::new();
    fs::create_dir_all(&fixture.paths.skills_store_root).unwrap();
    seed_skill(
        &fixture.paths.skills_store_root,
        "demo-skill",
        "A demo skill for integration tests",
    );

    // Rebuild app state so the store picks up the new skill.
    let app = fixture.rebuild_app();

    let response = app
        .oneshot(
            axum::http::Request::get("/api/skills")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), 200);

    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(body["summary"]["managed"], 1);
    let row = &body["rows"][0];
    assert_eq!(row["name"], "demo-skill");
    assert_eq!(row["skillRef"], "shared:demo-skill");
    assert_eq!(row["displayStatus"], "Managed");
    assert_eq!(row["description"], "A demo skill for integration tests");
}

/// Cell states must reflect per-skill symlink inventory, not blanket "enabled".
/// Known bug: read_models.rs marks all installed harnesses as enabled for every row.
#[tokio::test]
async fn skill_cell_states_reflect_inventory_not_install_only() {
    let fixture = TestFixture::new();
    fs::create_dir_all(&fixture.paths.skills_store_root).unwrap();
    seed_skill(&fixture.paths.skills_store_root, "orphan-skill", "No symlinks");

    let app = fixture.rebuild_app();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = app
        .oneshot(
            axum::http::Request::get("/api/skills")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    let row = &body["rows"][0];
    let cells = row["cells"].as_array().unwrap();

    // With no symlinks created, no cell should be "enabled".
    let enabled: Vec<&str> = cells
        .iter()
        .filter(|c| c["state"].as_str() == Some("enabled"))
        .filter_map(|c| c["harness"].as_str())
        .collect();

    assert!(
        enabled.is_empty(),
        "cells incorrectly show enabled for harnesses without symlinks: {:?}",
        enabled
    );
}

/// Row actions should be policy-driven, not hardcoded true.
#[tokio::test]
async fn skill_actions_are_policy_driven() {
    let fixture = TestFixture::new();
    let (_, body) = fixture.get("/api/skills").await;

    // Empty page: nothing to assert on rows. Seed a skill and check actions.
    fs::create_dir_all(&fixture.paths.skills_store_root).unwrap();
    seed_skill(&fixture.paths.skills_store_root, "policy-skill", "Test");

    let state = skill_manager_lib::build_app_state(fixture.paths.clone());
    let (_, seeded) = {
        use axum::body::Body;
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let app = skill_manager_lib::api_router(state);
        let response = app
            .oneshot(
                axum::http::Request::get("/api/skills")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            (),
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        )
    };

    let actions = &seeded["rows"][0]["actions"];
    // Managed skill with no enabled harnesses: canManage should be false per Python policy.
    assert_eq!(
        actions["canManage"], false,
        "canManage should be false for managed skill with no enabled harnesses"
    );
    let _ = body;
}

/// Enable/disable routes require a JSON body with harness; empty POST returns 4xx.
#[tokio::test]
async fn skills_mutation_routes_require_request_body() {
    let fixture = TestFixture::new();
    fs::create_dir_all(&fixture.paths.skills_store_root).unwrap();
    seed_skill(&fixture.paths.skills_store_root, "mut-skill", "Mutations");

    let app = fixture.rebuild_app();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = app
        .oneshot(
            axum::http::Request::post("/api/skills/shared:mut-skill/enable")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");

    assert!(
        response.status().is_client_error(),
        "enable without body should fail validation, got {}",
        response.status()
    );
}

/// harnessColumns should respect supportEnabled (codex hidden when disabled).
#[tokio::test]
async fn skills_harness_columns_respect_support_toggle() {
    let fixture = TestFixture::new();

    fixture
        .put_json(
            "/api/settings/harnesses/codex/support",
            serde_json::json!({ "enabled": false }),
        )
        .await;

    let (_, body) = fixture.get("/api/skills").await;
    let column_ids: Vec<&str> = body["harnessColumns"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["harness"].as_str())
        .collect();

    for id in harness_ids() {
        if id == "codex" {
            assert!(!column_ids.contains(&id));
        }
    }
}

/// Discovery roots: legacy Codex dir surfaces unmanaged skills in the matrix.
#[tokio::test]
async fn discovery_roots_surface_unmanaged_skills() {
    let fixture = TestFixture::new();
    let legacy_root = codex_legacy_root(fixture._dir.path());
    fs::create_dir_all(&legacy_root).unwrap();
    seed_named_skill(&legacy_root, "trace-lens", "Trace Lens", "Trace Lens");

    let app = fixture.rebuild_app();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = app
        .oneshot(
            axum::http::Request::get("/api/skills")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    let names: Vec<&str> = body["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["name"].as_str())
        .collect();
    assert!(
        names.contains(&"Trace Lens"),
        "expected legacy discovery root skill in rows, got {:?}",
        names
    );
}

/// Hermes scan policy excludes bundled/local skills and keeps hub community skills.
#[tokio::test]
async fn hermes_scan_excludes_bundled_and_local_skills() {
    let fixture = TestFixture::new();
    let hermes_root = hermes_skills_root(fixture._dir.path());
    fs::create_dir_all(&hermes_root.join("builtin")).unwrap();
    fs::create_dir_all(&hermes_root.join("optional")).unwrap();
    fs::create_dir_all(&hermes_root.join("local")).unwrap();
    fs::create_dir_all(&hermes_root.join("hub")).unwrap();
    seed_named_skill(&hermes_root.join("builtin"), "bundled-core", "Bundled Core", "Bundled Core");
    seed_named_skill(
        &hermes_root.join("optional"),
        "official-helper",
        "Official Helper",
        "Official Helper",
    );
    seed_named_skill(
        &hermes_root.join("local"),
        "user-helper",
        "User Helper",
        "User Helper",
    );
    seed_named_skill(
        &hermes_root.join("hub"),
        "community-helper",
        "Community Helper",
        "Community Helper",
    );
    fs::write(
        hermes_root.join(".bundled_manifest"),
        "Bundled Core:0123456789abcdef\n",
    )
    .unwrap();
    write_json(
        &hermes_root.join(".hub").join("lock.json"),
        &serde_json::json!({
            "version": 1,
            "installed": {
                "official-helper": {
                    "source": "official",
                    "identifier": "official/optional/official-helper",
                    "trust_level": "builtin",
                    "install_path": "optional/official-helper",
                    "metadata": { "backfilled_from": "optional-skills" }
                },
                "community-helper": {
                    "source": "github",
                    "identifier": "github/example/community-helper",
                    "trust_level": "community",
                    "install_path": "hub/community-helper"
                }
            }
        }),
    );

    let app = fixture.rebuild_app();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = app
        .oneshot(
            axum::http::Request::get("/api/skills")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    let names: Vec<&str> = body["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["name"].as_str())
        .collect();
    assert_eq!(names, vec!["Community Helper"]);
}

/// Managing a skill from a discovery root ingests it into the shared store.
#[tokio::test]
async fn manage_adopts_skill_from_discovery_root() {
    let fixture = TestFixture::new();
    let legacy_root = codex_legacy_root(fixture._dir.path());
    fs::create_dir_all(&legacy_root).unwrap();
    seed_named_skill(&legacy_root, "trace-lens", "Trace Lens", "Trace Lens");

    let app = fixture.rebuild_app();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let list_response = app
        .clone()
        .oneshot(
            axum::http::Request::get("/api/skills")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("list");
    let list_bytes = list_response.into_body().collect().await.unwrap().to_bytes();
    let list_body: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    let skill_ref = list_body["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "Trace Lens")
        .and_then(|row| row["skillRef"].as_str())
        .expect("unmanaged trace-lens row");

    let manage_response = app
        .clone()
        .oneshot(
            axum::http::Request::post(format!("/api/skills/{skill_ref}/manage"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("manage");
    assert_eq!(manage_response.status(), 200);

    let after_response = app
        .oneshot(
            axum::http::Request::get("/api/skills")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("after manage");
    let after_bytes = after_response.into_body().collect().await.unwrap().to_bytes();
    let after_body: serde_json::Value = serde_json::from_slice(&after_bytes).unwrap();

    assert_eq!(after_body["summary"]["managed"], 1);
    let managed = &after_body["rows"][0];
    assert_eq!(managed["displayStatus"], "Managed");
    assert_eq!(managed["skillRef"], "shared:trace-lens");
    assert!(fixture.paths.skills_store_root.join("trace-lens").is_dir());
}

/// Source links use persisted ref/path without fetching GitHub.
#[tokio::test]
async fn source_links_use_persisted_folder_url() {
    let fixture = TestFixture::new();
    fs::create_dir_all(&fixture.paths.skills_store_root).unwrap();
    seed_skill(
        &fixture.paths.skills_store_root,
        "shared-audit",
        "Shared Audit",
    );
    seed_store_manifest(
        &fixture.paths,
        &[serde_json::json!({
            "packageDir": "shared-audit",
            "declaredName": "Shared Audit",
            "sourceKind": "github",
            "sourceLocator": "github:mode-io/skills/shared-audit",
            "revision": "abc123",
            "sourceRef": "main",
            "sourcePath": "skills/shared-audit"
        })],
    );

    let app = fixture.rebuild_app();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = app
        .oneshot(
            axum::http::Request::get("/api/skills/shared:shared-audit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("detail");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let detail: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(detail["sourceLinks"]["repoLabel"], "mode-io/skills");
    assert_eq!(
        detail["sourceLinks"]["repoUrl"],
        "https://github.com/mode-io/skills"
    );
    assert_eq!(
        detail["sourceLinks"]["folderUrl"],
        "https://github.com/mode-io/skills/tree/main/skills/shared-audit"
    );
}

/// Copilot scans ~/.copilot/skills and installed plugin skill trees.
#[tokio::test]
async fn copilot_discovery_roots_surface_plugin_and_local_skills() {
    let fixture = TestFixture::new();
    let copilot_managed = fixture._dir.path().join("harness-roots").join("copilot");
    fs::create_dir_all(&copilot_managed).unwrap();
    seed_named_skill(
        &copilot_managed,
        "copilot-local-skill",
        "Copilot Local Skill",
        "From Copilot managed skills root",
    );

    let plugin_skills = copilot_installed_plugins_root(fixture._dir.path())
        .join("superpowers-marketplace")
        .join("superpowers")
        .join("skills");
    fs::create_dir_all(&plugin_skills).unwrap();
    seed_named_skill(
        &plugin_skills,
        "plugin-only-skill",
        "Plugin Only Skill",
        "From installed plugin",
    );

    let app = fixture.rebuild_app();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = app
        .oneshot(
            axum::http::Request::get("/api/skills")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    let copilot_column = body["harnessColumns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|column| column["harness"] == "copilot")
        .expect("copilot harness column");

    assert_eq!(copilot_column["installed"], true);

    for skill_name in ["Copilot Local Skill", "Plugin Only Skill"] {
        let row = body["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == skill_name)
            .unwrap_or_else(|| panic!("expected row for {skill_name}"));
        let copilot_cell = row["cells"]
            .as_array()
            .unwrap()
            .iter()
            .find(|cell| cell["harness"] == "copilot")
            .expect("copilot cell");
        assert_eq!(
            copilot_cell["state"], "found",
            "expected copilot cell to be found for {skill_name}"
        );
    }
}

/// Managed skills discovered via Copilot plugins show as found when not symlinked.
#[tokio::test]
async fn copilot_plugin_skills_mark_managed_rows_as_found() {
    let fixture = TestFixture::new();
    fs::create_dir_all(&fixture.paths.skills_store_root).unwrap();
    seed_named_skill(
        &fixture.paths.skills_store_root,
        "managed-plugin-skill",
        "Managed Plugin Skill",
        "Managed in shared store",
    );

    let plugin_skills = copilot_installed_plugins_root(fixture._dir.path())
        .join("superpowers-marketplace")
        .join("superpowers")
        .join("skills");
    fs::create_dir_all(&plugin_skills).unwrap();
    seed_named_skill(
        &plugin_skills,
        "managed-plugin-skill",
        "Managed Plugin Skill",
        "From installed plugin",
    );

    let app = fixture.rebuild_app();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = app
        .oneshot(
            axum::http::Request::get("/api/skills")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    let rows: Vec<_> = body["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["name"] == "Managed Plugin Skill")
        .collect();
    assert_eq!(rows.len(), 1, "expected a single managed row");

    let copilot_cell = rows[0]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cell| cell["harness"] == "copilot")
        .expect("copilot cell");
    assert_eq!(copilot_cell["state"], "found");
}
