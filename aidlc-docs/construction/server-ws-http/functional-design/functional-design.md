# Functional Design — Unit A: server-ws-http

**Date**: 2026-06-22  
**Crate**: `battletris-server`  
**Status**: PENDING code generation approval

---

## Scope Summary

The existing server accepts TCP connections only. This design makes it accept both TCP and WebSocket connections by introducing a `GameConn` transport abstraction. The session relay logic in `session.rs` is refactored to use the abstraction — the relay algorithm itself is unchanged.

Two new modules are added: `ws_listener.rs` (WS upgrade + rate limiting + origin validation) and `http_server.rs` (axum static file serving + security headers). Both share one axum `Router` and bind on port 7001.

---

## 1. New Module: `conn.rs` — GameConn Trait and Adapters

### Trait Definition

```rust
use async_trait::async_trait;
use battletris_engine::protocol::GameMessage;

#[async_trait]
pub trait GameConn: Send + 'static {
    async fn read_frame(&mut self) -> std::io::Result<GameMessage>;
    async fn write_frame(&mut self, msg: &GameMessage) -> std::io::Result<()>;
}
```

`async_trait` proc macro is used to work around Rust's current limitation that async fn in traits cannot be used as `dyn Trait`. It generates a boxed-future impl automatically.

### TcpConn — Adapter for Existing TCP Connections

```rust
pub struct TcpConn {
    stream: tokio::net::TcpStream,
    buf: Vec<u8>,
}
```

`read_frame`: Reads raw bytes from `stream` into `buf` until `protocol::decode` succeeds. Mirrors the existing `read_frame` in `session.rs` (which is removed after refactor). Enforces `MAX_FRAME_BYTES = 65536` limit before extending buf.

`write_frame`: Calls `protocol::encode(msg)` and `stream.write_all(&bytes)`. Mirrors existing `write_frame` in `session.rs`.

### WsConn — Adapter for WebSocket Connections

```rust
pub struct WsConn {
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
}
```

`read_frame`:
1. Await next message from the WebSocket stream (`ws.next().await`)
2. Reject non-binary frames (text, ping, pong, close) — return `InvalidData` error
3. Check frame payload length ≤ `MAX_FRAME_BYTES` (SECURITY-13)
4. Call `protocol::decode_raw(&bytes)` (a new zero-length-prefix variant, see Protocol note below) to deserialise a `GameMessage`
5. Return the message or `InvalidData` on decode failure

`write_frame`:
1. Call `protocol::encode_raw(msg)` → `Vec<u8>` (bincode only, no length prefix)
2. Send as a `tokio_tungstenite::Message::Binary(bytes)` over the WebSocket

**Protocol note**: TCP uses length-prefixed framing (`protocol::encode` / `protocol::decode` + frame_len). WebSocket provides its own frame boundaries, so the WS path uses raw bincode without the length prefix. This requires two small functions in `battletris_engine::protocol`:
- `pub fn encode_raw(msg: &GameMessage) -> Result<Vec<u8>, ProtocolError>` — bincode only
- `pub fn decode_raw(buf: &[u8]) -> Result<GameMessage, ProtocolError>` — bincode only

These additions are additive and non-breaking (existing TCP callers continue to use `encode`/`decode`).

---

## 2. Refactored `server.rs` — Transport-Agnostic Handshake

### Changes to `handle_client`

Signature changes from `TcpStream` to `Box<dyn GameConn>`:

```rust
async fn handle_client(
    mut conn: Box<dyn GameConn>,
    peer_addr: std::net::SocketAddr,
    db: Arc<Mutex<PlayerDb>>,
    pending: Arc<Mutex<Option<(Box<dyn GameConn>, String)>>>,
    active_names: Arc<Mutex<HashSet<String>>>,
)
```

`read_hello` is updated to call `conn.read_frame()` instead of reading raw TCP bytes:

```rust
async fn read_hello(conn: &mut Box<dyn GameConn>) -> Option<String> {
    match conn.read_frame().await {
        Ok(GameMessage::Hello { name }) => Some(name),
        _ => None,
    }
}
```

`write_msg` is replaced by `conn.write_frame(msg)` inline.

`pending` slot type changes from `Arc<Mutex<Option<(TcpStream, String)>>>` to `Arc<Mutex<Option<(Box<dyn GameConn>, String)>>>`.

### TCP Listener Task (in `run_server`)

Creates a `TcpConn` wrapper for each accepted `TcpStream` and calls `handle_client` — same flow as today, just wrapped.

### WS Listener Task (new, in `run_server`)

`WsListener::run()` is called as a parallel tokio task. It sends each accepted `Box<dyn WsConn>` wrapped as `Box<dyn GameConn>` into `handle_client` via the same shared `pending` / `active_names` state.

---

## 3. Refactored `session.rs` — Transport-Agnostic Relay

### `run_session` Signature

```rust
pub async fn run_session(
    mut a: Box<dyn GameConn>,
    name_a: String,
    mut b: Box<dyn GameConn>,
    name_b: String,
    db: Arc<Mutex<PlayerDb>>,
)
```

The `tokio::select!` relay loop is unchanged in logic:

```rust
loop {
    tokio::select! {
        result = a.read_frame() => { /* handle_message or disconnect */ }
        result = b.read_frame() => { /* handle_message or disconnect */ }
    }
}
```

`handle_message` and `handle_disconnect` signatures change from `&mut TcpStream` to `&mut Box<dyn GameConn>`. Calls to `write_frame()` become `conn.write_frame(msg).await`.

Local `read_frame` and `write_frame` free functions in `session.rs` are **removed** — their logic now lives in `TcpConn`'s `GameConn` impl.

---

## 4. New Module: `ws_listener.rs` — WebSocket Upgrade Handler

### State

No per-connection tracking state required (rate limiting removed — LAN deployment). `WsListenerState` is not needed as a struct; the handler is a plain axum function receiving shared `pending`/`active_names`/`db` via `axum::extract::State`.

### axum Handler

```rust
async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    State(shared): State<Arc<SharedState>>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        let conn: Box<dyn GameConn> = Box::new(WsConn { ws: socket });
        handle_client(conn, peer, shared).await;
    })
}
```

`SharedState` holds `pending`, `active_names`, and `db` — same state shared with the TCP listener task.

---

## 5. New Module: `http_server.rs` — Static File Serving

### Router

```rust
pub fn build_router(web_dir: PathBuf, shared: Arc<SharedState>) -> Router {
    Router::new()
        .route("/game", get(ws_upgrade_handler))
        .fallback_service(ServeDir::new(web_dir).not_found_service(not_found_handler))
        .layer(security_headers_layer())
        .with_state(shared)
        .layer(ConnectInfoLayer::<SocketAddr>::new())
}
```

### Security Headers Layer

Two headers applied to all responses via chained `SetResponseHeaderLayer`:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
```

HSTS and CSP are omitted — no TLS, LAN deployment (see NFR requirements revision).

### 404 Handler

Returns HTTP 404 with body `"Not Found"` — no path, no internal detail (SECURITY-09).

---

## 6. Updated `main.rs`

### CLI Changes

```rust
Command::Serve {
    /// TCP port for desktop clients.
    #[arg(long, default_value_t = 7000)]
    port: u16,
    /// HTTP + WebSocket port for browser clients.
    #[arg(long, default_value_t = 7001)]
    web_port: u16,
    /// Path to compiled battletris-web dist/ directory.
    #[arg(long, default_value = "./dist")]
    web_dir: PathBuf,
}
```

### Task Layout

```rust
rt.block_on(async {
    // Shared state
    let shared = Arc::new(SharedState {
        pending: Mutex::new(None),
        active_names: Mutex::new(HashSet::new()),
        db,
    });

    // Task 1: TCP listener (existing, now wraps TcpStream in TcpConn)
    let tcp_task = tokio::spawn(run_tcp_listener(port, shared.clone()));

    // Task 2: HTTP + WebSocket listener (new)
    let router = build_router(web_dir, shared.clone());
    let ws_task = tokio::spawn(
        axum::serve(
            TcpListener::bind(format!("0.0.0.0:{web_port}")).await.unwrap(),
            router,
        ).into_future()
    );

    tokio::join!(tcp_task, ws_task);
});
```

---

## 7. `battletris-engine` Protocol Additions

Two additive, non-breaking functions in `battletris_engine::protocol`:

```rust
/// Encode a GameMessage to raw bincode bytes (no length prefix).
/// Used by WsConn where WebSocket provides frame boundaries.
pub fn encode_raw(msg: &GameMessage) -> Result<Vec<u8>, ProtocolError> {
    bincode::serialize(msg).map_err(|_| ProtocolError::Encode)
}

/// Decode a GameMessage from raw bincode bytes (no length prefix, no offset).
/// Used by WsConn.
pub fn decode_raw(buf: &[u8]) -> Result<GameMessage, ProtocolError> {
    bincode::deserialize(buf).map_err(|_| ProtocolError::Decode)
}
```

Existing `encode` / `decode` / `frame_len` are untouched.

---

## 8. Dependency Changes

### `battletris-server/Cargo.toml`

```toml
async-trait = "0.1"
axum = { version = "0.7", features = ["ws", "macros"] }
tower-http = { version = "0.5", features = ["fs", "set-header"] }
tokio-tungstenite = { version = "0.23" }
futures-util = "0.3"   # for WS stream .next()
```

---

## 9. Test Coverage Plan

| Test | Type | Location |
|---|---|---|
| `encode_raw` / `decode_raw` roundtrip | unit | `battletris-engine::protocol` tests |
| `TcpConn` read_frame / write_frame | unit | `battletris-server::conn` tests |
| `WsConn` read_frame / write_frame with mock WS | unit | `battletris-server::conn` tests |
| WS upgrade accepted: message roundtrip | integration | `battletris-server` tests (axum `TestClient`) |
| HTTP GET static file: file present → 200 with security headers | integration | `battletris-server` tests |
| HTTP GET static file: file missing → 404 `"Not Found"` | integration | `battletris-server` tests |
| Mixed TCP+WS session: both players relay correctly | integration | `battletris-server` tests |
