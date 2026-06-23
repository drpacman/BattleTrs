# Browser Client — Requirements

## Intent Analysis

| Field | Value |
|---|---|
| **User Request** | Add a browser-based client whilst maintaining the desktop SDL2 client; reuse existing game implementation; new networking and rendering as appropriate for browser |
| **Request Type** | New Feature (additive) |
| **Scope** | System-wide — new crate, server modifications, shared engine |
| **Complexity** | Complex — cross-platform multiplayer, WASM compilation, dual transport protocol |

## Key Decisions (from Q&A)

| Decision | Choice |
|---|---|
| Engine location | WASM — `battletris-engine` compiled to WebAssembly, runs in browser |
| Browser rendering | Rust/WASM + HTML5 Canvas 2D via `web-sys` |
| Server transport | Extend `battletris-server` to accept both TCP (desktop) and WebSocket (browser) |
| Cross-play | All combinations — browser/browser, desktop/desktop, browser/desktop |
| Solo practice | Network play only in browser (no offline mode) |
| Build tooling | Trunk |
| Static file serving | `battletris-server` serves the browser client static files |
| Feature scope | Network play only initially; vs-computer and solo practice remain desktop-only |
| New crate name | `battletris-web` |
| Security extension | Enabled |
| PBT extension | Disabled |

---

## Functional Requirements

### FR-WEB-01 — WASM Engine
`battletris-engine` MUST compile to WASM without modification. The crate MUST have no platform-specific dependencies (no SDL2, no tokio, no file I/O). Game logic runs in the browser process.

### FR-WEB-02 — New `battletris-web` Crate
A new workspace member `battletris-web` MUST be created as a `cdylib` (WASM library) crate. It MUST depend on `battletris-engine` and use `wasm-bindgen` + `web-sys` for browser integration.

### FR-WEB-03 — Canvas Rendering
The browser client MUST render the game on an HTML5 Canvas element using the same logical layout as the desktop client: board, active piece, ghost piece, next piece preview, score/funds display, active weapon chips, opponent board, and game over/bazaar overlays.

### FR-WEB-04 — WebSocket Transport
The browser client MUST connect to the server via WebSocket (not raw TCP). Messages MUST use the same `GameMessage` framing as the desktop protocol (binary, bincode-encoded) so no new message types are required for the shared relay logic.

### FR-WEB-05 — Cross-Client Matchmaking
The server MUST match players regardless of their transport type. A browser client and a desktop client MUST be able to play each other. The server relay logic MUST treat WS and TCP connections uniformly once a game session is established.

### FR-WEB-06 — Server WebSocket Listener
`battletris-server` MUST open a WebSocket listener (distinct port or HTTP upgrade on an additional port) alongside the existing TCP listener. Both listeners share the same session management and relay logic.

### FR-WEB-07 — Static File Serving
`battletris-server` MUST serve the compiled `battletris-web` static files (HTML, WASM, JS glue, assets) over HTTP. The Trunk build output (`dist/`) MUST be embeddable in or servable by the server binary.

### FR-WEB-08 — Keyboard Input
The browser client MUST map the same keys as the desktop client (arrow keys, Z, Space, Down, P, Esc, 1-9/0, numpad Enter, Page Up/Down, B) using DOM keyboard events.

### FR-WEB-09 — Game Modes in Browser
The browser client MUST support network play only. VS-computer (Ernie) and solo practice modes are desktop-only and are NOT required in the browser client.

### FR-WEB-10 — Trunk Build
The `battletris-web` crate MUST be buildable with `trunk build` and servable locally with `trunk serve`. A `Trunk.toml` MUST be provided with appropriate WASM optimisation settings.

### FR-WEB-11 — Feature Parity for Network Play
During network play, the browser client MUST support all 34 weapons (visual effects and active-weapon display), the bazaar screen, ELO delta display on game over, and the opponent board display — matching the desktop client's network play capabilities.

---

## Non-Functional Requirements (Security Extension Enabled)

### NFR-WEB-SEC-04 — HTTP Security Headers
The server's static file HTTP responses MUST include: `Content-Security-Policy` (restrictive, no `unsafe-inline`/`unsafe-eval`), `Strict-Transport-Security`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`. (SECURITY-04)

### NFR-WEB-SEC-05 — WebSocket Input Validation
Every `GameMessage` received over WebSocket MUST be validated before processing: type-checked via bincode deserialization, size-bounded (reject payloads above a defined maximum), and any string fields (player name) length-bounded and stripped of control characters. (SECURITY-05)

### NFR-WEB-SEC-08 — WebSocket Origin and CORS
The WebSocket upgrade handler MUST validate the `Origin` header against an allowlist of permitted origins. `Access-Control-Allow-Origin: *` MUST NOT be used on authenticated or game-play endpoints. (SECURITY-08)

### NFR-WEB-SEC-09 — Error Handling
WebSocket and HTTP error responses MUST return generic messages; no stack traces, internal paths, or Rust panic details MUST be exposed to the client. (SECURITY-09)

### NFR-WEB-SEC-10 — Supply Chain
`Cargo.lock` is already committed. New dependencies (tokio-tungstenite, axum/tower, web-sys, wasm-bindgen, gloo) MUST be sourced from crates.io. No unpinned version ranges that permit major version drift. (SECURITY-10)

### NFR-WEB-SEC-11 — Rate Limiting
The WebSocket upgrade endpoint and connection handler MUST implement per-IP connection rate limiting to prevent resource exhaustion. (SECURITY-11)

### NFR-WEB-SEC-13 — Safe Deserialization
Bincode deserialization of untrusted WebSocket payloads MUST use size limits. Malformed or oversized messages MUST be rejected and the connection closed without panicking. (SECURITY-13)

### NFR-WEB-SEC-15 — Exception Handling
All WebSocket message handling paths MUST have explicit error handling. Connection errors MUST close the session cleanly and release server resources. Panics in WebSocket handlers MUST be caught and converted to clean disconnections. (SECURITY-15)

### NFR-WEB-PERF-01 — WASM Binary Size
The compiled WASM binary MUST use `wasm-opt` (via Trunk) and release-mode optimisation. Target: < 5 MB uncompressed.

### NFR-WEB-COMPAT-01 — Browser Compatibility
The client MUST function in current-version Chrome, Firefox, and Safari.

---

## Affected Components

| Component | Change Type | Reason |
|---|---|---|
| `battletris-engine` | Verify/minor | Must confirm no `#[cfg(not(target_arch = "wasm32"))]` exclusions needed; add WASM compilation check to CI |
| `battletris-server` | Significant | Add WS listener, HTTP static file serving, origin validation, security headers, rate limiting |
| `battletris-web` | New crate | Full new WASM browser client |
| `Cargo.toml` (workspace) | Minor | Add `battletris-web` workspace member |

---

## Out of Scope

- VS-computer (Ernie) in browser
- Solo practice in browser
- Server-side game logic / thin client model
- Mobile browser optimisation (responsive layout, touch input)
- Audio
- TLS termination (assumed to be handled upstream if needed; not in scope for this feature)
