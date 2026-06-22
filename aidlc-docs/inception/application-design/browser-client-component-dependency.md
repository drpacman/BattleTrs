# Component Dependency — Browser Client Extension

---

## Updated Crate Dependency Graph

```
                    ┌─────────────────────────────┐
                    │     battletris-engine (lib)  │
                    │  Engine, AI, Protocol comps  │
                    │  Pure Rust — no platform deps│
                    └──────────────┬──────────────┘
                                   │  (dep)
            ┌──────────────────────┼──────────────────────┐
            │                      │                       │
            ▼                      ▼                       ▼
┌──────────────────┐  ┌───────────────────────┐  ┌──────────────────────┐
│battletris-client │  │  battletris-server    │  │  battletris-web      │
│   (bin, SDL2)    │  │  (bin, tokio + axum)  │  │  (cdylib, WASM)      │
│                  │  │                       │  │                       │
│ Renderer         │  │ TcpListener           │  │ WasmApp              │
│ NetworkClient    │  │ WsListener    [NEW]   │  │ WsTransport          │
│                  │  │ HttpServer    [NEW]   │  │ CanvasRenderer       │
│                  │  │ SessionPairer [MOD]   │  │ InputHandler         │
│                  │  │ PlayerDb              │  │                       │
└──────────────────┘  └───────────────────────┘  └──────────────────────┘
```

**[NEW]** = new component for this feature  
**[MOD]** = existing component extended for this feature

---

## Network Communication Patterns

### Pattern 1: Browser Client ↔ Server (WebSocket, port 7001)

```
battletris-web (browser)                    battletris-server (port 7001)
        │                                              │
        │  1. HTTP GET /index.html                     │
        │ ─────────────────────────────────────────►  │
        │  2. 200 OK (index.html, headers)             │
        │ ◄─────────────────────────────────────────  │
        │                                              │
        │  3. HTTP GET /bt_bg.js (WASM glue)          │
        │ ─────────────────────────────────────────►  │
        │  4. HTTP GET /bt_bg.wasm                     │
        │ ─────────────────────────────────────────►  │
        │                                              │
        │  5. WS Upgrade GET /game                     │
        │     Origin: http://localhost:7001            │
        │ ─────────────────────────────────────────►  │  Origin validated (SECURITY-08)
        │  6. 101 Switching Protocols                  │  Rate limit checked (SECURITY-11)
        │ ◄─────────────────────────────────────────  │
        │                                              │
        │  7. WS Binary Frame (bincode GameMessage)    │
        │ ──────────────────────────────────────────► │  Frame size checked (SECURITY-13)
        │  8. WS Binary Frame (bincode GameMessage)    │  Bincode decoded to GameMessage
        │ ◄──────────────────────────────────────────  │
```

### Pattern 2: Desktop Client ↔ Server (TCP, port 7000, unchanged)

```
battletris-client (desktop)                 battletris-server (port 7000)
        │                                              │
        │  TCP connect                                 │
        │ ─────────────────────────────────────────►  │
        │  Raw bincode frames (length-prefixed)        │
        │ ◄──────────────────────────────────────────►│
```

### Pattern 3: Cross-play Session Relay (inside server)

```
Browser (WsConn)          SessionPairer          Desktop (TcpConn)
        │                       │                       │
        │  Box<dyn GameConn>    │                       │
        │ ─────────────────►   │                       │
        │                       │  Box<dyn GameConn>    │
        │                       │  ◄─────────────────   │
        │                       │                       │
        │      (pair matched — spawn relay task)        │
        │                       │                       │
        │  GameMessage (WS)     │  GameMessage (TCP)    │
        │ ◄──────────────────────────────────────────── │
        │ ──────────────────────────────────────────── ► │
```

---

## Module Dependencies within battletris-server

```
main.rs
    ├── tcp_listener.rs       (existing — wraps accepted TcpStream in TcpConn)
    ├── ws_listener.rs        [NEW] — axum WS upgrade handler; produces WsConn
    ├── http_server.rs        [NEW] — axum router; ServeDir; security headers
    ├── conn.rs               [NEW] — GameConn trait; TcpConn impl; WsConn impl
    ├── session.rs            [MOD] — SessionPairer now takes Box<dyn GameConn>
    ├── relay.rs              (existing — relay loop unchanged; uses GameConn trait)
    ├── player_db.rs          (existing — unchanged)
    └── elo.rs                (existing — unchanged)
```

---

## Module Structure for battletris-web (new crate)

```
battletris-web/
    src/
        lib.rs            — #[wasm_bindgen(start)] fn start()
        app.rs            — WasmApp struct; tick loop; phase state machine
        ws_transport.rs   — WsTransport; web_sys::WebSocket; bincode encode/decode
        renderer.rs       — CanvasRenderer; Canvas 2D; all screens
        input.rs          — InputHandler; keydown → PlayerInput mapping
    index.html            — HTML shell; <canvas id="game-canvas">; Trunk entry
    Trunk.toml            — wasm-opt flags; output dir = dist/
```

---

## New External Dependencies

### battletris-server additions (Cargo.toml)

```toml
[dependencies]
axum = { version = "0.7", features = ["ws"] }
tower-http = { version = "0.5", features = ["fs", "set-header"] }
tokio-tungstenite = { version = "0.23" }
```

### battletris-web (new crate, Cargo.toml)

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "Window", "Document", "HtmlCanvasElement", "CanvasRenderingContext2d",
    "WebSocket", "MessageEvent", "CloseEvent", "ErrorEvent",
    "KeyboardEvent", "EventTarget", "MouseEvent",
] }
battletris-engine = { path = "../battletris-engine" }
bincode = "1"
serde = { version = "1", features = ["derive"] }

[profile.release]
opt-level = "s"
lto = true
```

### battletris-engine addition (Cargo.toml)

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

---

## Interface Boundaries and Data Flow

| From | To | Data | Encoding |
|---|---|---|---|
| WsTransport (WASM) | WsConn (server) | `GameMessage` | bincode → WS binary frame |
| WsConn (server) | WsTransport (WASM) | `GameMessage` | WS binary frame → bincode |
| TcpConn (server) | NetworkClient (desktop) | `GameMessage` | bincode → length-prefixed TCP |
| NetworkClient (desktop) | TcpConn (server) | `GameMessage` | length-prefixed TCP → bincode |
| WsConn ↔ TcpConn | via relay | `GameMessage` | both decoded then re-encoded in other format |
| InputHandler (WASM) | WasmApp | `PlayerInput` | enum value, in-process |
| WasmApp | CanvasRenderer | `PlayingView`, `BazaarView`, `GameOverView` | struct refs, in-process |
