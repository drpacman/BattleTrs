# Services — Browser Client Extension

Two new services support the browser client. Both run as independent tokio tasks in the server process.

---

## Service 1: WsRelayService (battletris-server)

**Module**: `battletris-server::ws_relay_service`

### Purpose

Extends the existing `RelayService` (which handles TCP-to-TCP sessions) to also accept WebSocket connections. Both transport types enter a single shared queue and are matched into sessions transparently.

### Architecture

```
TCP Listener (port 7000)
    → TcpConn (impl GameConn)
         \
          --> queue_tx (mpsc::Sender<Box<dyn GameConn>>)
         /                                               \
WS Listener (port 7001 /game)                            --> SessionPairer task
    → WsConn (impl GameConn)                            /     waits for 2 conns
         \                                             /      spawns relay task per pair
          --> queue_tx (mpsc::Sender<Box<dyn GameConn>>)
```

### Session Relay Logic (unchanged from existing)

Each matched pair of `Box<dyn GameConn>` gets a `tokio::spawn` relay task:
```
loop {
    select! {
        msg = conn_a.read_message() => conn_b.write_message(&msg).await,
        msg = conn_b.read_message() => conn_a.write_message(&msg).await,
    }
}
```
No transport awareness needed in relay — `GameConn` trait handles framing differences.

### Responsibilities

- Spawn `WsListener` and `TcpListener` as independent tokio tasks
- Both share the same `mpsc::Sender<Box<dyn GameConn>>`
- `SessionPairer` task reads from `mpsc::Receiver<Box<dyn GameConn>>`:
  - Holds one connection in a slot
  - On second connection: spawn relay task for the pair, clear slot
- On relay task completion: log disconnection; slot remains available for new pairs
- Enforce SECURITY-15: relay tasks must handle `ProtocolError` and `Disconnected` states cleanly without panicking

### Configuration

```rust
pub struct WsRelayConfig {
    pub tcp_addr: SocketAddr,         // default: 0.0.0.0:7000
    pub ws_http_addr: SocketAddr,     // default: 0.0.0.0:7001
    pub allowed_origins: Vec<String>, // required; no wildcard (SECURITY-08)
    pub max_conns_per_ip: usize,      // default: 5 (SECURITY-11)
    pub web_dir: PathBuf,             // default: "./dist" (runtime serving)
    pub max_frame_bytes: usize,       // default: 65536 (SECURITY-13)
}
```

### Error Handling

- Origin validation failure → HTTP 403, no detail (SECURITY-09)
- Rate limit exceeded → HTTP 429, no detail (SECURITY-09)
- Relay read/write error → close both connections; log at WARN level; no panic (SECURITY-15)
- `web_dir` path not found on startup → log ERROR and exit; server does not start without static files

---

## Service 2: HttpStaticService (battletris-server)

**Module**: `battletris-server::http_server` (part of `WsRelayService` router, not a standalone process)

### Purpose

Serves the compiled `battletris-web` WASM artifacts (index.html, .wasm, .js glue, CSS, assets) to browser clients. Runs on the same axum instance as `WsListener` on port 7001.

### Architecture

```
axum Router (port 7001)
    GET /game        → WebSocket upgrade handler (WsListener)
    GET /*           → tower-http ServeDir (web_dir from config)
    ALL routes       → SecurityHeadersMiddleware (tower Layer)
```

### Responsibilities

- Serve all files under `web_dir` verbatim (no template rendering)
- Set correct `Content-Type` headers (handled automatically by `ServeDir`)
- Apply security header middleware to every HTTP response (SECURITY-04):

  | Header | Value |
  |---|---|
  | `Content-Security-Policy` | `default-src 'self'; script-src 'self'; wasm-src 'self'` |
  | `X-Content-Type-Options` | `nosniff` |
  | `X-Frame-Options` | `DENY` |
  | `Referrer-Policy` | `strict-origin-when-cross-origin` |
  | `Cache-Control` | `no-store` (index.html); `max-age=31536000, immutable` (hashed assets) |

- Respond with HTTP 404 and generic body `"Not Found"` for missing files (SECURITY-09)
- Log incoming requests at DEBUG level; log 404s at INFO level; never log request bodies

### Runtime File Serving

Files are NOT embedded in the binary. The server reads `web_dir` from disk at request time (`ServeDir` handles this). This means:
- The WASM client can be updated without recompiling the server
- `web_dir` path must be set before starting (`--web-dir <path>` CLI flag or `BATTLETRIS_WEB_DIR` env var)
- The server will not start if the path does not exist or is not readable

### Integration with WsRelayService

`HttpStaticService` is not a standalone service; it is a set of routes mounted on the same `axum::Router` instance that `WsListener` uses. Both are started together in `main.rs` via a single `axum::serve(listener, router)` call on port 7001.
