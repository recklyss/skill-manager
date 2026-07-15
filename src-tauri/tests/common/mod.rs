use std::fs;
use std::path::{Path, PathBuf};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use skill_manager_lib::{api_router, build_app_state_with_env, paths::AppPaths};
use tempfile::TempDir;
use tower::ServiceExt;

pub struct TestFixture {
    pub _dir: TempDir,
    pub paths: AppPaths,
    pub app: Router,
    env: std::collections::HashMap<String, String>,
}

impl TestFixture {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("temp dir");
        let root = dir.path();
        let paths = AppPaths::from_dirs(
            root.join("config"),
            root.join("data"),
            root.join("state"),
        );
        let env = isolate_harness_roots(root);
        let state = build_app_state_with_env(paths.clone(), env.clone());
        let app = api_router(state);
        Self {
            _dir: dir,
            paths,
            app,
            env,
        }
    }

    pub fn rebuild_app(&self) -> Router {
        api_router(build_app_state_with_env(
            self.paths.clone(),
            self.env.clone(),
        ))
    }

    pub async fn get(&self, path: &str) -> (StatusCode, serde_json::Value) {
        self.request(Request::get(path).body(Body::empty()).unwrap())
            .await
    }

    pub async fn put_json(&self, path: &str, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        self.request(
            Request::put(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
    }

    pub async fn post(&self, path: &str) -> (StatusCode, serde_json::Value) {
        self.request(Request::post(path).body(Body::empty()).unwrap())
            .await
    }

    pub async fn post_json(&self, path: &str, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        self.request(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
    }

    pub async fn delete(&self, path: &str) -> (StatusCode, serde_json::Value) {
        self.request(Request::delete(path).body(Body::empty()).unwrap())
            .await
    }

    async fn request(&self, req: Request<Body>) -> (StatusCode, serde_json::Value) {
        let response = self.app.clone().oneshot(req).await.expect("request");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json = if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or_else(|_| {
                serde_json::json!({ "raw": String::from_utf8_lossy(&bytes) })
            })
        };
        (status, json)
    }
}

pub fn write_json(path: &Path, value: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir");
    }
    fs::write(path, serde_json::to_string_pretty(value).unwrap()).expect("write json");
}

/// Point harness skill roots at empty temp dirs so host machine skills do not leak into tests.
fn isolate_harness_roots(root: &Path) -> std::collections::HashMap<String, String> {
    let mut env: std::collections::HashMap<String, String> = std::env::vars().collect();
    let harness_roots = root.join("harness-roots");
    let home = root.join("home");
    fs::create_dir_all(&home).expect("test home");
    env.insert("HOME".into(), home.display().to_string());
    env.insert(
        "XDG_CONFIG_HOME".into(),
        home.join(".config").display().to_string(),
    );

    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir");
    for executable in ["codex", "claude", "cursor-agent", "opencode", "hermes", "openclaw"] {
        write_cli_stub(&bin_dir.join(executable), executable);
    }
    let path = env.get("PATH").cloned().unwrap_or_default();
    env.insert("PATH".into(), format!("{}:{path}", bin_dir.display()));

    for (env_key, name) in [
        ("SKILL_MANAGER_CODEX_ROOT", "codex"),
        ("SKILL_MANAGER_CLAUDE_ROOT", "claude"),
        ("SKILL_MANAGER_CURSOR_ROOT", "cursor"),
        ("SKILL_MANAGER_OPENCODE_ROOT", "opencode"),
        ("SKILL_MANAGER_HERMES_ROOT", "hermes"),
        ("SKILL_MANAGER_OPENCLAW_ROOT", "openclaw"),
    ] {
        let path = harness_roots.join(name);
        fs::create_dir_all(&path).expect("harness root");
        env.insert(env_key.into(), path.display().to_string());
    }
    env
}

fn write_cli_stub(path: &Path, executable: &str) {
    fs::write(
        path,
        format!("#!/bin/sh\nprintf '%s\\n' '{executable}'\n"),
    )
    .expect("write cli stub");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("chmod stub");
    }
}

pub fn codex_legacy_root(root: &Path) -> PathBuf {
    root.join("home").join(".codex").join("skills")
}

pub fn hermes_skills_root(root: &Path) -> PathBuf {
    root.join("harness-roots").join("hermes")
}

pub fn seed_store_manifest(paths: &AppPaths, entries: &[serde_json::Value]) {
    write_json(
        &paths.skills_store_manifest,
        &serde_json::json!({ "entries": entries }),
    );
}

pub fn seed_skill(store_root: &Path, name: &str, description: &str) -> PathBuf {
    seed_named_skill(store_root, name, name, description)
}

pub fn seed_named_skill(
    store_root: &Path,
    dir_name: &str,
    declared_name: &str,
    description: &str,
) -> PathBuf {
    let skill_dir = store_root.join(dir_name);
    fs::create_dir_all(&skill_dir).expect("skill dir");
    fs::write(
        skill_dir.join("SKILL.md"),
        format!(
            "---\nname: {declared_name}\ndescription: {description}\n---\n\n# {declared_name}\n"
        ),
    )
    .expect("SKILL.md");
    skill_dir
}

pub fn harness_ids() -> Vec<&'static str> {
    vec![
        "codex",
        "claude",
        "cursor",
        "opencode",
        "hermes",
        "openclaw",
        "copilot",
    ]
}

pub fn find_harness<'a>(harnesses: &'a [serde_json::Value], id: &str) -> &'a serde_json::Value {
    harnesses
        .iter()
        .find(|h| h.get("harness").and_then(|v| v.as_str()) == Some(id))
        .unwrap_or_else(|| panic!("harness '{id}' not found"))
}
