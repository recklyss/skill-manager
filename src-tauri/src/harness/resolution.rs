use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Macos,
    Linux,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Linux => "linux",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolutionContext {
    pub env: HashMap<String, String>,
    pub platform: Platform,
    pub home: PathBuf,
    pub xdg_config_home: PathBuf,
    pub xdg_data_home: PathBuf,
    pub xdg_state_home: PathBuf,
}

pub fn resolve_context(env: Option<HashMap<String, String>>) -> ResolutionContext {
    let mut active_env: HashMap<String, String> = std::env::vars().collect();
    if let Some(overrides) = env {
        active_env.extend(overrides);
    }

    let home = active_env
        .get("HOME")
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .expect("HOME not set");

    let platform = if cfg!(target_os = "macos") {
        Platform::Macos
    } else {
        Platform::Linux
    };

    ResolutionContext {
        xdg_config_home: active_env
            .get("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config")),
        xdg_data_home: active_env
            .get("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local").join("share")),
        xdg_state_home: active_env
            .get("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local").join("state")),
        env: active_env,
        platform,
        home,
    }
}

pub type PathFn = fn(&ResolutionContext) -> PathBuf;

pub fn hermes_home(ctx: &ResolutionContext) -> PathBuf {
    ctx.env
        .get("SKILL_MANAGER_HERMES_HOME")
        .or(ctx.env.get("HERMES_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| ctx.home.join(".hermes"))
}

pub fn codex_skills_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".agents").join("skills")
}

pub fn codex_admin_skills_root(_ctx: &ResolutionContext) -> PathBuf {
    PathBuf::from("/etc/codex/skills")
}

pub fn codex_legacy_skills_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".codex").join("skills")
}

pub fn agents_skills_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".agents").join("skills")
}

pub fn claude_skills_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".claude").join("skills")
}

pub fn cursor_skills_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".cursor").join("skills")
}

pub fn opencode_skills_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.xdg_config_home.join("opencode").join("skills")
}

pub fn hermes_skills_root(ctx: &ResolutionContext) -> PathBuf {
    hermes_home(ctx).join("skills")
}

pub fn openclaw_skills_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".openclaw").join("skills")
}

pub fn copilot_skills_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".copilot").join("skills")
}

pub fn copilot_installed_plugins_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".copilot").join("installed-plugins")
}

pub fn copilot_settings_skill_directories(ctx: &ResolutionContext) -> Vec<PathBuf> {
    let settings_path = ctx.home.join(".copilot").join("settings.json");
    let Ok(raw) = std::fs::read_to_string(&settings_path) else {
        return Vec::new();
    };
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(&strip_jsonc_line_comments(&raw)) else {
        return Vec::new();
    };
    let Some(values) = payload
        .get("skillDirectories")
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };

    let mut directories = Vec::new();
    for value in values {
        let Some(raw_path) = value.as_str() else {
            continue;
        };
        let trimmed = raw_path.trim();
        if trimmed.is_empty() {
            continue;
        }
        let expanded = if let Some(stripped) = trimmed.strip_prefix("~/") {
            ctx.home.join(stripped)
        } else if trimmed == "~" {
            ctx.home.clone()
        } else {
            PathBuf::from(trimmed)
        };
        directories.push(expanded);
    }
    directories
}

fn strip_jsonc_line_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if escaped {
            output.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            output.push(ch);
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            output.push(ch);
            continue;
        }
        if !in_string && ch == '/' && chars.peek() == Some(&'/') {
            while matches!(chars.peek(), Some('\n' | '\r')) {
                chars.next();
            }
            while matches!(chars.peek(), Some(c) if *c != '\n' && *c != '\r') {
                chars.next();
            }
            continue;
        }
        output.push(ch);
    }
    output
}

pub fn cursor_app_paths(ctx: &ResolutionContext) -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications/Cursor.app"),
        ctx.home.join("Applications").join("Cursor.app"),
    ]
}
