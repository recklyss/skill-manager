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
        .route("/servers", get(list_servers).post(install_server))
        .route("/servers/:name", get(get_server).delete(uninstall_server))
        .route(
            "/servers/:name/availability/check",
            post(check_availability),
        )
        .route("/servers/:name/enable", post(enable_server))
        .route("/servers/:name/disable", post(disable_server))
        .route("/servers/:name/reconcile", post(reconcile_server))
        .route("/servers/:name/set-harnesses", post(set_harnesses))
        .route("/unmanaged/by-server", get(list_unmanaged))
        .route("/unmanaged/adopt", post(adopt_server))
}

async fn list_servers(State(state): State<AppState>) -> Json<Value> {
    Json(state.mcp.queries.list_servers())
}

async fn get_server(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<Value>> {
    state
        .mcp
        .queries
        .get_server(&name)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("unknown mcp server: {name}")))
}

#[derive(Deserialize)]
struct AddMcpServerRequest {
    #[serde(rename = "qualifiedName")]
    qualified_name: String,
}

async fn install_server(
    State(state): State<AppState>,
    Json(body): Json<AddMcpServerRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .mcp
            .mutations
            .install_from_marketplace(&body.qualified_name)
            .await?,
    ))
}

async fn uninstall_server(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.mcp.mutations.uninstall_server(&name)?))
}

async fn check_availability(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<Value>> {
    state
        .mcp
        .queries
        .check_availability(&name)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("unknown mcp server: {name}")))
}

#[derive(Deserialize)]
struct EnableMcpServerRequest {
    harness: String,
    config: Option<Value>,
}

async fn enable_server(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<EnableMcpServerRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .mcp
            .mutations
            .enable_server(&name, &body.harness, body.config)?,
    ))
}

#[derive(Deserialize)]
struct DisableMcpServerRequest {
    harness: String,
}

async fn disable_server(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<DisableMcpServerRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .mcp
            .mutations
            .disable_server(&name, &body.harness)?,
    ))
}

#[derive(Deserialize)]
struct ReconcileMcpServerRequest {
    #[serde(rename = "sourceKind")]
    source_kind: String,
    #[serde(rename = "observedHarness")]
    observed_harness: Option<String>,
    harnesses: Option<Vec<String>>,
}

async fn reconcile_server(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<ReconcileMcpServerRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.mcp.mutations.reconcile_server(
        &name,
        &body.source_kind,
        body.observed_harness,
        body.harnesses,
    )?))
}

#[derive(Deserialize)]
struct SetMcpServerHarnessesRequest {
    target: String,
    config: Option<Value>,
}

async fn set_harnesses(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<SetMcpServerHarnessesRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .mcp
            .mutations
            .set_server_all_harnesses(&name, &body.target, body.config)?,
    ))
}

async fn list_unmanaged(State(state): State<AppState>) -> Json<Value> {
    Json(state.mcp.queries.list_unmanaged_by_server())
}

#[derive(Deserialize)]
struct AdoptMcpRequest {
    name: String,
    #[serde(rename = "observedHarness")]
    observed_harness: Option<String>,
    harnesses: Option<Vec<String>>,
}

async fn adopt_server(
    State(state): State<AppState>,
    Json(body): Json<AdoptMcpRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.mcp.mutations.adopt(
        &body.name,
        body.observed_harness.as_deref(),
        body.harnesses,
    )?))
}
