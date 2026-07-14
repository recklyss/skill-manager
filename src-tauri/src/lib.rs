mod server;
mod paths;
mod harness;
mod skills;

use std::net::TcpListener;
use std::sync::OnceLock;
use tauri::Manager;

use paths::AppPaths;
use harness::HarnessKernel;
use skills::store::SkillStore;
use skills::read_models::SkillsReadModelService;
use skills::queries::SkillsQueryService;

static SERVER_PORT: OnceLock<u16> = OnceLock::new();

/// Shared application state injected into axum routes.
#[derive(Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub harness_kernel: HarnessKernel,
    pub skills_queries: SkillsQueryService,
}

/// Returns the base URL of the embedded axum server.
#[tauri::command]
fn get_server_url() -> String {
    let port = SERVER_PORT.get().copied().unwrap_or(0);
    format!("http://127.0.0.1:{}", port)
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

    // Bind to an OS-assigned port.
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind server socket");
    let port = listener.local_addr().unwrap().port();
    SERVER_PORT.set(port).ok();

    let server_handle = server::start(listener, state);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_server_url])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    server_handle.shutdown();
}
