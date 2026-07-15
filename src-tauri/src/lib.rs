pub mod server;
pub mod paths;
pub mod harness;
pub mod skills;
pub mod error;
mod db;
mod mcp;
mod slash_commands;
mod scan;
mod marketplace;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;

use paths::AppPaths;
use harness::{HarnessKernelService, HarnessSupportStore};
use skills::mutations::SkillsMutationService;
use skills::queries::SkillsQueryService;
use skills::read_models::SkillsReadModelService;
use skills::source_fetch::SourceFetchService;
use skills::store::SkillStore;
use db::Database;
use mcp::McpServices;
use slash_commands::SlashCommandServices;
use scan::ScanServices;
use marketplace::MarketplaceServices;

const SERVER_PORT: u16 = 18000;
const SERVER_ADDR: &str = "127.0.0.1";

/// Shared application state injected into axum routes.
#[derive(Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub harness_kernel: HarnessKernelService,
    pub skills_queries: SkillsQueryService,
    pub skills_mutations: SkillsMutationService,
    pub mcp: McpServices,
    pub slash_commands: SlashCommandServices,
    pub scan: ScanServices,
    pub marketplace: MarketplaceServices,
}

/// Build application state for a resolved data directory layout.
pub fn build_app_state(paths: AppPaths) -> AppState {
    build_app_state_with_env(paths, std::env::vars().collect())
}

/// Build application state with explicit environment overrides (integration tests).
pub fn build_app_state_with_env(
    paths: AppPaths,
    env: HashMap<String, String>,
) -> AppState {
    let support_store = HarnessSupportStore::new(paths.settings_path.clone());
    let kernel = HarnessKernelService::from_environment(Some(env), support_store);
    let skill_store = SkillStore::from_paths(&paths);
    let _ = skill_store.init();
    let skills_read_models = SkillsReadModelService::new(skill_store, kernel.clone());
    let source_fetcher = SourceFetchService::new();
    let skills_queries = SkillsQueryService::new(skills_read_models.clone(), source_fetcher.clone());
    let skills_mutations =
        SkillsMutationService::new(skills_read_models, skills_queries.clone(), source_fetcher);

    let db = Arc::new(
        Database::open(&paths.db_path).expect("failed to open database"),
    );
    let marketplace = MarketplaceServices::new(&paths);
    let mcp = McpServices::new(&paths, &kernel, &marketplace.mcp);
    let slash_commands = SlashCommandServices::new(&paths, &kernel);
    let scan = ScanServices::new(db, kernel.clone(), skills_queries.clone());

    AppState {
        paths,
        harness_kernel: kernel,
        skills_queries,
        skills_mutations,
        mcp,
        slash_commands,
        scan,
        marketplace,
    }
}

/// Expose the API router for integration tests (no static file fallback).
pub fn api_router(state: AppState) -> Router {
    Router::new().nest("/api", server::api_router(state))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = build_app_state(AppPaths::resolve());

    let addr: SocketAddr = format!("{}:{}", SERVER_ADDR, SERVER_PORT)
        .parse()
        .expect("invalid server address");
    let server_handle = server::start(addr, state);
    server_handle.wait_ready();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    server_handle.shutdown();
}
