use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

pub fn pi_skills_root(ctx: &ResolutionContext) -> PathBuf {
    pi_agent_dir(ctx).join("skills")
}

/// Pi coding agent directory: `$PI_CODING_AGENT_DIR` override, otherwise `~/.pi/agent`.
pub fn pi_agent_dir(ctx: &ResolutionContext) -> PathBuf {
    ctx.env
        .get("PI_CODING_AGENT_DIR")
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| ctx.home.join(".pi").join("agent"))
}

pub fn pi_mcp_config(ctx: &ResolutionContext) -> PathBuf {
    pi_agent_dir(ctx).join("mcp.json")
}

/// DeepSeek Harness (dsh) home: `$DSH_HOME` override, otherwise `~/.dsh`.
pub fn dsh_home(ctx: &ResolutionContext) -> PathBuf {
    ctx.env
        .get("DSH_HOME")
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| ctx.home.join(".dsh"))
}

pub fn deepseek_skills_root(ctx: &ResolutionContext) -> PathBuf {
    dsh_home(ctx).join("skills")
}

/// DeepSeek Harness MCP config: user MCP servers live as `dsh-mcp-client`
/// plugin rows in a profile's `cordis.patch.yml`. Defaults to the `web`
/// profile; override with `SKILL_MANAGER_DEEPSEEK_MCP_CONFIG`.
pub fn deepseek_mcp_config(ctx: &ResolutionContext) -> PathBuf {
    ctx.env
        .get("SKILL_MANAGER_DEEPSEEK_MCP_CONFIG")
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dsh_home(ctx)
                .join("profiles")
                .join("web")
                .join("cordis.patch.yml")
        })
}

pub fn copilot_settings_skill_directories(ctx: &ResolutionContext) -> Vec<PathBuf> {
    let settings_path = ctx.home.join(".copilot").join("settings.json");
    let Ok(raw) = std::fs::read_to_string(&settings_path) else {
        return Vec::new();
    };
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(&crate::mcp::strip_jsonc(&raw)) else {
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

/// Resolve a CLI binary using the active environment's `PATH` (not only process env).
pub fn resolve_executable_path(ctx: &ResolutionContext, binary: &str) -> Option<PathBuf> {
    let path_var = ctx.env.get("PATH").map(String::as_str).unwrap_or_default();
    for dir in std::env::split_paths(path_var) {
        let candidate = dir.join(binary);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    None
}

pub fn is_executable_on_path(ctx: &ResolutionContext, binary: &str) -> bool {
    resolve_executable_path(ctx, binary).is_some()
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return std::fs::metadata(path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false);
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[cfg(unix)]
    fn write_executable_stub(path: &Path, name: &str) {
        let stub = path.join(name);
        fs::write(&stub, format!("#!/bin/sh\nprintf '%s\\n' '{name}'\n")).expect("write stub");
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&stub).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&stub, perms).expect("chmod stub");
    }

    #[test]
    #[cfg(unix)]
    fn resolve_executable_path_uses_context_path() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_executable_stub(dir.path(), "copilot");

        let mut env = std::collections::HashMap::new();
        env.insert("PATH".into(), dir.path().display().to_string());
        let ctx = resolve_context(Some(env));

        assert!(is_executable_on_path(&ctx, "copilot"));
        assert_eq!(
            resolve_executable_path(&ctx, "copilot").expect("copilot path"),
            dir.path().join("copilot")
        );
    }
}
