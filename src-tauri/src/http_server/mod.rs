// HTTP server for serving test patterns to CasparCG
// CasparCG's CEF browser needs HTTP access to templates - it cannot use file:// or asset:// protocols

use axum::{
    Router,
    routing::get,
    response::IntoResponse,
    http::{StatusCode, header},
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};

/// State for the test HTTP server
#[derive(Debug)]
pub struct TestServer {
    /// The port the server is running on (None if not running)
    port: Option<u16>,
    /// Handle to shut down the server
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl TestServer {
    pub fn new() -> Self {
        Self {
            port: None,
            shutdown_tx: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.port.is_some()
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn url(&self) -> Option<String> {
        self.port.map(|p| format!("http://127.0.0.1:{}", p))
    }
}

impl Default for TestServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Global test server state
pub type TestServerState = Arc<RwLock<TestServer>>;

/// Create a new test server state
pub fn create_test_server_state() -> TestServerState {
    Arc::new(RwLock::new(TestServer::new()))
}

/// Start the test HTTP server
///
/// Serves files from the test/ directory at the application root.
/// Returns the port the server is running on.
pub async fn start_server(
    state: TestServerState,
    preferred_port: Option<u16>,
    test_dir: PathBuf,
) -> Result<u16, String> {
    // Check if already running
    {
        let server = state.read().await;
        if server.is_running() {
            return Err("Test server is already running".to_string());
        }
    }

    // Verify test directory exists
    if !test_dir.exists() {
        return Err(format!("Test directory not found: {}", test_dir.display()));
    }

    // Build the router with CORS enabled for CasparCG access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Serve static files from test directory
    let app = Router::new()
        .nest_service("/", ServeDir::new(&test_dir))
        .layer(cors);

    // Try the preferred port, then fallback to random
    let port = preferred_port.unwrap_or(9966);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(_) => {
            // Fallback to any available port
            let addr = SocketAddr::from(([127, 0, 0, 1], 0));
            tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| format!("Failed to bind to any port: {}", e))?
        }
    };

    let actual_port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn the server
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    // Update state
    {
        let mut server = state.write().await;
        server.port = Some(actual_port);
        server.shutdown_tx = Some(shutdown_tx);
    }

    eprintln!("[test-server] Started on port {}", actual_port);
    eprintln!("[test-server] Serving files from: {}", test_dir.display());

    Ok(actual_port)
}

/// Stop the test HTTP server
pub async fn stop_server(state: TestServerState) -> Result<(), String> {
    let mut server = state.write().await;

    if let Some(tx) = server.shutdown_tx.take() {
        let _ = tx.send(());
        server.port = None;
        eprintln!("[test-server] Stopped");
        Ok(())
    } else {
        Err("Test server is not running".to_string())
    }
}

/// Get the URL of the test server
pub async fn get_server_url(state: TestServerState) -> Option<String> {
    let server = state.read().await;
    server.url()
}

/// Get the URL for the key/fill identifier template
pub fn get_test_pattern_url(base_url: &str, channel: u32, mode: &str) -> String {
    format!(
        "{}/key-fill-identifier.html?mode={}&id={}",
        base_url, mode, channel
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_url() {
        let url = get_test_pattern_url("http://127.0.0.1:9966", 1, "fill");
        assert_eq!(url, "http://127.0.0.1:9966/key-fill-identifier.html?mode=fill&id=1");
    }
}
