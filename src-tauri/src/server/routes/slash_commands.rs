use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::error::{ApiError, ApiResult};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/slash-commands", get(list_commands).post(create_command))
        .route("/slash-commands/review/import", post(import_command))
        .route("/slash-commands/review/resolve", post(resolve_review))
        .route(
            "/slash-commands/:name",
            get(get_command)
                .put(update_command)
                .delete(delete_command),
        )
        .route("/slash-commands/:name/sync", post(sync_command))
}

async fn list_commands(State(state): State<AppState>) -> Json<Value> {
    Json(state.slash_commands.queries.list_commands())
}

async fn get_command(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<Value>> {
    state
        .slash_commands
        .queries
        .get_command(&name)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("unknown slash command: {name}")))
}

#[derive(Deserialize)]
struct SlashCommandMutationRequest {
    name: String,
    description: String,
    prompt: String,
    targets: Option<Vec<String>>,
}

async fn create_command(
    State(state): State<AppState>,
    Json(body): Json<SlashCommandMutationRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.slash_commands.mutations.create_command(
        &body.name,
        &body.description,
        &body.prompt,
        body.targets,
    )?))
}

#[derive(Deserialize)]
struct SlashCommandUpdateRequest {
    description: String,
    prompt: String,
    targets: Option<Vec<String>>,
}

async fn update_command(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<SlashCommandUpdateRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.slash_commands.mutations.update_command(
        &name,
        &body.description,
        &body.prompt,
        body.targets,
    )?))
}

#[derive(Deserialize)]
struct SlashSyncRequest {
    targets: Option<Vec<String>>,
}

async fn sync_command(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<SlashSyncRequest>,
) -> ApiResult<Json<Value>> {
    let sync = state
        .slash_commands
        .mutations
        .sync_command(&name, body.targets)?;
    Ok(Json(serde_json::json!({
        "ok": sync.get("ok").cloned().unwrap_or(serde_json::Value::Bool(true)),
        "command": state.slash_commands.queries.get_command(&name),
        "sync": sync.get("sync").cloned().unwrap_or(serde_json::Value::Array(vec![])),
    })))
}

async fn delete_command(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.slash_commands.mutations.delete_command(&name)?))
}

#[derive(Deserialize)]
struct SlashCommandImportRequest {
    target: String,
    name: String,
}

async fn import_command(
    State(state): State<AppState>,
    Json(body): Json<SlashCommandImportRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .slash_commands
            .mutations
            .import_unmanaged_command(&body.target, &body.name)?,
    ))
}

#[derive(Deserialize)]
struct SlashCommandResolveRequest {
    target: String,
    name: String,
    action: String,
}

async fn resolve_review(
    State(state): State<AppState>,
    Json(body): Json<SlashCommandResolveRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.slash_commands.mutations.resolve_review_command(
        &body.target,
        &body.name,
        &body.action,
    )?))
}
