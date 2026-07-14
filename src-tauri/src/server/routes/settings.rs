use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::harness::HarnessStatus;
use crate::harness::Platform;
use crate::paths::AppPaths;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings", get(get_settings))
        .route("/settings/harnesses/:harness/support", put(set_harness_support))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsResponse {
    storage: SettingsStorageResponse,
    harnesses: Vec<SettingsHarnessResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsStorageResponse {
    platform: &'static str,
    config_dir: String,
    data_dir: String,
    state_dir: String,
    skills_store_path: String,
    marketplace_cache_path: String,
    settings_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsHarnessResponse {
    harness: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logo_key: Option<String>,
    support_enabled: bool,
    installed: bool,
    managed_location: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SetHarnessSupportRequest {
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct SetHarnessSupportResponse {
    ok: bool,
    enabled: bool,
}

async fn get_settings(State(state): State<AppState>) -> Json<SettingsResponse> {
    let enabled: std::collections::HashSet<String> = state
        .harness_kernel
        .enabled_harness_ids()
        .into_iter()
        .collect();

    Json(SettingsResponse {
        storage: storage_payload(&state.paths, state.harness_kernel.context.platform),
        harnesses: state
            .harness_kernel
            .harness_statuses()
            .into_iter()
            .map(|status| {
                let harness_id = status.harness.clone();
                harness_payload(status, enabled.contains(&harness_id))
            })
            .collect(),
    })
}

async fn set_harness_support(
    State(state): State<AppState>,
    Path(harness): Path<String>,
    Json(body): Json<SetHarnessSupportRequest>,
) -> Result<Json<SetHarnessSupportResponse>, (StatusCode, String)> {
    if !state.harness_kernel.is_known_harness(&harness) {
        return Err((StatusCode::NOT_FOUND, format!("unknown harness: {harness}")));
    }

    state
        .harness_kernel
        .support_store
        .set_enabled(&harness, body.enabled)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(SetHarnessSupportResponse {
        ok: true,
        enabled: body.enabled,
    }))
}

fn storage_payload(paths: &AppPaths, platform: Platform) -> SettingsStorageResponse {
    SettingsStorageResponse {
        platform: platform.as_str(),
        config_dir: paths.config_dir.to_string_lossy().into_owned(),
        data_dir: paths.data_dir.to_string_lossy().into_owned(),
        state_dir: paths.state_dir.to_string_lossy().into_owned(),
        skills_store_path: paths.skills_store_root.to_string_lossy().into_owned(),
        marketplace_cache_path: paths.marketplace_cache_root.to_string_lossy().into_owned(),
        settings_path: paths.settings_path.to_string_lossy().into_owned(),
    }
}

fn harness_payload(status: HarnessStatus, support_enabled: bool) -> SettingsHarnessResponse {
    SettingsHarnessResponse {
        harness: status.harness,
        label: status.label,
        logo_key: status.logo_key,
        support_enabled,
        installed: status.installed,
        managed_location: status
            .managed_location
            .map(|path| path.to_string_lossy().into_owned()),
    }
}
