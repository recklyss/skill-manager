use axum::{extract::State, routing::get, Json, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(get_settings))
}

async fn get_settings(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "storage": {
            "platform": if cfg!(target_os = "macos") { "macos" } else { "linux" },
            "configDir": state.paths.config_dir.to_string_lossy(),
            "dataDir": state.paths.data_dir.to_string_lossy(),
            "stateDir": state.paths.state_dir.to_string_lossy(),
            "skillsStorePath": state.paths.skills_store_root.to_string_lossy(),
            "marketplaceCachePath": state.paths.marketplace_cache_root.to_string_lossy(),
            "settingsPath": state.paths.settings_path.to_string_lossy(),
        },
        "harnesses": state.harness_kernel.statuses(),
    }))
}
