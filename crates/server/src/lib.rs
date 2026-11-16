use std::io;
use std::net::{Ipv4Addr, SocketAddr};

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use transf_orm_core::config::GlobalConfig;

/// Flags specific to the server mode.
#[derive(clap::Args, Debug)]
pub struct ServerArgs {
    /// Port to listen on in server mode (0 = any free port)
    #[arg(long, default_value_t = 3000, requires = "serveur")]
    pub port: u16,
}

/// Runs the server mode until the process receives Ctrl+C.
///
/// Listens on `127.0.0.1`; `--ip` / `--host` come with #91. Once the socket
/// is bound, prints `Listening on http://<address>` on stderr with the real
/// port, which matters when `--port 0` lets the OS pick one.
///
/// # Arguments
///
/// * `config` - Options from the global flags
/// * `args` - Server-specific flags
///
/// # Errors
///
/// Returns an error when the tokio runtime cannot start or the port cannot
/// be bound (e.g. already in use).
pub fn run(config: &GlobalConfig, args: ServerArgs) -> io::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve(config, args))
}

async fn serve(config: &GlobalConfig, args: ServerArgs) -> io::Result<()> {
    if config.verbose {
        eprintln!("[verbose] Binding 127.0.0.1:{}", args.port);
    }
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, args.port))).await?;
    let addr = listener.local_addr()?;
    tracing::info!(%addr, "server listening");
    eprintln!("Listening on http://{addr}");

    axum::serve(listener, router())
        .with_graceful_shutdown(shutdown_signal())
        .await
}

/// Builds the HTTP routes of the server.
///
/// Exposed separately from [`run`] so it can be tested without binding a port.
pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

/// `GET /health` — liveness probe, always `200 {"status": "ok"}`.
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// Resolves on Ctrl+C so in-flight requests can finish before exiting.
async fn shutdown_signal() {
    if tokio::signal::ctrl_c().await.is_ok() {
        tracing::info!("shutting down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_200() {
        let response = router()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
    }
}
