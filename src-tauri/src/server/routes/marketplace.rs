use axum::{routing::get, Json, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/popular", get(empty_list))
        .route("/search", get(empty_list))
        .route("/mcp/popular", get(empty_list))
        .route("/mcp/search", get(empty_list))
        .route("/clis/popular", get(empty_list))
        .route("/clis/search", get(empty_list))
}

async fn empty_list() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "items": [],
        "nextOffset": null,
        "hasMore": false,
    }))
}
