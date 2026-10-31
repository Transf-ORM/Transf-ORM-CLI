use std::any::Any;
use std::future::Future;
use std::io;
use std::net::{IpAddr, Ipv4Addr};

use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use transf_orm_core::config::GlobalConfig;

mod error;

pub use error::ApiError;

/// Flags specific to the server mode.
#[derive(clap::Args, Debug)]
pub struct ServerArgs {
    /// IP address to listen on in server mode
    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST), requires = "serveur")]
    pub ip: IpAddr,

    /// Port to listen on in server mode (0 = any free port)
    #[arg(long, default_value_t = 3000, requires = "serveur")]
    pub port: u16,

    /// Public hostname of the server, used to build its public URL
    #[arg(long, requires = "serveur")]
    pub host: Option<String>,
}

/// Network settings of the server mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    /// Address the listener binds to.
    pub ip: IpAddr,
    /// Port the listener binds to; `0` lets the OS pick a free one.
    pub port: u16,
    /// Public hostname, when the server is reached through another name
    /// than its bind address (e.g. behind a reverse proxy).
    pub host: Option<String>,
}

impl From<ServerArgs> for ServerConfig {
    fn from(args: ServerArgs) -> Self {
        Self {
            ip: args.ip,
            port: args.port,
            host: args.host,
        }
    }
}

/// Runs the server mode until the process receives Ctrl+C.
///
/// Once the socket is bound, prints `Listening on http://<address>` on
/// stderr with the real port, which matters when `--port 0` lets the OS pick
/// one, then `Public URL: http://<host>:<port>` when `--host` is set.
///
/// # Arguments
///
/// * `config` - Options from the global flags
/// * `server` - Network settings of the listener
///
/// # Errors
///
/// Returns an error when the tokio runtime cannot start or the address
/// cannot be bound (e.g. port already in use).
pub fn run(config: &GlobalConfig, server: ServerConfig) -> io::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            if config.verbose {
                eprintln!("[verbose] Binding {}:{}", server.ip, server.port);
            }
            let listener = TcpListener::bind((server.ip, server.port)).await?;
            let addr = listener.local_addr()?;
            tracing::info!(%addr, "server listening");
            eprintln!("Listening on http://{addr}");
            if let Some(host) = &server.host {
                eprintln!("Public URL: http://{host}:{}", addr.port());
            }

            serve(listener, shutdown_signal()).await
        })
}

/// Serves the API on an already bound listener until `shutdown` resolves.
///
/// In-flight requests are allowed to finish before it returns.
///
/// # Errors
///
/// Returns an error when accepting connections fails.
pub async fn serve<F>(listener: TcpListener, shutdown: F) -> io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    axum::serve(listener, router())
        .with_graceful_shutdown(shutdown)
        .await
}

/// Builds the HTTP routes of the server, with their middleware.
///
/// Every error, including an unknown route or a panicking handler, is
/// answered with an [`ApiError`] JSON body. Each response is logged through
/// `tracing` (method, URI, status, latency), visible with `--debug info`.
pub fn router() -> Router {
    with_middleware(Router::new().route("/health", get(health)))
}

/// Adds the JSON fallbacks and the middleware to `routes`.
///
/// Layers only wrap the routes that exist when they are added, so this must
/// run after every route is registered.
fn with_middleware(routes: Router) -> Router {
    routes
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .layer(CatchPanicLayer::custom(panic_response))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
}

/// `GET /health` — liveness probe, always `200 {"status": "ok"}`.
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn not_found(method: Method, uri: Uri) -> ApiError {
    ApiError::not_found(format!("No route for {method} {}", uri.path()))
}

async fn method_not_allowed(method: Method, uri: Uri) -> ApiError {
    ApiError::method_not_allowed(format!("{method} is not allowed on {}", uri.path()))
}

fn panic_response(panic: Box<dyn Any + Send + 'static>) -> Response {
    let detail = panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic");
    tracing::error!(panic = detail, "handler panicked");
    ApiError::internal("Internal server error").into_response()
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

    async fn call(router: Router, method: Method, path: &str) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    #[tokio::test]
    async fn health_returns_200() {
        let (status, body) = call(router(), Method::GET, "/health").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({ "status": "ok" }));
    }

    #[tokio::test]
    async fn unknown_route_returns_404_json() {
        let (status, body) = call(router(), Method::GET, "/nope").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(
            body,
            json!({ "error": "not_found", "message": "No route for GET /nope" })
        );
    }

    #[tokio::test]
    async fn wrong_method_returns_405_json() {
        let (status, body) = call(router(), Method::POST, "/health").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(body["error"], "method_not_allowed");
    }

    async fn boom() {
        panic!("boom");
    }

    #[tokio::test]
    async fn panic_returns_500_json() {
        let router = with_middleware(Router::new().route("/boom", get(boom)));
        let (status, body) = call(router, Method::GET, "/boom").await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            body,
            json!({ "error": "internal", "message": "Internal server error" })
        );
    }

    #[test]
    fn args_convert_to_config() {
        let args = ServerArgs {
            ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 8080,
            host: Some("api.example.com".into()),
        };
        assert_eq!(
            ServerConfig::from(args),
            ServerConfig {
                ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                port: 8080,
                host: Some("api.example.com".into()),
            }
        );
    }
}
