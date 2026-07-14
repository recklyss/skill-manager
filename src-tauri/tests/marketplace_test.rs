mod common;

use std::fs;

use common::{seed_named_skill, seed_store_manifest, TestFixture};
use sha1::{Digest, Sha1};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Inline fixture mirroring tests/support/marketplace_payloads.py search results.
fn fixture_search_items(query: &str) -> Vec<serde_json::Value> {
    let needle = query.to_lowercase();
    [
        serde_json::json!({
            "source": "mode-io/skills",
            "skillId": "mode-switch",
            "name": "Mode Switch",
            "installs": 128,
            "description": "Switch between supported skill execution modes."
        }),
        serde_json::json!({
            "source": "vercel-labs/skills",
            "skillId": "trace-scout",
            "name": "Trace Scout",
            "installs": 84,
            "description": "Review traces and highlight suspicious flows."
        }),
    ]
    .into_iter()
    .filter(|item| {
        item["name"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains(&needle)
            || item["description"]
                .as_str()
                .unwrap()
                .to_lowercase()
                .contains(&needle)
    })
    .collect()
}

/// GET /api/marketplace/popular returns a paginated envelope.
#[tokio::test]
async fn skills_popular_returns_page_envelope() {
    let fixture = TestFixture::new();
    let (status, body) = fixture.get("/api/marketplace/popular").await;

    assert_eq!(status, 200);
    assert!(body.get("items").unwrap().is_array());
    assert!(body["hasMore"].is_boolean());
    assert!(body["nextOffset"].is_null() || body["nextOffset"].is_number());
}

/// GET /api/marketplace/search queries skills.sh and returns matching items.
#[tokio::test]
async fn skills_search_returns_remote_catalog_results() {
    let fixture = TestFixture::new();
    let expected = fixture_search_items("mode");
    assert!(
        !expected.is_empty(),
        "fixture should define at least one mode-switch hit"
    );

    let (status, body) = fixture.get("/api/marketplace/search?q=mode").await;
    assert_eq!(status, 200);
    let items = body["items"].as_array().expect("items array");
    assert!(
        !items.is_empty(),
        "skills.sh search should return at least one result for 'mode'"
    );
}

/// GET /api/marketplace/items/{id} resolves github-prefixed item ids.
#[tokio::test]
async fn skill_detail_returns_payload_for_github_item_id() {
    let fixture = TestFixture::new();
    let (status, body) = fixture
        .get("/api/marketplace/items/github%3Amode-io%2Fskills%2Fmode-switch")
        .await;

    assert_eq!(status, 200);
    assert_eq!(
        body["id"].as_str(),
        Some("github:mode-io/skills/mode-switch")
    );
    assert!(body.get("installation").is_some());
}

/// Wiremock fixture server is reachable for isolated search payloads.
#[tokio::test]
async fn wiremock_fixture_serves_search_payload() {
    let server = MockServer::start().await;
    let payload = serde_json::json!({
        "items": fixture_search_items("mode"),
        "nextOffset": null,
        "hasMore": false
    });
    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("q", "mode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload.clone()))
        .mount(&server)
        .await;

    let response = reqwest::get(format!("{}/search?q=mode", server.uri()))
        .await
        .expect("wiremock request")
        .json::<serde_json::Value>()
        .await
        .expect("json");

    assert_eq!(response["items"].as_array().unwrap().len(), 1);
}

/// POST /api/marketplace/install requires installToken in the request body.
#[tokio::test]
async fn install_skill_requires_install_token() {
    let fixture = TestFixture::new();
    let (status, _) = fixture
        .post_json("/api/marketplace/install", serde_json::json!({}))
        .await;
    assert_eq!(status, 422);
}

/// POST /api/marketplace/install rejects unknown install tokens.
#[tokio::test]
async fn install_skill_rejects_unknown_token() {
    let fixture = TestFixture::new();
    let (status, _) = fixture
        .post_json(
            "/api/marketplace/install",
            serde_json::json!({ "installToken": "not-a-real-token" }),
        )
        .await;
    assert_eq!(status, 400);
}

/// Marketplace detail reflects managed skills already present in the shared store.
#[tokio::test]
async fn skill_detail_marks_installed_managed_skill() {
    let fixture = TestFixture::new();
    fs::create_dir_all(&fixture.paths.skills_store_root).expect("store root");
    seed_named_skill(
        &fixture.paths.skills_store_root,
        "mode-switch",
        "mode-switch",
        "Switch between supported skill execution modes.",
    );
    seed_store_manifest(
        &fixture.paths,
        &[serde_json::json!({
            "packageDir": "mode-switch",
            "declaredName": "mode-switch",
            "sourceKind": "github",
            "sourceLocator": "github:mode-io/skills/mode-switch",
            "revision": "abc123",
        })],
    );

    let app = fixture.rebuild_app();
    let response = app
        .oneshot(
            axum::http::Request::get(
                "/api/marketplace/items/github%3Amode-io%2Fskills%2Fmode-switch",
            )
            .body(axum::body::Body::empty())
            .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), 200);
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["installation"]["status"], "installed");
    assert_eq!(
        body["installation"]["installedSkillRef"].as_str(),
        Some("shared:mode-switch")
    );
}

fn seed_mcp_registry_detail(
    fixture: &TestFixture,
    qualified_name: &str,
    detail: &serde_json::Value,
) {
    let digest = format!("{:x}", Sha1::digest(qualified_name.as_bytes()));
    let cache_path = fixture
        .paths
        .marketplace_cache_root
        .join("mcp-registry-detail-v1")
        .join(format!("{digest}.json"));
    let fetched_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    common::write_json(
        &cache_path,
        &serde_json::json!({
            "fetchedAt": fetched_at,
            "payload": detail,
        }),
    );
}

/// POST /api/mcp/servers reinstalls an already-managed marketplace server without conflict.
#[tokio::test]
async fn mcp_marketplace_install_reinstalls_existing_server() {
    let fixture = TestFixture::new();
    let qualified_name = "io.github.example/test-mcp";
    seed_mcp_registry_detail(
        &fixture,
        qualified_name,
        &serde_json::json!({
            "qualifiedName": qualified_name,
            "displayName": "Test MCP",
            "managedName": "test-mcp",
            "connection": { "kind": "stdio" },
        }),
    );
    common::write_json(
        &fixture.paths.mcp_store_manifest,
        &serde_json::json!({
            "version": 6,
            "servers": [{
                "name": "test-mcp",
                "displayName": "Test MCP",
                "source": { "kind": "marketplace", "locator": qualified_name },
                "transport": "stdio",
                "command": "npx",
                "args": ["-y", format!("{qualified_name}@latest")],
            }]
        }),
    );

    let app = fixture.rebuild_app();
    let response = app
        .oneshot(
            axum::http::Request::post("/api/mcp/servers")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({ "qualifiedName": qualified_name }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), 200);
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["ok"], true);
    assert_eq!(body["reinstalled"], true);
    assert_eq!(body["server"]["name"], "test-mcp");
}
