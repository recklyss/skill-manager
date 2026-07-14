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
        .merge(mcp::router())
        .merge(slash_commands::router())
        .merge(marketplace::router())
        .merge(scan::router())
}
