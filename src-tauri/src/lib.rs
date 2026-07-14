mod server;
mod paths;
mod harness;
mod skills;

use std::net::TcpListener;

use paths::AppPaths;
use harness::HarnessKernel;
use skills::store::SkillStore;
use skills::read_models::SkillsReadModelService;
use skills::queries::SkillsQueryService;

const SERVER_PORT: u16 = 18000;

/// Shared application state injected into axum routes.
#[derive(Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub harness_kernel: HarnessKernel,
    pub skills_queries: SkillsQueryService,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let paths = AppPaths::resolve();
    let kernel = HarnessKernel::new();
    let skill_store = SkillStore::new(&paths);
    let _ = skill_store.init();
    let skills_read_models = SkillsReadModelService::new(skill_store, kernel.clone());
    let skills_queries = SkillsQueryService::new(skills_read_models);

    let state = AppState {
        paths,
        harness_kernel: kernel,
        skills_queries,
    };

    // Bind to a fixed port so the frontend always knows where the API is.
    let listener = TcpListener::bind(format!("127.0.0.1:{}", SERVER_PORT))
        .expect("failed to bind server socket");

    let server_handle = server::start(listener, state);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    server_handle.shutdown();
}
