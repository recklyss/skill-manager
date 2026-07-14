mod common;

use common::{find_harness, harness_ids, TestFixture};

/// GET /api/settings returns storage paths rooted in the test data dir.
#[tokio::test]
async fn get_settings_storage_paths_match_fixture() {
    let fixture = TestFixture::new();
    let (status, body) = fixture.get("/api/settings").await;

    assert_eq!(status, 200);
    let storage = &body["storage"];
    assert_eq!(
        storage["skillsStorePath"].as_str().unwrap(),
        fixture.paths.skills_store_root.to_string_lossy()
    );
    assert_eq!(
        storage["dataDir"].as_str().unwrap(),
        fixture.paths.data_dir.to_string_lossy()
    );
    assert_eq!(
        storage["settingsPath"].as_str().unwrap(),
        fixture.paths.settings_path.to_string_lossy()
    );
    assert!(storage["platform"].is_string());
}

/// Settings must expose all six catalog harnesses.
#[tokio::test]
async fn get_settings_lists_six_harnesses() {
    let fixture = TestFixture::new();
    let (_, body) = fixture.get("/api/settings").await;

    let harnesses = body["harnesses"].as_array().expect("harnesses array");
    assert_eq!(harnesses.len(), 6);
    for id in harness_ids() {
        find_harness(harnesses, id);
    }
}

/// Frontend contract: harness entries use camelCase and include supportEnabled.
/// See frontend/src/api/generated.ts SettingsHarnessResponse.
#[tokio::test]
async fn settings_harness_json_matches_frontend_contract() {
    let fixture = TestFixture::new();
    let (_, body) = fixture.get("/api/settings").await;
    let harnesses = body["harnesses"].as_array().unwrap();

    for harness in harnesses {
        assert!(harness.get("logoKey").is_some(), "missing logoKey (got snake_case?)");
        assert!(
            harness.get("managedLocation").is_some(),
            "missing managedLocation (got snake_case?)"
        );
        assert!(
            harness.get("supportEnabled").is_some(),
            "missing supportEnabled field"
        );
        assert!(
            harness.get("logo_key").is_none(),
            "snake_case logo_key should not appear in API response"
        );
        assert!(
            harness.get("managed_location").is_none(),
            "snake_case managed_location should not appear in API response"
        );
    }
}

/// supportEnabled defaults to true for all harnesses (Python parity).
#[tokio::test]
async fn settings_support_enabled_defaults_true() {
    let fixture = TestFixture::new();
    let (_, body) = fixture.get("/api/settings").await;
    let harnesses = body["harnesses"].as_array().unwrap();

    for harness in harnesses {
        assert_eq!(
            harness["supportEnabled"].as_bool(),
            Some(true),
            "supportEnabled should default to true"
        );
    }
}

/// PUT /api/settings/harnesses/{harness}/support toggles supportEnabled.
#[tokio::test]
async fn put_harness_support_toggle_persists() {
    let fixture = TestFixture::new();

    let (put_status, _) = fixture
        .put_json(
            "/api/settings/harnesses/codex/support",
            serde_json::json!({ "enabled": false }),
        )
        .await;
    assert_eq!(
        put_status,
        200,
        "PUT /api/settings/harnesses/{{harness}}/support not implemented"
    );

    let (_, body) = fixture.get("/api/settings").await;
    let codex = find_harness(body["harnesses"].as_array().unwrap(), "codex");
    assert_eq!(codex["supportEnabled"], false);

    // Disabled harness should disappear from skills harnessColumns (Python parity).
    let (_, skills) = fixture.get("/api/skills").await;
    let columns: Vec<&str> = skills["harnessColumns"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["harness"].as_str())
        .collect();
    assert!(!columns.contains(&"codex"));
}
