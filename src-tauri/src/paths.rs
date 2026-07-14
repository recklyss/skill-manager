use std::path::PathBuf;

const APP_NAME: &str = "skill-manager";

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
    pub db_path: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> Self {
        let (config_dir, data_dir, state_dir) = base_dirs();

        Self {
            settings_path: config_dir.join("settings.json"),
            skills_store_root: data_dir.join("shared"),
            skills_store_manifest: data_dir.join("manifest.json"),
            marketplace_cache_root: data_dir.join("marketplace"),
            mcp_store_manifest: data_dir.join("mcp").join("manifest.json"),
            slash_command_store_root: data_dir.join("slash-commands"),
            slash_command_commands_dir: data_dir.join("slash-commands").join("commands"),
            slash_command_sync_state_path: data_dir.join("slash-commands").join("sync-state.json"),
            db_path: data_dir.join("skill-manager.db"),
            config_dir,
            data_dir,
            state_dir,
        }
    }
}

fn base_dirs() -> (PathBuf, PathBuf, PathBuf) {
    if cfg!(target_os = "macos") {
        let home = dirs::home_dir().expect("HOME not set");
        let base = home.join("Library").join("Application Support").join(APP_NAME);
        // On macOS, config/data/state share the same base.
        (base.clone(), base.clone(), base)
    } else {
        let config = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
            .join(APP_NAME);
        let data = dirs::data_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".local").join("share"))
            .join(APP_NAME);
        let state = dirs::state_dir()
            .unwrap_or_else(|| data.clone());
        (config, data, state)
    }
}
