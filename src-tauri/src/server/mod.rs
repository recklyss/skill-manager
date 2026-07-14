mod routes;

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::AppState;

pub struct ServerHandle {
    shutdown: Arc<AtomicBool>,
    ready: mpsc::Receiver<()>,
}

impl ServerHandle {
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }

    /// Block until the axum server has bound its listen socket.
    pub fn wait_ready(&self) {
        self.ready
            .recv()
            .expect("skill-manager server failed to start");
    }
}

/// Start the axum HTTP server, returning a handle that can shut it down.
pub fn start(addr: SocketAddr, state: AppState) -> ServerHandle {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_flag = shutdown.clone();
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);

    let app = build_router(state);

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        rt.block_on(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("failed to bind server socket");

            let _ = ready_tx.send(());

            println!("skill-manager server listening on {}", addr);

            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
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

    ServerHandle {
        shutdown,
        ready: ready_rx,
    }
}

/// API-only router (no SPA fallback) for integration tests.
pub fn api_router(state: AppState) -> Router {
    routes::api_router().with_state(state)
}

fn build_router(state: AppState) -> Router {
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
