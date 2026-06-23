# Component Methods — Browser Client Extension

New and modified method signatures for the 6 browser client components.

---

## GameConn Trait (battletris-server::conn)

```rust
/// Unifies TCP and WebSocket connections for the relay session.
/// Both adapters must send/receive bincode-encoded GameMessage in binary frames.
pub trait GameConn: Send + 'static {
    fn read_message<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GameMessage, ProtocolError>> + Send + 'a>>;

    fn write_message<'a>(
        &'a mut self,
        msg: &'a GameMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ProtocolError>> + Send + 'a>>;
}

/// TCP adapter
pub struct TcpConn {
    stream: tokio::net::TcpStream,
}
impl GameConn for TcpConn { ... }

/// WebSocket adapter
pub struct WsConn {
    ws: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
}
impl GameConn for WsConn { ... }
```

---

## WsListener (battletris-server::ws_listener)

```rust
impl WsListener {
    /// Create listener bound to addr (e.g. "0.0.0.0:7001").
    /// allowed_origins: set of permitted Origin values; if empty, deny all non-same-origin upgrades.
    /// max_conns_per_ip: maximum concurrent WS connections per source IP.
    pub fn new(
        addr: std::net::SocketAddr,
        allowed_origins: Vec<String>,
        max_conns_per_ip: usize,
    ) -> Self;

    /// Run the WS upgrade handler.
    /// On each accepted and validated WebSocket, wraps in WsConn and pushes
    /// Box<dyn GameConn> into queue_tx.
    pub async fn run(
        self,
        queue_tx: tokio::sync::mpsc::Sender<Box<dyn GameConn>>,
    ) -> Result<(), std::io::Error>;

    /// axum handler called per WS upgrade request.
    /// Validates Origin header (SECURITY-08), enforces per-IP rate limit (SECURITY-11),
    /// returns 403 on any failure with no internal detail (SECURITY-09).
    async fn handle_upgrade(
        ws: axum::extract::WebSocketUpgrade,
        origin: Option<axum::http::HeaderValue>,
        ip: std::net::IpAddr,
        state: Arc<WsListenerState>,
        queue_tx: tokio::sync::mpsc::Sender<Box<dyn GameConn>>,
    ) -> axum::response::Response;

    /// Validate a single Origin header value against the allowlist.
    fn validate_origin(origin: &str, allowed: &[String]) -> bool;
}
```

---

## HttpServer (battletris-server::http_server)

```rust
impl HttpServer {
    /// Create file server for static files under `web_dir`.
    pub fn new(addr: std::net::SocketAddr, web_dir: std::path::PathBuf) -> Self;

    /// Build the axum router:
    ///   GET /game  -> WebSocket upgrade (delegated to WsListener handler)
    ///   GET /*     -> ServeDir from web_dir
    /// Wraps all routes in security header middleware (SECURITY-04).
    pub fn build_router(
        web_dir: std::path::PathBuf,
        ws_state: Arc<WsListenerState>,
        queue_tx: tokio::sync::mpsc::Sender<Box<dyn GameConn>>,
    ) -> axum::Router;

    /// Middleware layer that injects security headers into every HTTP response:
    ///   Content-Security-Policy, Strict-Transport-Security (if TLS), X-Content-Type-Options,
    ///   X-Frame-Options, Referrer-Policy.
    fn security_headers_layer() -> tower_http::set_header::SetResponseHeaderLayer<...>;

    /// Serve the built router on self.addr until the process exits.
    pub async fn serve(self, router: axum::Router) -> Result<(), std::io::Error>;
}
```

---

## WasmApp (battletris-web::app)

```rust
/// WASM entry point — called once when the WASM module is instantiated.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue>;

impl WasmApp {
    /// Construct app: initialise canvas, ws transport, game state, input handler.
    /// Returns Err if canvas element is not found.
    pub fn new() -> Result<Self, JsValue>;

    /// Called every animation frame.
    /// 1. Drain input handler; feed inputs to game state.
    /// 2. Drain WsTransport incoming queue; feed server messages to game state.
    /// 3. Drain game state outgoing messages; send via WsTransport.
    /// 4. Advance game state tick (physics, drop timer, weapon timers).
    /// 5. Render current view to canvas.
    pub fn tick(&mut self, timestamp_ms: f64);

    /// Returns true if the connection has been lost and the loop should halt.
    pub fn is_disconnected(&self) -> bool;
}

/// Schedules next requestAnimationFrame call pointing at the WASM tick function.
fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> i32;
```

---

## WsTransport (battletris-web::ws_transport)

```rust
impl WsTransport {
    /// Open WebSocket connection to `url` (e.g. "ws://localhost:7001/game").
    /// Registers onmessage/onerror/onclose callbacks.
    /// Binary messages are bincode-decoded into GameMessage and pushed to incoming queue.
    pub fn connect(url: &str) -> Result<Self, JsValue>;

    /// Drain and return all GameMessages received since last call.
    pub fn drain_incoming(&self) -> Vec<GameMessage>;

    /// Encode `msg` via bincode and send as a WebSocket binary frame.
    /// Silently drops message if socket is not in OPEN state.
    pub fn send(&self, msg: &GameMessage);

    /// Returns true if the WebSocket is in OPEN state.
    pub fn is_connected(&self) -> bool;

    /// Returns true if the WebSocket received an error or close event.
    pub fn is_disconnected(&self) -> bool;
}
```

---

## CanvasRenderer (battletris-web::renderer)

```rust
impl CanvasRenderer {
    /// Acquire CanvasRenderingContext2d from the DOM element with id "game-canvas".
    pub fn new() -> Result<Self, JsValue>;

    /// Clear canvas to background colour.
    pub fn clear(&self);

    /// Render the title/lobby screen.
    pub fn draw_title(&self, player_name: &str);

    /// Render the connecting/waiting screen.
    pub fn draw_waiting(&self, message: &str);

    /// Render the main playing screen:
    /// - Local board (left panel): grid, active piece, ghost, next piece
    /// - Remote board (right panel): opponent grid silhouette
    /// - HUD: score, funds, active weapons, drop level
    pub fn draw_playing(&self, view: &PlayingView);

    /// Render the bazaar overlay on top of the playing screen.
    pub fn draw_bazaar(&self, view: &BazaarView);

    /// Render the game over screen with final scores and ELO delta.
    pub fn draw_game_over(&self, view: &GameOverView);

    /// Draw a single text string using the 5x7 bitmap pixel font via fillRect calls.
    fn draw_text(&self, x: f64, y: f64, text: &str, color: &str);

    /// Draw a filled cell at board coordinates (col, row) for the given cell colour index.
    fn draw_cell(&self, panel_x: f64, panel_y: f64, col: i32, row: i32, cell: Cell);
}
```

---

## InputHandler (battletris-web::input)

```rust
impl InputHandler {
    /// Register keydown listener on `window`.
    /// Captures game keys; calls prevent_default() to suppress browser scroll etc.
    pub fn new() -> Result<Self, JsValue>;

    /// Drain and return all PlayerInput values received since last call.
    pub fn drain(&self) -> Vec<PlayerInput>;

    /// Map a KeyboardEvent.code string to a PlayerInput variant.
    /// Returns None for unmapped keys.
    fn map_key_code(code: &str) -> Option<PlayerInput>;
}
```

---

## Key Mapping Table (InputHandler)

| `KeyboardEvent.code` | `PlayerInput` |
|---|---|
| `ArrowLeft` | `MoveLeft` |
| `ArrowRight` | `MoveRight` |
| `ArrowDown` | `SoftDrop` |
| `ArrowUp` | `RotateCW` |
| `KeyZ` | `RotateCCW` |
| `Space` | `HardDrop` |
| `ShiftLeft`, `ShiftRight` | `Hold` |
| `KeyQ` | `Quit` |
| `Digit1`–`Digit9` | `SelectWeapon(1)`–`SelectWeapon(9)` |
| `Enter` | `Confirm` |
