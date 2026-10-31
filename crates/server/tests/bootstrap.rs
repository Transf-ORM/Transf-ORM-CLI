use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// Starts the server on an ephemeral port; send on the returned channel to
/// stop it.
async fn start() -> (
    SocketAddr,
    oneshot::Sender<()>,
    JoinHandle<std::io::Result<()>>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stopped) = oneshot::channel();
    let server = tokio::spawn(transf_orm_server::serve(listener, async {
        let _ = stopped.await;
    }));
    (addr, stop, server)
}

/// Sends a bodyless request and returns the raw HTTP response.
async fn request(addr: SocketAddr, method: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request =
        format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

#[tokio::test]
async fn bootstrap_serves_health() {
    let (addr, stop, server) = start().await;

    let response = request(addr, "GET", "/health").await;
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.ends_with(r#"{"status":"ok"}"#), "{response}");

    stop.send(()).unwrap();
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn bootstrap_unknown_route_returns_404_json() {
    let (addr, stop, server) = start().await;

    let response = request(addr, "GET", "/nope").await;
    assert!(response.starts_with("HTTP/1.1 404 Not Found"), "{response}");
    assert!(
        response.contains("content-type: application/json"),
        "{response}"
    );
    assert!(
        response.ends_with(r#"{"error":"not_found","message":"No route for GET /nope"}"#),
        "{response}"
    );

    stop.send(()).unwrap();
    server.await.unwrap().unwrap();
}
