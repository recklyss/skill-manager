use axum::{extract::State, routing::get, Json, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/skills", get(list_skills))
}

async fn list_skills(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let page = state.skills_queries.page_response();
    Json(serde_json::to_value(page).unwrap_or_default())
}
