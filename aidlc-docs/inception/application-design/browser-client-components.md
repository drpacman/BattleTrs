# Components — Browser Client Extension

Extends the original 6-component design with 6 new components (2 server-side, 4 WASM client-side) and 1 modified component.

---

## Modified Component: Server (battletris-server)

**Original role**: tokio TCP relay, PlayerDb, ELO

**Extended role**: Now also owns a `GameConn` trait that abstracts both TCP and WebSocket connections, allowing the existing relay session logic to work without modification regardless of transport type.

**New responsibility added**:
- Define and implement `trait GameConn` for `TcpConn` and `WsConn` adapters
- Maintain a single shared player queue (`Arc<Mutex<Vec<Box<dyn GameConn>>>>`) fed by both TCP and WS listeners

---

## New Component 7: WsListener (battletris-server)

**Crate**: `battletris-server`
**Module**: `battletris-server::ws_listener`

**Responsibilities**:
- Listen on the HTTP/WS port (default 7001) via axum
- Handle WebSocket upgrade requests at route `GET /game`
- Validate `Origin` header against a configurable allowlist before upgrading (SECURITY-08)
- Track per-IP connection count and enforce rate limiting — reject upgrades exceeding threshold (SECURITY-11)
- On successful upgrade, wrap the `WebSocketStream` in a `WsConn` adapter implementing `GameConn`
- Hand the `Box<dyn GameConn>` to the shared session queue
- Return generic error responses on failure — no internal details exposed (SECURITY-09)
- Size-bound incoming binary WebSocket frames before passing to bincode deserialiser (SECURITY-13)

---

## New Component 8: HttpServer (battletris-server)

**Crate**: `battletris-server`
**Module**: `battletris-server::http_server`

**Responsibilities**:
- Serve static files from a disk directory (configurable via `--web-dir <path>` CLI arg) using `tower-http::services::ServeDir`
- Attach security header middleware to all HTTP responses (SECURITY-04):
  - `Content-Security-Policy: default-src 'self'; script-src 'self'; wasm-src 'self'`
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains`
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: strict-origin-when-cross-origin`
- Return `404` for missing files with generic message body
- Share the same axum `Router` and port 7001 as `WsListener` (routes: `GET /game` → WS upgrade; all other GET → static files)

---

## New Component 9: WasmApp (battletris-web)

**Crate**: `battletris-web`
**Module**: `battletris-web::app`

**Responsibilities**:
- WASM entry point (`#[wasm_bindgen(start)]`)
- Hold all top-level state in a `thread_local! { static APP: RefCell<Option<WasmApp>> }`
- Own `GameState`, `WsTransport`, `CanvasRenderer`, `InputHandler`
- Drive the requestAnimationFrame tick loop: drain `InputHandler`, tick `GameState`, send outgoing `GameMessage` via `WsTransport`, drain incoming `GameMessage` from `WsTransport` into `GameState`, render current view via `CanvasRenderer`
- Manage the app phase state machine: `Title → Connecting → WaitingForOpponent → Playing → InBazaar → GameOver`
- Schedule next rAF callback unconditionally (loop continues until page unload)

---

## New Component 10: WsTransport (battletris-web)

**Crate**: `battletris-web`
**Module**: `battletris-web::ws_transport`

**Responsibilities**:
- Open a `web_sys::WebSocket` connection to `ws://<host>:7001/game`
- Register `onmessage` callback: decode incoming binary `ArrayBuffer` frames via `bincode` into `GameMessage`; push to an internal `VecDeque<GameMessage>` backed by `Rc<RefCell<VecDeque<GameMessage>>>`
- Register `onerror` and `onclose` callbacks: set a disconnected flag
- Provide `drain_incoming() -> Vec<GameMessage>` for the game loop to poll each tick
- Provide `send(msg: &GameMessage)` — bincode-encodes to `Vec<u8>` and calls `WebSocket::send_with_u8_array()`
- Handle connection lifecycle: not-yet-connected, open, closed/error states

---

## New Component 11: CanvasRenderer (battletris-web)

**Crate**: `battletris-web`
**Module**: `battletris-web::renderer`

**Responsibilities**:
- Hold a reference to the `CanvasRenderingContext2d` obtained from the DOM
- Render all game screens to Canvas 2D using `fillRect`, `fillText`, and colour operations
- Implement the same logical layout as the SDL2 renderer: board grid, active piece, ghost piece, next piece preview, score/funds panel, opponent board panel, active weapon chips, bazaar overlay, game over screen, title screen
- Reuse the same 5×7 bitmap font approach (draw pixels via `fillRect`) for consistent cross-platform text rendering
- Accept `PlayingView`, `BazaarView`, `GameOverView` from `battletris-engine` (same view types as SDL2 client)

---

## New Component 12: InputHandler (battletris-web)

**Crate**: `battletris-web`
**Module**: `battletris-web::input`

**Responsibilities**:
- Register a `keydown` event listener on `window` via `web-sys`
- Map `KeyboardEvent.code` strings to `PlayerInput` variants (same mapping as `scancode_to_input` in `battletris-client`)
- Push `PlayerInput` values into an internal `Rc<RefCell<VecDeque<PlayerInput>>>` queue
- Provide `drain() -> Vec<PlayerInput>` for the game loop to poll each tick
- Suppress default browser behaviour (e.g. `Space` scrolling) for game keys via `event.prevent_default()`
