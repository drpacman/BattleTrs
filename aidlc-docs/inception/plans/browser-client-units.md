# Units Generation — Browser Client Extension

**Feature**: Browser-based BattleTris client  
**Date**: 2026-06-22  
**Basis**: Application Design COMPLETED; 2 units confirmed in Workflow Planning

---

## Pre-Unit Setup (not a separate unit — part of Unit A Code Generation step 1)

Before Unit A construction begins, two workspace-level changes must be made:

| Change | File | Notes |
|---|---|---|
| Add `battletris-web` workspace member | `Cargo.toml` | `members = [..., "battletris-web"]` |
| Add `getrandom` WASM feature to engine | `battletris-engine/Cargo.toml` | `[target.'cfg(target_arch = "wasm32")'.dependencies]` block |

These are done as the first two checkboxes of Unit A Code Generation, since they are prerequisites for any subsequent code to compile.

---

## Unit A — Server WebSocket + HTTP

### Identity

| Field | Value |
|---|---|
| **Unit name** | `server-ws-http` |
| **Crate** | `battletris-server` |
| **Type** | Extension of existing crate |
| **Stages** | Functional Design → NFR Requirements → NFR Design → Code Generation |

### Scope

**New files**:
- `battletris-server/src/conn.rs` — `GameConn` trait; `TcpConn` and `WsConn` adapters
- `battletris-server/src/ws_listener.rs` — axum WebSocket upgrade handler; origin validation; rate limiting
- `battletris-server/src/http_server.rs` — axum router; ServeDir; security headers middleware

**Modified files**:
- `battletris-server/src/main.rs` — spawn WS listener + HTTP server tasks; accept `--web-port` and `--web-dir` CLI args
- `battletris-server/src/session.rs` — `SessionPairer` accepts `Box<dyn GameConn>` instead of bare `TcpStream`
- `battletris-server/Cargo.toml` — add `axum`, `tower-http`, `tokio-tungstenite`

**Unchanged files**: `relay.rs`, `player_db.rs`, `elo.rs`, all existing tests

### Entry Criteria

- [ ] Application Design COMPLETED and approved
- [ ] `battletris-engine` compiles cleanly (`cargo build -p battletris-engine`)
- [ ] All 83 existing server tests pass (`cargo test -p battletris-server`)

### Exit Criteria

- [ ] `cargo build -p battletris-server` succeeds with 0 errors, 0 warnings
- [ ] All pre-existing 83 tests still pass
- [ ] At least 3 new integration tests added: WS upgrade rejected (bad origin), WS upgrade rejected (rate limit), WS upgrade accepted + message round-trip
- [ ] HTTP GET for a static file returns the file with all security headers present
- [ ] `cargo clippy -p battletris-server` passes with 0 warnings
- [ ] Security NFRs SECURITY-04, SECURITY-05, SECURITY-08, SECURITY-09, SECURITY-11, SECURITY-13, SECURITY-15 all verifiable via tests or manual check

### Dependencies

- Depends on: `battletris-engine` (unchanged; just verifying WASM compat flag)
- Blocks: Unit B (Unit B's WsTransport connects to the WebSocket endpoint Unit A creates)
- Does NOT affect: `battletris-client` (desktop client is unchanged)

### Sequencing Rationale

Unit A before Unit B because:
1. The browser WASM client needs a running server WS endpoint to connect against during integration testing
2. Server-side `GameConn` trait and `WsConn` adapter must exist before WASM connection code can be written against a known protocol

---

## Unit B — battletris-web WASM Client

### Identity

| Field | Value |
|---|---|
| **Unit name** | `battletris-web-wasm` |
| **Crate** | `battletris-web` (new cdylib) |
| **Type** | New crate |
| **Stages** | Functional Design → Code Generation (NFR Requirements: SKIP, NFR Design: SKIP, Infrastructure: SKIP) |

### Scope

**New crate** (`battletris-web/`):

| File | Component |
|---|---|
| `src/lib.rs` | WASM entry (`#[wasm_bindgen(start)]`); rAF scheduling |
| `src/app.rs` | `WasmApp`; tick loop; phase state machine |
| `src/ws_transport.rs` | `WsTransport`; `web_sys::WebSocket`; bincode encode/decode |
| `src/renderer.rs` | `CanvasRenderer`; Canvas 2D; all screens |
| `src/input.rs` | `InputHandler`; keydown → `PlayerInput` |
| `index.html` | HTML shell; `<canvas id="game-canvas">`; Trunk entry point |
| `Trunk.toml` | WASM build config; `--release` wasm-opt flags |
| `Cargo.toml` | cdylib; wasm-bindgen; web-sys features; battletris-engine dep |

**No existing files modified** (Unit B is entirely additive at the Rust level; workspace `Cargo.toml` modified in Unit A setup step).

### Entry Criteria

- [ ] Unit A exit criteria all met
- [ ] `battletris-server` running on port 7001 with WS endpoint available
- [ ] `trunk` CLI installed (`trunk --version`)
- [ ] `wasm32-unknown-unknown` target installed (`rustup target add wasm32-unknown-unknown`)

### Exit Criteria

- [ ] `trunk build` succeeds in `battletris-web/` (produces `dist/` directory)
- [ ] `trunk build --release` succeeds with WASM size optimisation applied
- [ ] Browser loads `index.html` from server, WASM initialises without JS errors
- [ ] All 4 screens render correctly: title, connecting/waiting, playing, game over
- [ ] Keyboard input correctly translates to `PlayerInput` (all mapped keys verified)
- [ ] Browser client successfully connects to server via WebSocket and plays a full network game against a desktop client (manual integration test)
- [ ] Browser client vs browser client game works end-to-end
- [ ] `cargo clippy --target wasm32-unknown-unknown -p battletris-web` passes with 0 warnings

### Dependencies

- Depends on: Unit A (WS endpoint at `ws://<host>:7001/game`)
- Depends on: `battletris-engine` (all game types: `GameState`, `PlayerInput`, `PlayingView`, `BazaarView`, `GameOverView`, `GameMessage`)
- Does NOT affect: `battletris-client`, `battletris-server` (Unit B is read-only consumer of both)

### NFR Skip Rationale

| Stage | Decision | Reason |
|---|---|---|
| NFR Requirements | SKIP | Security NFRs are server-side (Unit A). WASM binary size is a Functional Design concern, not a separate NFR stage. |
| NFR Design | SKIP | NFR Requirements skipped |
| Infrastructure Design | SKIP | No cloud infrastructure; Trunk is a build tool, not infrastructure |

---

## Unit Sequence Summary

```
Pre-Unit Setup (within Unit A CG step 1)
    workspace Cargo.toml → add battletris-web member
    battletris-engine/Cargo.toml → getrandom WASM feature
         |
         v
Unit A — server-ws-http
    Functional Design
        |
    NFR Requirements
        |
    NFR Design
        |
    Code Generation (battletris-server)
         |
         v
Unit B — battletris-web-wasm
    Functional Design
        |
    Code Generation (battletris-web)
         |
         v
Build and Test
    cargo test --workspace
    trunk build --release
    manual: browser vs desktop integration test
    manual: browser vs browser integration test
```

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `battletris-engine` has a transitive dep that breaks WASM compile | Medium | High | Resolved in Unit A setup step; fail fast before Unit B starts |
| WebSocket binary frames rejected by browser security policy | Low | High | Unit A WsListener sets permissive but specific CORS for WS upgrade; Unit A exit test validates round-trip |
| Canvas 2D rendering performance on large boards | Low | Medium | Limit full redraws to changed regions; acceptable for 10×20 Tetris board |
| bincode GameMessage size exceeds WS frame limit | Low | Low | max_frame_bytes=65536 in config; a full board state is ~220 bytes — well within limit |
| trunk build tool not available in CI | Low | Medium | Document manual install step; Trunk.toml pins wasm-bindgen-cli version |
