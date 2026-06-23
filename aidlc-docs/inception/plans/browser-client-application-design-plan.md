# Browser Client — Application Design Plan

## Context

Extending the existing 3-crate workspace with:
- New crate `battletris-web` (WASM binary, Canvas 2D, WebSocket transport)
- Modified `battletris-server` (adds WebSocket listener + HTTP static file server)
- `battletris-engine` unchanged (pure Rust, WASM-compatible as-is, modulo getrandom feature flag)

Existing components (Engine, AI, Protocol, Renderer, NetworkClient, Server) remain. New components are additive.

---

## Design Artifacts to Generate (after answers received)

- [x] Updated `components.md` — existing 6 components + 6 new browser/server components
- [x] Updated `component-methods.md` — new component method signatures
- [x] Updated `services.md` — WsRelayService + HttpStaticService
- [x] Updated `component-dependency.md` — new crate graph + communication patterns
- [x] Updated `application-design.md` — consolidated summary

---

## Design Questions

Please fill in the `[Answer]:` tags below.

---

### Question 1
How should the server expose its TCP game port and its new WebSocket/HTTP endpoints?

The current TCP listener uses a single port (default 7000). Browser clients need HTTP (for static file download) and WebSocket (for game play). These can be on the same port (HTTP upgrade to WS) or separate ports.

A) **Separate ports** — TCP game on port 7000 (unchanged); HTTP + WS on a new port (e.g. 7001). Simplest to implement: no TCP/HTTP multiplexing needed. Desktop clients unchanged.
B) **Combined port** — A single port serves HTTP, WebSocket upgrades, and acts as the raw TCP game port. Requires sniffing the first bytes to distinguish HTTP from raw TCP. Complex but minimises firewall rules.
C) **Three ports** — TCP game (7000), WebSocket game (7001), HTTP static (7002). Maximum separation; most configuration overhead.
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 2
How should the compiled `battletris-web` WASM files be delivered by the server?

After `trunk build` produces a `dist/` directory (index.html, .wasm, .js glue, assets), the server needs to deliver these files to the browser.

A) **Compile-time embed** — Use `rust-embed` (or `include_bytes!`) to bake the entire `dist/` into the server binary at compile time. Single self-contained binary; no separate files to deploy. Requires running `trunk build` before `cargo build --release` for the server.
B) **Runtime disk serving** — Server reads `dist/` from disk at startup (path configurable via CLI arg). Files stay separate from the binary. Simpler build pipeline; easier to update web client without recompiling server.
X) Other (please describe after [Answer]: tag below)

[Answer]: B

---

### Question 3
Which Rust HTTP/WebSocket library should be used for the new server endpoints?

The existing server uses raw `tokio` TcpListener with manual framing. The new endpoints need HTTP and WebSocket handling.

A) **axum** — tokio ecosystem, ergonomic routing, built-in static file serving via `tower-http`; `tokio-tungstenite` for WebSocket upgrade. Most popular modern choice.
B) **warp** — filter-based, built-in WebSocket and static file support; well-tested but less maintained than axum.
C) **hyper + tokio-tungstenite directly** — minimal dependencies, explicit control, but significantly more boilerplate.
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 4
How should the server unify TCP and WebSocket connections in the session relay?

Once a player connects (either via TCP or WebSocket), the session relay needs to read and write `GameMessage` frames bidirectionally. The existing relay logic works on `AsyncRead + AsyncWrite` byte streams.

A) **Trait object abstraction** — Define a `GameConn: AsyncRead + AsyncWrite + Send + Unpin` trait object. Both TCP and WebSocket connections are adapted to this interface. The relay logic is unchanged.
B) **Framed stream unification** — Wrap both TCP and WebSocket in `tokio_util::codec::Framed<_, GameMessageCodec>`. Relay works on `Stream<Item=GameMessage> + Sink<GameMessage>`. Clean but requires a `Codec` implementation.
C) **Enum dispatch** — `enum ServerConn { Tcp(TcpStream), Ws(WebSocketStream) }` with explicit match arms in relay. Most explicit but duplicates relay logic per variant.
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 5
How should the `battletris-web` WASM crate manage its main game loop?

The browser has no native thread sleep — timing is driven by the JavaScript event loop.

A) **`requestAnimationFrame` (rAF) loop** — Standard browser game loop. WASM exports a `tick()` function called each frame via rAF. `GameState` held in a `thread_local! { static APP: RefCell<Option<App>> }`. ~60fps, battery-efficient.
B) **`setInterval` via `gloo-timers`** — Fixed-interval timer calls `tick()`. Simpler than rAF, slightly less precise, not frame-rate adaptive.
X) Other (please describe after [Answer]: tag below)

[Answer]: A
