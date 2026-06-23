# NFR Design — Unit A: server-ws-http

**Date**: 2026-06-22  
**Basis**: NFR Requirements (revised — LAN deployment context)

Maps each NFR to a concrete implementation pattern. All patterns are already reflected in the Functional Design; this document records the specific choices made and why.

---

## NFR-A-SEC-01 — HTTP Security Headers

**Pattern**: `tower_http::set_header::SetResponseHeaderLayer`

```rust
fn security_headers_layer() -> impl Layer<...> {
    tower::ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
}
```

Applied as a top-level `.layer()` on the axum `Router` so it covers every route — static files, 404s, and the 101 upgrade response.

`overriding` mode is used (not `if_not_present`) so downstream handlers cannot inadvertently omit the headers.

---

## NFR-A-SEC-02 — WebSocket Frame Validation

**Pattern**: Match on `axum::extract::ws::Message` variants before decode.

```rust
async fn read_frame(ws: &mut WebSocket) -> std::io::Result<GameMessage> {
    loop {
        match ws.recv().await {
            Some(Ok(Message::Binary(bytes))) => {
                if bytes.len() > MAX_FRAME_BYTES {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
                }
                return protocol::decode_raw(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
            }
            Some(Ok(Message::Close(_))) | None => {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "connection closed"));
            }
            Some(Ok(_)) => continue, // ping/pong/text — ignore, keep reading
            Some(Err(e)) => return Err(io::Error::new(io::ErrorKind::BrokenPipe, e.to_string())),
        }
    }
}
```

Ping and pong frames are silently skipped (axum handles pong replies automatically). Text frames are skipped rather than erroring — a browser that sends an accidental text frame shouldn't kill the session.

---

## NFR-A-SEC-03 — Bounded Deserialisation

**Pattern**: Named constant checked before every decode call.

```rust
// conn.rs
pub const MAX_FRAME_BYTES: usize = 65_536;
```

Used in both adapters:
- `WsConn::read_frame`: `bytes.len() > MAX_FRAME_BYTES` check (see NFR-A-SEC-02 snippet above)
- `TcpConn::read_frame`: `buf.len() + n > MAX_FRAME_BYTES` check before `buf.extend_from_slice` (matches existing `session.rs` pattern, now centralised in `conn.rs`)

Single definition; both adapters import from `conn::MAX_FRAME_BYTES`. No need to make it configurable — 64 KiB is an appropriate ceiling for a game message.

---

## NFR-A-SEC-04 — Generic Error Responses

**Pattern**: Explicit `into_response()` calls with fixed string literals. No `format!` or `Display` of internal errors in response bodies.

```rust
// 404 fallback
async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

// Unexpected server error — axum's default handler is overridden
async fn internal_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
}
```

Internal errors are logged at WARN/ERROR level server-side using `eprintln!` (consistent with existing server logging style). They are never forwarded to the HTTP response body.

---

## NFR-A-SEC-05 — Error Handling and Resource Release

**Pattern**: Match on `Result` in relay loop; no `.unwrap()` or `.expect()` on network I/O.

```rust
// In run_session — error path
Err(e) => {
    eprintln!("[SESSION] {name} disconnected: {e}");
    let _ = peer.write_frame(&GameMessage::PeerDisconnected).await;
    // ... handle_disconnect logic unchanged
    return;
}
```

No `ConnGuard` needed (rate limiting removed). The `handle_client` task exits cleanly after `run_session` returns, releasing all heap allocations for the connection.

The `tokio::spawn` boundary around each relay task already isolates panics — a panic in one session does not affect the server loop or other sessions. No additional panic-catching infrastructure is required.

Server loop (`run_tcp_listener`, axum `serve`) both run indefinitely; individual session task failures do not propagate upward.

---

## NFR-A-SEC-06 — Dependency Supply Chain

**Pattern**: Standard Cargo workspace dependencies with version pinning.

New entries in `battletris-server/Cargo.toml` use exact major versions matching stable releases:

```toml
async-trait = "0.1"
axum = { version = "0.7", features = ["ws", "macros"] }
tower-http = { version = "0.5", features = ["fs", "set-header"] }
tokio-tungstenite = { version = "0.23" }
futures-util = "0.3"
```

`Cargo.lock` is updated by `cargo build` and committed. No `git` or `path` deps for new crates. Verified by inspecting the diff before committing.

---

## Integration Summary

| NFR | Pattern | Location |
|---|---|---|
| NFR-A-SEC-01 (headers) | `SetResponseHeaderLayer` ×2 | `http_server::security_headers_layer()` |
| NFR-A-SEC-02 (WS validation) | `Message::Binary` match + skip others | `conn::WsConn::read_frame()` |
| NFR-A-SEC-03 (size bound) | `MAX_FRAME_BYTES` const, checked pre-decode | `conn.rs` — both `TcpConn` and `WsConn` |
| NFR-A-SEC-04 (generic errors) | Fixed string literals in response handlers | `http_server::not_found()` and axum error handler |
| NFR-A-SEC-05 (error handling) | `match Err` in relay; `eprintln!` server-side | `session::run_session()` |
| NFR-A-SEC-06 (supply chain) | Version-pinned crates.io deps | `battletris-server/Cargo.toml` |
