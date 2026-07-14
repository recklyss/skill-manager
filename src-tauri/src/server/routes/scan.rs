use axum::{routing::get, Json, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/configs", get(list_configs))
}

async fn list_configs() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "configs": [],
        "activeId": null,
    }))
}
