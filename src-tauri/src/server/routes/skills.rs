use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/skills", get(list_skills))
}

async fn list_skills(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let skills = state.skills_queries.all_skills();

    let managed: Vec<serde_json::Value> = skills
        .managed
        .iter()
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect();
    let unmanaged: Vec<serde_json::Value> = skills
        .unmanaged
        .iter()
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect();

    Ok(Json(serde_json::json!({
        "managed": managed,
        "unmanaged": unmanaged,
    })))
}
