# Application Design — Browser Client Extension

**Feature**: Browser-based BattleTris client with cross-play support  
**Date**: 2026-06-22  
**Status**: Application Design COMPLETED — pending Units Generation

---

## Design Summary

The browser client is implemented as a new WASM crate (`battletris-web`) that reuses `battletris-engine` entirely unchanged, paired with a dual-transport upgrade to `battletris-server` (TCP retained on 7000; WebSocket + HTTP added on 7001).

### Architecture Overview

```
Existing (unchanged):
  battletris-client (SDL2 desktop)
      → TCP port 7000
          → battletris-server relay

New:
  battletris-web (WASM browser)
      → WebSocket port 7001/game
          → WsListener (origin check, rate limit)
              → WsConn : GameConn
                  → SessionPairer (shared with TCP)
                      → Relay task (transport-agnostic)
  
  battletris-server (port 7001, HTTP)
      → axum ServeDir(dist/)
      → Security headers middleware
```

---

## Design Decisions

| # | Question | Decision | Rationale |
|---|---|---|---|
| Q1 | Server ports | Separate: TCP 7000, HTTP+WS 7001 | Zero impact on existing desktop client; no byte-sniffing complexity |
| Q2 | Static file delivery | Runtime disk serving (axum ServeDir) | Decoupled build pipeline; WASM updates don't require server recompile |
| Q3 | HTTP/WS library | axum + tower-http + tokio-tungstenite | Idiomatic tokio; composable middleware; production-grade |
| Q4 | Transport abstraction | `GameConn` trait object | Relay logic unchanged; both transports work identically from session perspective |
| Q5 | WASM game loop | requestAnimationFrame with `thread_local!` App | Standard browser pattern; ~60fps; battery efficient; safe for single-threaded WASM |

---

## Component Inventory

| # | Component | Crate | Status | Responsibility |
|---|---|---|---|---|
| 1 | Engine | battletris-engine | Unchanged | Game state, physics, piece logic |
| 2 | AI (Ernie) | battletris-engine | Unchanged | Computer opponent |
| 3 | Protocol | battletris-engine | Unchanged | GameMessage, bincode framing |
| 4 | Renderer | battletris-client | Unchanged | SDL2 desktop rendering |
| 5 | NetworkClient | battletris-client | Unchanged | TCP connection to server |
| 6 | Server | battletris-server | Extended | Relay + PlayerDb + ELO; now uses GameConn trait |
| 7 | WsListener | battletris-server | **NEW** | WS upgrade; origin allowlist; rate limiting |
| 8 | HttpServer | battletris-server | **NEW** | Static file serving; security headers |
| 9 | WasmApp | battletris-web | **NEW** | WASM entry; rAF tick loop; game state management |
| 10 | WsTransport | battletris-web | **NEW** | WebSocket client; bincode send/receive |
| 11 | CanvasRenderer | battletris-web | **NEW** | Canvas 2D rendering of all game screens |
| 12 | InputHandler | battletris-web | **NEW** | DOM keydown → PlayerInput mapping |

---

## Service Inventory

| # | Service | Process | Purpose |
|---|---|---|---|
| 1 | WsRelayService | battletris-server | Shared session queue; matches TCP and WS players; spawns relay tasks |
| 2 | HttpStaticService | battletris-server | Serves dist/ files on port 7001 with security headers |

---

## Security Compliance Summary

Security extension enabled (Q10=A). All applicable rules addressed in this design:

| Rule | Status | Design Artifact |
|---|---|---|
| SECURITY-01 | N/A | No new data stores |
| SECURITY-02 | N/A | No API gateway / load balancer |
| SECURITY-03 | Compliant | Error handling specified in WsListener, relay task, HttpServer |
| SECURITY-04 | Compliant | HttpServer security headers layer (see services.md) |
| SECURITY-05 | Compliant | WsListener validates frame type and routes only binary frames to bincode |
| SECURITY-06 | N/A | No IAM |
| SECURITY-07 | N/A | No cloud networking |
| SECURITY-08 | Compliant | WsListener Origin allowlist; 403 on failure; no wildcard |
| SECURITY-09 | Compliant | WsListener and HttpServer return generic errors; no stack traces |
| SECURITY-10 | Compliant | Cargo.lock committed; new deps from crates.io only; verified in Build & Test |
| SECURITY-11 | Compliant | WsListener per-IP connection count enforced before upgrade |
| SECURITY-12 | N/A | No user authentication passwords |
| SECURITY-13 | Compliant | WsListener checks frame byte length before bincode decode; configurable limit |
| SECURITY-14 | N/A | No cloud monitoring stack |
| SECURITY-15 | Compliant | Relay tasks handle ProtocolError without panic; connections closed on error |

---

## Key Interface: GameConn Trait

The single most critical design decision — it gates all construction work for Unit A:

```rust
pub trait GameConn: Send + 'static {
    fn read_message<'a>(&'a mut self)
        -> Pin<Box<dyn Future<Output = Result<GameMessage, ProtocolError>> + Send + 'a>>;
    fn write_message<'a>(&'a mut self, msg: &'a GameMessage)
        -> Pin<Box<dyn Future<Output = Result<(), ProtocolError>> + Send + 'a>>;
}
```

`TcpConn` wraps a `tokio::net::TcpStream` with length-prefixed bincode framing (existing behaviour).  
`WsConn` wraps a `tokio_tungstenite::WebSocketStream`, encoding/decoding GameMessage as binary WebSocket frames using the same bincode serialiser.

The relay task never needs to know which transport it is working with.

---

## Workspace Changes Required

### Cargo.toml (workspace root)

```toml
members = [
    "battletris-engine",
    "battletris-client",
    "battletris-server",
    "battletris-web",   # ADD
]
```

### battletris-engine

Add WASM target dependency only:
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```
No other changes. All game logic, AI, and protocol code compiles to WASM as-is.

### battletris-server

Add HTTP + WS dependencies:
```toml
axum = { version = "0.7", features = ["ws"] }
tower-http = { version = "0.5", features = ["fs", "set-header"] }
tokio-tungstenite = { version = "0.23" }
```

### battletris-web (new crate)

See component-dependency.md for full Cargo.toml.

---

## Units of Work

| Unit | Crate(s) | Stages |
|---|---|---|
| Unit A — Server WS+HTTP | battletris-server | Functional Design → NFR Requirements → NFR Design → Code Generation |
| Unit B — WASM Client | battletris-web | Functional Design → Code Generation |

Followed by: Build and Test (trunk build, cargo test, manual browser integration)

---

## Build Tooling

| Tool | Purpose | Install |
|---|---|---|
| `trunk` | WASM build + bundling for battletris-web | `cargo install trunk` |
| `wasm-bindgen-cli` | JS glue generation (installed by trunk automatically) | via trunk |
| `wasm-opt` | WASM binary size optimisation (release builds) | via trunk |

**Build order**:
1. `trunk build --release` in `battletris-web/` → produces `dist/`
2. `cargo build --release -p battletris-server -- --web-dir battletris-web/dist`

**Dev workflow**:
1. `trunk serve` in `battletris-web/` → hot-reload WASM at localhost:8080
2. `cargo run -p battletris-server -- --tcp-port 7000 --web-port 7001 --web-dir battletris-web/dist`
