use axum::{extract::State, routing::get, Json, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/slash-commands", get(list_commands))
}

async fn list_commands(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "storePath": "",
        "syncStatePath": "",
        "targets": [],
        "defaultTargets": [],
        "commands": [],
        "reviewCommands": [],
    }))
}
