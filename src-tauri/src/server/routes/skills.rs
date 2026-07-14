use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/skills", get(list_skills))
        .route("/skills/manage-all", post(manage_all_skills))
        .route("/skills/:skill_ref/source-status", get(get_skill_source_status))
        .route("/skills/:skill_ref/enable", post(enable_skill))
        .route("/skills/:skill_ref/disable", post(disable_skill))
        .route("/skills/:skill_ref/set-harnesses", post(set_skill_harnesses))
        .route("/skills/:skill_ref/manage", post(manage_skill))
        .route("/skills/:skill_ref/update", post(update_skill))
        .route("/skills/:skill_ref/unmanage", post(unmanage_skill))
        .route("/skills/:skill_ref/delete", post(delete_skill))
        .route("/skills/:skill_ref", get(get_skill_detail))
}

async fn list_skills(State(state): State<AppState>) -> Json<serde_json::Value> {
    let page = state.skills_queries.page_response();
    Json(serde_json::to_value(page).unwrap_or_default())
}

async fn get_skill_detail(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let detail = state
        .skills_queries
        .get_skill_detail(&skill_ref)
        .ok_or_else(|| ApiError::not_found(format!("unknown skill ref: {skill_ref}")))?;
    Ok(Json(serde_json::to_value(detail).unwrap_or_default()))
}

async fn get_skill_source_status(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let status = state
        .skills_queries
        .get_skill_source_status(&skill_ref)
        .ok_or_else(|| ApiError::not_found(format!("unknown skill ref: {skill_ref}")))?;
    Ok(Json(serde_json::to_value(status).unwrap_or_default()))
}

#[derive(Deserialize)]
struct HarnessTargetRequest {
    harness: String,
}

async fn enable_skill(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
    Json(body): Json<HarnessTargetRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(
        state
            .skills_mutations
            .enable_skill(&skill_ref, &body.harness)?,
    ))
}

async fn disable_skill(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
    Json(body): Json<HarnessTargetRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(
        state
            .skills_mutations
            .disable_skill(&skill_ref, &body.harness)?,
    ))
}

#[derive(Deserialize)]
struct SetSkillHarnessesRequest {
    target: String,
}

async fn set_skill_harnesses(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
    Json(body): Json<SetSkillHarnessesRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(
        state
            .skills_mutations
            .set_skill_all_harnesses(&skill_ref, &body.target)?,
    ))
}

async fn manage_skill(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.skills_mutations.manage_skill(&skill_ref)?))
}

async fn manage_all_skills(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.skills_mutations.manage_all_skills()?))
}

async fn update_skill(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.skills_mutations.update_skill(&skill_ref)?))
}

async fn unmanage_skill(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.skills_mutations.unmanage_skill(&skill_ref)?))
}

async fn delete_skill(
    State(state): State<AppState>,
    Path(skill_ref): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.skills_mutations.delete_skill(&skill_ref)?))
}
