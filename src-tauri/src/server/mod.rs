mod routes;

use std::net::TcpListener as StdTcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;

use crate::AppState;

pub struct ServerHandle {
    shutdown: Arc<AtomicBool>,
}

impl ServerHandle {
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

/// Start the axum HTTP server on the given listener, returning a handle
/// that can be used to shut it down.
pub fn start(listener: StdTcpListener, state: AppState) -> ServerHandle {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_flag = shutdown.clone();

    let app = build_router(state);

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        rt.block_on(async move {
            info!("skill-manager server listening on {}", listener.local_addr().unwrap());

            let tokio_listener = tokio::net::TcpListener::from_std(listener)
                .expect("failed to convert listener to tokio");
            axum::serve(tokio_listener, app)
                .with_graceful_shutdown(async move {
                    // Wait until shutdown is signaled. In a production app we'd
                    // also listen for Ctrl-C, but the Tauri lifecycle manages this.
                    loop {
                        if shutdown_flag.load(Ordering::SeqCst) {
                            break;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                })
                .await
                .expect("server error");
        });
    });

    ServerHandle { shutdown }
}

fn build_router(state: AppState) -> Router {
    // In debug mode, the frontend is served by the Vite dev server.
    // In release, we serve the built assets from frontend/dist.
    let frontend_dist =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../frontend/dist");

    let api_routes = routes::api_router().with_state(state);

    let app = Router::new()
        .nest("/api", api_routes)
        .layer(CorsLayer::permissive());

    if frontend_dist.exists() {
        app.fallback_service(ServeDir::new(frontend_dist).append_index_html_on_directories(true))
    } else {
        app
    }
}
