mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestFixture;
use http_body_util::BodyExt;
use tower::ServiceExt;

fn test_home(fixture: &TestFixture) -> std::path::PathBuf {
    fixture._dir.path().join("home")
}

fn write_cursor_mcp(fixture: &TestFixture, servers: serde_json::Value) {
    let path = test_home(fixture).join(".cursor").join("mcp.json");
    common::write_json(&path, &serde_json::json!({ "mcpServers": servers }));
}

fn write_claude_mcp(fixture: &TestFixture, servers: serde_json::Value) {
    let path = test_home(fixture).join(".claude.json");
    common::write_json(&path, &serde_json::json!({ "mcpServers": servers }));
}

async fn mcp_get(fixture: &TestFixture, path: &str) -> (StatusCode, serde_json::Value) {
    let response = fixture
        .rebuild_app()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .expect("request");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, json)
}

async fn mcp_post_json(
    fixture: &TestFixture,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let response = fixture
        .rebuild_app()
        .oneshot(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .expect("request");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, json)
}

#[tokio::test]
async fn list_servers_returns_inventory_envelope() {
    let fixture = TestFixture::new();
    let (status, body) = mcp_get(&fixture, "/api/mcp/servers").await;
    assert_eq!(status, 200);
    assert!(body.get("columns").unwrap().is_array());
    assert!(body.get("entries").unwrap().is_array());
    assert!(body.get("issues").unwrap().is_array());
    let columns = body["columns"].as_array().unwrap();
    let harnesses: Vec<_> = columns
        .iter()
        .filter_map(|c| c.get("harness").and_then(|v| v.as_str()))
        .collect();
    for id in common::harness_ids() {
        assert!(harnesses.contains(&id), "missing harness column {id}");
    }
}

#[tokio::test]
async fn list_servers_loads_seeded_manifest() {
    let fixture = TestFixture::new();
    common::write_json(
        &fixture.paths.mcp_store_manifest,
        &serde_json::json!({
            "version": 6,
            "servers": [{
                "name": "fixture-server",
                "displayName": "Fixture Server",
                "source": { "kind": "manual", "locator": "fixture-server" },
                "transport": "http",
                "url": "https://mcp.example.com"
            }]
        }),
    );
    let (_, body) = mcp_get(&fixture, "/api/mcp/servers").await;
    let names: Vec<_> = body["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("name").and_then(|v| v.as_str()))
        .collect();
    assert!(names.contains(&"fixture-server"));
}

#[tokio::test]
async fn unmanaged_by_server_dedupes_identical_entries() {
    let fixture = TestFixture::new();
    let payload = serde_json::json!({ "context7": { "command": "uvx", "args": ["context7-mcp"] } });
    write_cursor_mcp(&fixture, payload.clone());
    write_claude_mcp(&fixture, payload);

    let (_, body) = mcp_get(&fixture, "/api/mcp/unmanaged/by-server").await;
    let servers = body["servers"].as_array().unwrap();
    assert_eq!(servers.len(), 1, "expected one deduped server, got {body}");
    assert_eq!(servers[0]["name"], "context7");
    assert_eq!(servers[0]["identical"], true);
    let harnesses: std::collections::HashSet<_> = servers[0]["sightings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s.get("harness").and_then(|v| v.as_str()))
        .collect();
    assert!(harnesses.contains("cursor"));
    assert!(harnesses.contains("claude"));
}

#[tokio::test]
async fn unmanaged_by_server_marks_differing_payloads() {
    let fixture = TestFixture::new();
    write_cursor_mcp(
        &fixture,
        serde_json::json!({ "foo": { "type": "http", "url": "https://cursor.example" } }),
    );
    write_claude_mcp(
        &fixture,
        serde_json::json!({ "foo": { "type": "http", "url": "https://claude.example" } }),
    );

    let (_, body) = mcp_get(&fixture, "/api/mcp/unmanaged/by-server").await;
    let server = &body["servers"][0];
    assert_eq!(server["identical"], false);
    assert!(server["canonicalSpec"].is_null());
}

#[tokio::test]
async fn adopt_identical_promotes_all_harnesses() {
    let fixture = TestFixture::new();
    let payload = serde_json::json!({ "context7": { "command": "uvx", "args": ["context7-mcp"] } });
    write_cursor_mcp(&fixture, payload.clone());
    write_claude_mcp(&fixture, payload);

    let (status, result) = mcp_post_json(
        &fixture,
        "/api/mcp/unmanaged/adopt",
        serde_json::json!({ "name": "context7" }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(result["ok"], true);
    let succeeded: std::collections::HashSet<_> = result["succeeded"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(succeeded.contains("cursor"));
    assert!(succeeded.contains("claude"));

    let (_, servers) = mcp_get(&fixture, "/api/mcp/servers").await;
    let names: Vec<_> = servers["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("name").and_then(|v| v.as_str()))
        .collect();
    assert!(names.contains(&"context7"));
}

#[tokio::test]
async fn adopt_differing_without_observed_harness_returns_409() {
    let fixture = TestFixture::new();
    write_cursor_mcp(
        &fixture,
        serde_json::json!({ "foo": { "type": "http", "url": "https://a.example" } }),
    );
    write_claude_mcp(
        &fixture,
        serde_json::json!({ "foo": { "type": "http", "url": "https://b.example" } }),
    );

    let (status, _) = mcp_post_json(
        &fixture,
        "/api/mcp/unmanaged/adopt",
        serde_json::json!({ "name": "foo" }),
    )
    .await;
    assert_eq!(status, 409);
}

#[tokio::test]
async fn adopt_differing_uses_selected_observed_harness() {
    let fixture = TestFixture::new();
    write_cursor_mcp(
        &fixture,
        serde_json::json!({ "foo": { "type": "http", "url": "https://cursor.example" } }),
    );
    write_claude_mcp(
        &fixture,
        serde_json::json!({ "foo": { "type": "http", "url": "https://claude.example" } }),
    );

    let (status, result) = mcp_post_json(
        &fixture,
        "/api/mcp/unmanaged/adopt",
        serde_json::json!({ "name": "foo", "observedHarness": "claude" }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(result["server"]["url"], "https://claude.example");
}

#[tokio::test]
async fn adopt_uses_observed_spec_not_placeholder_npx() {
    let fixture = TestFixture::new();
    write_cursor_mcp(
        &fixture,
        serde_json::json!({ "my-tool": { "command": "uvx", "args": ["my-tool-mcp"] } }),
    );

    let (_, result) = mcp_post_json(
        &fixture,
        "/api/mcp/unmanaged/adopt",
        serde_json::json!({ "name": "my-tool" }),
    )
    .await;
    assert_eq!(result["server"]["command"], "uvx");
    assert_eq!(result["server"]["args"], serde_json::json!(["my-tool-mcp"]));
}

#[tokio::test]
async fn get_server_returns_detail_for_managed_server() {
    let fixture = TestFixture::new();
    common::write_json(
        &fixture.paths.mcp_store_manifest,
        &serde_json::json!({
            "version": 6,
            "servers": [{
                "name": "fixture-server",
                "displayName": "Fixture Server",
                "source": { "kind": "manual", "locator": "fixture-server" },
                "transport": "http",
                "url": "https://mcp.example.com"
            }]
        }),
    );
    let (status, body) = mcp_get(&fixture, "/api/mcp/servers/fixture-server").await;
    assert_eq!(status, 200);
    assert_eq!(body["name"], "fixture-server");
    assert!(body.get("configChoices").is_some());
}

#[tokio::test]
async fn get_unknown_server_returns_404() {
    let fixture = TestFixture::new();
    let (status, _) = mcp_get(&fixture, "/api/mcp/servers/missing").await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn unmanaged_by_server_masks_secret_preview_fields() {
    let fixture = TestFixture::new();
    write_cursor_mcp(
        &fixture,
        serde_json::json!({
            "secreted": {
                "type": "http",
                "url": "https://api.example/mcp?api_key=live_secret_value",
                "headers": { "Authorization": "Bearer live_secret_value" }
            },
            "secretenv": {
                "command": "npx",
                "args": ["-y", "secretenv"],
                "env": { "EXA_API_KEY": "live_secret_value" }
            }
        }),
    );

    let (_, body) = mcp_get(&fixture, "/api/mcp/unmanaged/by-server").await;
    let encoded = body.to_string();
    assert!(!encoded.contains("live_secret_value"));
    let servers: std::collections::HashMap<_, _> = body["servers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|server| (server["name"].as_str().unwrap().to_string(), server.clone()))
        .collect();
    let remote = &servers["secreted"];
    assert!(remote["canonicalSpec"]["url"]
        .as_str()
        .unwrap()
        .contains("api_key=%5Bredacted%5D"));
    let stdio = &servers["secretenv"];
    assert_eq!(
        stdio["canonicalSpec"]["env"]["EXA_API_KEY"],
        "[redacted]"
    );
}
