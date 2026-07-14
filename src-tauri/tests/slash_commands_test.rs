mod common;

use std::fs;

use common::{write_json, TestFixture};
use sha2::{Digest, Sha256};

fn test_home(fixture: &TestFixture) -> std::path::PathBuf {
    fixture._dir.path().join("home")
}

fn codex_prompts_dir(fixture: &TestFixture) -> std::path::PathBuf {
    test_home(fixture).join(".codex").join("prompts")
}

fn cursor_commands_dir(fixture: &TestFixture) -> std::path::PathBuf {
    test_home(fixture).join(".cursor").join("commands")
}

/// GET /api/slash-commands returns SlashCommandListResponse fields.
#[tokio::test]
async fn list_commands_returns_list_envelope() {
    let fixture = TestFixture::new();
    let (status, body) = fixture.get("/api/slash-commands").await;

    assert_eq!(status, 200);
    assert!(body.get("storePath").is_some());
    assert!(body.get("syncStatePath").is_some());
    assert!(body.get("targets").unwrap().is_array());
    assert!(body.get("commands").unwrap().is_array());
    assert!(body.get("reviewCommands").unwrap().is_array());
}

/// On-disk TOML command records are indexed by the slash-command store.
#[tokio::test]
async fn list_commands_reads_seeded_toml_record() {
    let fixture = TestFixture::new();
    let commands_dir = &fixture.paths.slash_command_commands_dir;
    fs::create_dir_all(commands_dir).unwrap();
    fs::write(
        commands_dir.join("hello.toml"),
        r#"
name = "hello"
description = "Say hello"
prompt = "Hello world"
"#,
    )
    .unwrap();

    let (_, body) = fixture.get("/api/slash-commands").await;
    let commands = body["commands"].as_array().unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0]["name"], "hello");
}

/// POST /api/slash-commands creates a managed command.
#[tokio::test]
async fn create_command_persists_record() {
    let fixture = TestFixture::new();
    let prompts = codex_prompts_dir(&fixture);
    fs::create_dir_all(&prompts).unwrap();

    let (status, body) = fixture
        .post_json(
            "/api/slash-commands",
            serde_json::json!({
                "name": "hello",
                "description": "Say hello",
                "prompt": "Hello",
                "targets": ["codex"]
            }),
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(body["command"]["name"], "hello");
    assert!(body["sync"].as_array().unwrap()[0]["status"] == "synced");
}

/// GET /api/slash-commands/{name} returns a managed command.
#[tokio::test]
async fn get_command_returns_managed_record() {
    let fixture = TestFixture::new();
    let commands_dir = &fixture.paths.slash_command_commands_dir;
    fs::create_dir_all(commands_dir).unwrap();
    fs::write(
        commands_dir.join("hello.toml"),
        "name = \"hello\"\ndescription = \"Say hello\"\nprompt = \"Hello world\"\n",
    )
    .unwrap();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = fixture
        .rebuild_app()
        .oneshot(
            axum::http::Request::get("/api/slash-commands/hello")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), 200);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["name"], "hello");
}

/// Sync hash tracking surfaces drift in review rows.
#[tokio::test]
async fn drifted_tracked_file_appears_in_review() {
    let fixture = TestFixture::new();
    let prompts = codex_prompts_dir(&fixture);
    fs::create_dir_all(&prompts).unwrap();
    let output = prompts.join("hello.md");
    fs::write(&output, "---\ndescription: Say hello\n---\n\nHello world\n").unwrap();
    let content_hash = format!("sha256:{:x}", Sha256::digest(fs::read(&output).unwrap()));
    write_json(
        &fixture.paths.slash_command_sync_state_path,
        &serde_json::json!({
            "commands": {
                "hello": {
                    "codex": {
                        "target": "codex",
                        "path": output,
                        "contentHash": content_hash,
                        "renderFormat": "frontmatter_markdown"
                    }
                }
            }
        }),
    );
    fs::create_dir_all(&fixture.paths.slash_command_commands_dir).unwrap();
    fs::write(
        fixture.paths.slash_command_commands_dir.join("hello.toml"),
        "name = \"hello\"\ndescription = \"Say hello\"\nprompt = \"Hello world\"\n",
    )
    .unwrap();
    fs::write(&output, "manual edit").unwrap();

    let (_, body) = fixture.get("/api/slash-commands").await;
    let review = body["reviewCommands"].as_array().unwrap();
    assert_eq!(review.len(), 1);
    assert_eq!(review[0]["kind"], "drifted");
    let actions: Vec<&str> = review[0]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(
        actions,
        vec!["restore_managed", "adopt_target", "remove_binding"]
    );
}

/// Sync refuses to overwrite an untracked manual file.
#[tokio::test]
async fn sync_blocks_manual_file_overwrite() {
    let fixture = TestFixture::new();
    let prompts = codex_prompts_dir(&fixture);
    fs::create_dir_all(&prompts).unwrap();
    let manual = prompts.join("code-review.md");
    fs::write(&manual, "manual").unwrap();
    fs::create_dir_all(&fixture.paths.slash_command_commands_dir).unwrap();
    fs::write(
        fixture
            .paths
            .slash_command_commands_dir
            .join("code-review.toml"),
        "name = \"code-review\"\ndescription = \"Review\"\nprompt = \"$ARGUMENTS\"\n",
    )
    .unwrap();

    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let response = fixture
        .rebuild_app()
        .oneshot(
            axum::http::Request::post("/api/slash-commands/code-review/sync")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "targets": ["codex"] }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), 200);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["ok"], false);
    assert_eq!(body["sync"][0]["status"], "blocked_manual_file");
    assert_eq!(fs::read_to_string(&manual).unwrap(), "manual");
}

/// adopt_target records hash without overwriting harness file.
#[tokio::test]
async fn adopt_target_does_not_overwrite_harness_file() {
    let fixture = TestFixture::new();
    let prompts = codex_prompts_dir(&fixture);
    fs::create_dir_all(&prompts).unwrap();
    let output = prompts.join("code-review.md");
    fs::write(&output, "manual edit").unwrap();
    fs::create_dir_all(&fixture.paths.slash_command_commands_dir).unwrap();
    fs::write(
        fixture
            .paths
            .slash_command_commands_dir
            .join("code-review.toml"),
        "name = \"code-review\"\ndescription = \"Review\"\nprompt = \"$ARGUMENTS\"\n",
    )
    .unwrap();
    let content_hash = format!("sha256:{:x}", Sha256::digest(b"original"));
    write_json(
        &fixture.paths.slash_command_sync_state_path,
        &serde_json::json!({
            "commands": {
                "code-review": {
                    "codex": {
                        "target": "codex",
                        "path": output,
                        "contentHash": content_hash,
                        "renderFormat": "frontmatter_markdown"
                    }
                }
            }
        }),
    );

    let (_, body) = fixture
        .post_json(
            "/api/slash-commands/review/resolve",
            serde_json::json!({
                "target": "codex",
                "name": "code-review",
                "action": "adopt_target"
            }),
        )
        .await;
    assert_eq!(body["ok"], true);
    assert_eq!(fs::read_to_string(&output).unwrap(), "manual edit");
}

/// import_unmanaged_command imports without destructive re-sync.
#[tokio::test]
async fn import_unmanaged_command_preserves_harness_file() {
    let fixture = TestFixture::new();
    let commands = cursor_commands_dir(&fixture);
    fs::create_dir_all(&commands).unwrap();
    let target_file = commands.join("code-review.md");
    fs::write(&target_file, "Review code\n\nReview:\n$ARGUMENTS\n").unwrap();

    let (_, body) = fixture
        .post_json(
            "/api/slash-commands/review/import",
            serde_json::json!({
                "target": "cursor",
                "name": "code-review"
            }),
        )
        .await;
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"]["description"], "Review code");
    assert_eq!(
        fs::read_to_string(&target_file).unwrap(),
        "Review code\n\nReview:\n$ARGUMENTS\n"
    );
}

/// Missing tracked files appear in review with restore/remove actions.
#[tokio::test]
async fn missing_tracked_file_appears_in_review() {
    let fixture = TestFixture::new();
    let prompts = codex_prompts_dir(&fixture);
    fs::create_dir_all(&prompts).unwrap();
    let output = prompts.join("code-review.md");
    fs::write(&output, "---\ndescription: Review\n---\n\n$ARGUMENTS\n").unwrap();
    let content_hash = format!("sha256:{:x}", Sha256::digest(fs::read(&output).unwrap()));
    fs::remove_file(&output).unwrap();
    fs::create_dir_all(&fixture.paths.slash_command_commands_dir).unwrap();
    fs::write(
        fixture
            .paths
            .slash_command_commands_dir
            .join("code-review.toml"),
        "name = \"code-review\"\ndescription = \"Review\"\nprompt = \"$ARGUMENTS\"\n",
    )
    .unwrap();
    write_json(
        &fixture.paths.slash_command_sync_state_path,
        &serde_json::json!({
            "commands": {
                "code-review": {
                    "codex": {
                        "target": "codex",
                        "path": output,
                        "contentHash": content_hash,
                        "renderFormat": "frontmatter_markdown"
                    }
                }
            }
        }),
    );

    let (_, body) = fixture.get("/api/slash-commands").await;
    let review = body["reviewCommands"].as_array().unwrap();
    assert_eq!(review[0]["kind"], "missing");
    let actions: Vec<&str> = review[0]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(actions, vec!["restore_managed", "remove_binding"]);
}
