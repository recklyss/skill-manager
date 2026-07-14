mod health;
mod settings;
mod skills;
mod mcp;
mod slash_commands;
mod marketplace;
mod scan;

use axum::{routing::get, Router};
use crate::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health_check))
        .merge(settings::router())
        .merge(skills::router())
        .merge(slash_commands::router())
        .nest("/mcp", mcp::router())
        .nest("/marketplace", marketplace::router())
        .nest("/scan", scan::router())
}
