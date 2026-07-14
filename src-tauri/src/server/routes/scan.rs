use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiResult;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/availability", get(check_availability))
        .route("/llm/detection", get(detect_llm))
        .route(
            "/configs",
            get(list_configs).post(create_config),
        )
        .route("/configs/validate", post(validate_config))
        .route(
            "/configs/:config_id",
            put(update_config).delete(delete_config),
        )
        .route("/configs/:config_id/active", put(set_active_config))
        .route("/configs/:config_id/secret", get(reveal_secret))
        .route("/skills/:skill_ref", post(scan_skill))
}

async fn check_availability(State(state): State<AppState>) -> Json<Value> {
    Json(serde_json::json!({ "available": state.scan.service.available() }))
}

async fn detect_llm(State(state): State<AppState>) -> Json<Value> {
    Json(state.scan.service.detect_llm())
}

async fn list_configs(State(state): State<AppState>) -> Json<Value> {
    Json(state.scan.config.list_configs())
}

async fn create_config(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.scan.config.create_config(&body)?))
}

async fn validate_config(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<Value> {
    Json(state.scan.config.validate_config(&body))
}

async fn update_config(
    State(state): State<AppState>,
    Path(config_id): Path<i64>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.scan.config.update_config(config_id, &body)?))
}

async fn delete_config(
    State(state): State<AppState>,
    Path(config_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.scan.config.delete_config(config_id)?))
}

async fn set_active_config(
    State(state): State<AppState>,
    Path(config_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.scan.config.set_active(config_id)?))
}

async fn reveal_secret(
    State(state): State<AppState>,
    Path(config_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.scan.config.reveal_secret(config_id)?))
}

#[derive(Deserialize, Default)]
struct ScanOptionsRequest {
    #[serde(rename = "useLlm", default = "default_use_llm")]
    use_llm: bool,
}

fn default_use_llm() -> bool {
    true
}

async fn scan_skill(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
    body: Option<Json<ScanOptionsRequest>>,
) -> ApiResult<Json<Value>> {
    let options = body.map(|Json(request)| {
        serde_json::json!({
            "useLlm": request.use_llm,
        })
    });
    Ok(Json(state.scan.service.scan_skill(&skill_ref, options)?))
}
