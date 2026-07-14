mod common;

use common::{seed_skill, TestFixture};
use std::fs;

/// GET /api/scan/configs returns ScanConfigListResponse envelope.
#[tokio::test]
async fn list_configs_returns_empty_active_list() {
    let fixture = TestFixture::new();
    let (status, body) = fixture.get("/api/scan/configs").await;

    assert_eq!(status, 200);
    assert!(body.get("configs").unwrap().is_array());
    assert!(body.get("activeId").unwrap().is_null());
}

/// POST /api/scan/configs creates a validated config record.
#[tokio::test]
async fn create_config_persists_validated_record() {
    let fixture = TestFixture::new();
    let (status, body) = fixture
        .post_json(
            "/api/scan/configs",
            serde_json::json!({
                "name": "local",
                "baseUrl": "https://api.openai.com/v1",
                "apiKey": "sk-test",
                "model": "gpt-4o-mini",
                "provider": "openai",
                "apiVersion": "",
                "awsRegion": "",
                "awsProfile": "",
                "awsSessionToken": "",
                "maxTokens": 4096,
                "consensusRuns": 1
            }),
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(body["name"].as_str(), Some("local"));
    assert_eq!(body["provider"].as_str(), Some("openai"));
}

/// GET /api/scan/availability reports whether scan can run.
#[tokio::test]
async fn scan_availability_returns_boolean() {
    let fixture = TestFixture::new();
    let (status, body) = fixture.get("/api/scan/availability").await;
    assert_eq!(status, 200);
    assert!(body.get("available").unwrap().is_boolean());
}

/// GET /api/scan/configs/{id}/secret returns 404 for unknown config ids.
#[tokio::test]
async fn reveal_secret_returns_not_found_for_missing_config() {
    let fixture = TestFixture::new();
    let (status, _) = fixture.get("/api/scan/configs/1/secret").await;
    assert_eq!(status, 404);
}

/// DELETE /api/scan/configs/{id} succeeds even when the config is absent.
#[tokio::test]
async fn delete_config_returns_ok() {
    let fixture = TestFixture::new();
    let (status, body) = fixture.delete("/api/scan/configs/1").await;
    assert_eq!(status, 200);
    assert_eq!(body["ok"].as_bool(), Some(true));
}

/// POST /api/scan/skills/{skill_ref} returns a ScanResultResponse envelope.
#[tokio::test]
async fn scan_skill_returns_result_envelope() {
    let fixture = TestFixture::new();
    fs::create_dir_all(&fixture.paths.skills_store_root).unwrap();
    seed_skill(
        &fixture.paths.skills_store_root,
        "demo-skill",
        "Demo skill",
    );

    // SAFETY: integration tests run sequentially in one process.
    unsafe { std::env::set_var("OPENAI_API_KEY", "sk-test") };

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = fixture
        .rebuild_app()
        .oneshot(
            axum::http::Request::post("/api/scan/skills/shared:demo-skill")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "useLlm": false }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), 200);

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(body["skillName"].as_str(), Some("demo-skill"));
    assert!(body.get("findings").unwrap().is_array());
    assert!(body.get("isSafe").unwrap().is_boolean());
}
