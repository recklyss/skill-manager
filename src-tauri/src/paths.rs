use std::path::PathBuf;

const APP_NAME: &str = "skill-manager";
const SETTINGS_PATH_ENV: &str = "SKILL_MANAGER_SETTINGS_PATH";
const STATE_DIR_ENV: &str = "SKILL_MANAGER_STATE_DIR";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    pub skills_store_root: PathBuf,
    pub skills_store_manifest: PathBuf,
    pub marketplace_cache_root: PathBuf,
    pub mcp_store_manifest: PathBuf,
    pub slash_command_store_root: PathBuf,
    pub slash_command_commands_dir: PathBuf,
    pub slash_command_sync_state_path: PathBuf,
    pub settings_path: PathBuf,
    pub runtime_state_path: PathBuf,
    pub server_log_path: PathBuf,
    pub db_path: PathBuf,
}

impl AppPaths {
    /// Build paths from explicit config/data/state dirs (integration tests).
    pub fn from_dirs(config_dir: PathBuf, data_dir: PathBuf, state_dir: PathBuf) -> Self {
        Self {
            settings_path: config_dir.join("settings.json"),
            skills_store_root: data_dir.join("shared"),
            skills_store_manifest: data_dir.join("manifest.json"),
            marketplace_cache_root: data_dir.join("marketplace"),
            mcp_store_manifest: data_dir.join("mcp").join("manifest.json"),
            slash_command_store_root: data_dir.join("slash-commands"),
            slash_command_commands_dir: data_dir.join("slash-commands").join("commands"),
            slash_command_sync_state_path: data_dir.join("slash-commands").join("sync-state.json"),
            runtime_state_path: state_dir.join("runtime.json"),
            server_log_path: state_dir.join("server.log"),
            db_path: data_dir.join("skill-manager.db"),
            config_dir,
            data_dir,
            state_dir,
        }
    }

    /// Build paths rooted at a single directory (used by integration tests).
    pub fn from_data_root(data_dir: PathBuf) -> Self {
        Self::from_dirs(data_dir.clone(), data_dir.clone(), data_dir)
    }

    pub fn resolve() -> Self {
        Self::resolve_from(std::env::vars().collect())
    }

    pub fn resolve_from(env: std::collections::HashMap<String, String>) -> Self {
        let (config_dir, data_dir, state_dir) = base_dirs(&env);

        let settings_path = env
            .get(SETTINGS_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| config_dir.join("settings.json"));

        Self {
            skills_store_root: data_dir.join("shared"),
            skills_store_manifest: data_dir.join("manifest.json"),
            marketplace_cache_root: data_dir.join("marketplace"),
            mcp_store_manifest: data_dir.join("mcp").join("manifest.json"),
            slash_command_store_root: data_dir.join("slash-commands"),
            slash_command_commands_dir: data_dir.join("slash-commands").join("commands"),
            slash_command_sync_state_path: data_dir.join("slash-commands").join("sync-state.json"),
            settings_path,
            runtime_state_path: state_dir.join("runtime.json"),
            server_log_path: state_dir.join("server.log"),
            db_path: data_dir.join("skill-manager.db"),
            config_dir,
            data_dir,
            state_dir,
        }
    }
}

fn base_dirs(env: &std::collections::HashMap<String, String>) -> (PathBuf, PathBuf, PathBuf) {
    let home = dirs::home_dir().expect("HOME not set");

    if cfg!(target_os = "macos") {
        let default_macos = home
            .join("Library")
            .join("Application Support")
            .join(APP_NAME);
        let config_dir = xdg_dir(env, "XDG_CONFIG_HOME", &default_macos);
        let data_dir = xdg_dir(env, "XDG_DATA_HOME", &default_macos);
        let state_dir = env
            .get(STATE_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| xdg_dir(env, "XDG_STATE_HOME", &default_macos));
        (config_dir, data_dir, state_dir)
    } else {
        let xdg_config = env
            .get("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        let xdg_data = env
            .get("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local").join("share"));
        let xdg_state = env
            .get("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local").join("state"));

        let config_dir = xdg_dir(env, "XDG_CONFIG_HOME", &xdg_config.join(APP_NAME));
        let data_dir = xdg_dir(env, "XDG_DATA_HOME", &xdg_data.join(APP_NAME));
        let state_dir = env
            .get(STATE_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| xdg_dir(env, "XDG_STATE_HOME", &xdg_state.join(APP_NAME)));
        (config_dir, data_dir, state_dir)
    }
}

fn xdg_dir(env: &std::collections::HashMap<String, String>, key: &str, fallback: &PathBuf) -> PathBuf {
    env.get(key)
        .map(|value| PathBuf::from(value).join(APP_NAME))
        .unwrap_or_else(|| fallback.clone())
}
