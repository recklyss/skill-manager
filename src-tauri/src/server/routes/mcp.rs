use axum::{extract::State, routing::get, Json, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/servers", get(list_servers))
}

async fn list_servers(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "columns": [],
        "entries": [],
        "issues": [],
    }))
}
