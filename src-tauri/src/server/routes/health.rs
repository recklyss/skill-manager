use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let snapshot = state.skills_queries.read_models().snapshot();
    Json(json!({
        "ok": true,
        "app": "skill-manager",
        "readOnly": false,
        "harnessCount": snapshot.harness_scans.len(),
    }))
}
