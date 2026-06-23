# Functional Design — Unit B: battletris-web WASM Client

**Date**: 2026-06-22  
**Crate**: `battletris-web` (cdylib)  
**Status**: PENDING code generation approval

---

## Scope Summary

New crate implementing a browser WASM game client. Reuses `battletris-engine` unchanged. Mirrors the SDL2 desktop client's game loop logic (`battletris-client/src/game_loop.rs`) but drives it from a `requestAnimationFrame` tick instead of a sleep loop, and renders via Canvas 2D instead of SDL2.

---

## 1. Connection and Name Entry

Rather than implementing in-canvas text input, the browser's native `window.prompt()` is used for player name on startup, and the WebSocket URL is derived from the current page origin.

```rust
// In WasmApp::new()
let name = window.prompt_with_message("Enter your player name")?.unwrap_or_default();
if name.is_empty() { return Err("name required".into()); }

let location = window.location();
let origin = location.origin()?;
// e.g. "http://192.168.1.10:7001" → "ws://192.168.1.10:7001/game"
let ws_url = origin
    .replace("https://", "wss://")
    .replace("http://",  "ws://")
    + "/game";
```

This keeps the WASM client simple — no HTML form elements, no in-canvas keyboard input tracking for text fields.

---

## 2. rAF Game Loop Pattern

The browser's single-threaded event model requires all game logic to run in the rAF callback. The standard wasm-bindgen pattern stores the tick `Closure` in a `thread_local` so it is never dropped:

```rust
thread_local! {
    static APP:  RefCell<Option<WasmApp>> = RefCell::new(None);
    static TICK: RefCell<Option<Closure<dyn FnMut(f64)>>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();  // readable panics in console

    let app = WasmApp::new().expect("init failed");
    APP.with(|a| *a.borrow_mut() = Some(app));

    TICK.with(|t| {
        *t.borrow_mut() = Some(Closure::wrap(Box::new(|ts: f64| {
            APP.with(|a| {
                if let Some(ref mut app) = *a.borrow_mut() {
                    app.tick(ts);
                }
            });
            // Reschedule unconditionally — loop ends only on page unload
            TICK.with(|t| {
                web_sys::window().unwrap()
                    .request_animation_frame(
                        t.borrow().as_ref().unwrap().as_ref().unchecked_ref()
                    )
                    .unwrap();
            });
        }) as Box<dyn FnMut(f64)>));
    });

    // Kick off first frame
    TICK.with(|t| {
        web_sys::window().unwrap()
            .request_animation_frame(
                t.borrow().as_ref().unwrap().as_ref().unchecked_ref()
            )
            .unwrap();
    });
}
```

`WasmApp::tick(timestamp_ms: f64)` computes `elapsed_ms = timestamp - last_timestamp` and feeds it to `GameState::tick`.

---

## 3. WasmApp — Phase State Machine

```
Connecting
    │ onopen fires → send Hello → phase: WaitingForOpponent
    ↓
WaitingForOpponent
    │ GameStart received → phase: Playing
    ↓
Playing  ←──────────────────────────────────────────┐
    │ BazaarOpen received → phase: InBazaar          │
    │ GameOver received → phase: GameOver            │
    ↓                                               │
InBazaar                                           │
    │ BazaarEnd received → phase: Playing ──────────┘
    ↓
GameOver
    │ Enter/Space pressed → re-prompt name, reconnect
```

Each `tick()` call:
1. `input_handler.drain()` → `latest_input: Option<PlayerInput>`
2. `ws_transport.drain_incoming()` → process each `GameMessage` (same logic as `process_peer_message` in `game_loop.rs`)
3. Compute `elapsed_ms` from rAF timestamp
4. `game_state.tick(latest_input, elapsed_ms)` → `Vec<GameEvent>`
5. For each `GameEvent`, send corresponding `GameMessage` via `ws_transport.send()`
6. Render via `canvas_renderer`

---

## 4. WsTransport

### Struct

```rust
pub struct WsTransport {
    ws: WebSocket,
    incoming: Rc<RefCell<VecDeque<GameMessage>>>,
    connected: Rc<Cell<bool>>,
    disconnected: Rc<Cell<bool>>,
    // Closures kept alive here so they are not dropped
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_open:    Closure<dyn FnMut(Event)>,
    _on_close:   Closure<dyn FnMut(CloseEvent)>,
    _on_error:   Closure<dyn FnMut(Event)>,
}
```

### Construction

```rust
impl WsTransport {
    pub fn connect(url: &str) -> Result<Self, JsValue> {
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(BinaryType::Arraybuffer);

        let incoming = Rc::new(RefCell::new(VecDeque::new()));
        let connected = Rc::new(Cell::new(false));
        let disconnected = Rc::new(Cell::new(false));

        // onmessage: decode binary ArrayBuffer → GameMessage → push to queue
        let incoming_c = incoming.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(buf) = e.data().dyn_into::<ArrayBuffer>() {
                let bytes = Uint8Array::new(&buf).to_vec();
                if let Ok(msg) = protocol::decode_raw(&bytes) {
                    incoming_c.borrow_mut().push_back(msg);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        // onopen, onclose, onerror: update connected/disconnected flags
        // ...

        Ok(Self { ws, incoming, connected, disconnected,
                  _on_message: on_message, /* ... */ })
    }

    pub fn drain_incoming(&self) -> Vec<GameMessage> {
        self.incoming.borrow_mut().drain(..).collect()
    }

    pub fn send(&self, msg: &GameMessage) {
        if !self.connected.get() { return; }
        if let Ok(bytes) = protocol::encode_raw(msg) {
            let arr = Uint8Array::from(bytes.as_slice());
            let _ = self.ws.send_with_array_buffer(&arr.buffer());
        }
    }

    pub fn is_connected(&self) -> bool { self.connected.get() }
    pub fn is_disconnected(&self) -> bool { self.disconnected.get() }
}
```

The `Closure` instances are owned by the struct (stored as `_on_message` etc.) so they live as long as `WsTransport`. No `.forget()` calls needed.

---

## 5. InputHandler

### Key Mapping

| `KeyboardEvent.code` | `PlayerInput` |
|---|---|
| `ArrowLeft` | `MoveLeft` |
| `ArrowRight` | `MoveRight` |
| `ArrowDown` | `SoftDrop` |
| `ArrowUp` | `RotateCW` |
| `KeyZ` | `RotateCCW` |
| `Space` | `HardDrop` |
| `ShiftLeft` / `ShiftRight` | `Hold` |
| `Digit1`–`Digit9` | `SelectWeapon(1)`–`SelectWeapon(9)` |
| `Enter` | `StartGame` |
| `KeyN` | `StartGame` (title screen "new game") |

### Prevent Default

`event.prevent_default()` is called for all mapped keys to suppress browser scroll on Space/arrows.

### Struct

```rust
pub struct InputHandler {
    queue: Rc<RefCell<VecDeque<PlayerInput>>>,
    _on_keydown: Closure<dyn FnMut(KeyboardEvent)>,
}
```

The closure is owned by the struct. The event listener is registered on `window` (not `document` — catches events when canvas is focused).

`drain()` returns `Vec<PlayerInput>`; `tick()` takes `latest_input = queue.drain(..).last()` (only the most recent input per frame, matching SDL2 client behaviour).

---

## 6. CanvasRenderer

### Canvas Setup

The HTML has `<canvas id="game-canvas" width="820" height="860"></canvas>`. The renderer holds `CanvasRenderingContext2d` obtained via `canvas.get_context("2d")`.

### Layout Constants

Identical to SDL2 client (logical pixels):

```
CELL_PX        = 28
WINDOW_W       = 820
WINDOW_H       = 860
PLAYER_BOARD_X = 20,   PLAYER_BOARD_Y = 40
STATS_X        = 310,  STATS_Y        = 40
OPP_BOARD_X    = 520,  OPP_BOARD_Y    = 40
BOARD_PX_W     = 280,  BOARD_PX_H     = 784
```

### Color Palette

Reuse the same BT color index → CSS hex mapping as SDL2's `bt_color()`:

```rust
fn bt_color(index: u8) -> &'static str {
    match index {
        1 => "#ffffe6", // BT_IVORY
        2 => "#ffdc00", // BT_YELLOW
        3 => "#dc3232", // BT_RED
        4 => "#3264dc", // BT_BLUE
        5 => "#ff8c00", // BT_ORANGE
        6 => "#32c832", // BT_GREEN
        7 => "#00c8dc", // BT_CYAN
        8 => "#a032c8", // BT_PURPLE
        _  => "#808080",
    }
}
```

### Cell Rendering

```rust
fn draw_cell(&self, px: f64, py: f64, color: &str) {
    // Darker border: draw full cell in border color, then inset fill
    let dark = darken_hex(color); // compute 40-unit darker variant
    self.ctx.set_fill_style_str(&dark);
    self.ctx.fill_rect(px, py, CELL_PX, CELL_PX);
    self.ctx.set_fill_style_str(color);
    self.ctx.fill_rect(px + 1.0, py + 1.0, CELL_PX - 2.0, CELL_PX - 2.0);
}
```

### Font

Port the 5×7 bitmap font from `renderer/font.rs` to Canvas 2D. The `FONT` constant is copied verbatim (65 glyph bitmaps). Rendering: for each character, for each of the 7 rows, test each of the 5 bits; if set, `ctx.fill_rect(...)` a 1×scale pixel. Scale factor matches size parameter.

```rust
fn draw_text(&self, text: &str, x: f64, y: f64, color: &str, scale: f64) {
    self.ctx.set_fill_style_str(color);
    let mut cursor_x = x;
    for ch in text.chars() {
        let idx = (ch as usize).saturating_sub(32);
        if idx < FONT.len() {
            for (row, &bits) in FONT[idx].iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.ctx.fill_rect(
                            cursor_x + col as f64 * scale,
                            y + row as f64 * scale,
                            scale,
                            scale,
                        );
                    }
                }
            }
        }
        cursor_x += (GLYPH_W as f64 + 1.0) * scale; // 1px letter spacing
    }
}
```

### Screens

| Screen | Trigger | Notes |
|---|---|---|
| `draw_connecting` | `Connecting` phase | "Connecting…" text centered |
| `draw_waiting` | `WaitingForOpponent` phase | "Waiting for opponent…" text centered |
| `draw_playing` | `Playing` + `InBazaar` phases | Full layout: player board, stats, opponent board; bazaar overlay on top when InBazaar |
| `draw_game_over` | `GameOver` phase | Winner name, ELO delta, "Press ENTER to play again" |

The `draw_playing` screen mirrors `playing.rs` from the SDL2 client: draws board from `PlayingView`, active piece, ghost piece, next piece preview, score/funds panel, weapon slots, opponent board.

Die pip patterns and happy-face symbols drawn with `fill_rect` calls (same geometry as SDL2 client, trivially translated).

---

## 7. Game Message Processing

The `process_peer_message` logic from `game_loop.rs` is ported verbatim to `WasmApp::process_message()`:

- `GameStart` → set `game_started = true`
- `BazaarOpen` → `state.open_bazaar_now(); state.ernie_bazaar_done()`
- `BoardUpdate` → store `peer_board`
- `ScoreUpdate` → store `peer_score`
- `WeaponLaunched` → `state.apply_incoming_weapon(kind, &mut rng)` + handle reflect
- `WeaponReflected` → `state.apply_incoming_weapon(kind, &mut rng)`
- `FundsReceived` → `state.score.funds += amount`
- `GameOver` → determine `i_won`, store `network_result`, set `state.phase = GameOver { won: i_won }`
- `PeerDisconnected` → set `peer_disconnected = true`
- `GameVoid` → `state.phase = Title`

---

## 8. Outgoing Message Generation

Per-tick, after `state.tick()` returns `Vec<GameEvent>`, the same mapping as `game_loop.rs` applies:

| `GameEvent` | `GameMessage` sent |
|---|---|
| `WeaponFired { reflect: false, kind }` | `WeaponLaunched { kind }` |
| `LinesCleared(n)` | `LinesCleared { count: n, funds_earned: 0 }` |
| `PieceLocked` | `BoardUpdate { snapshot: state.board.snapshot() }` |
| (every tick, Playing phase) | `ScoreUpdate { score, lines, funds }` |
| (local board tops out) | `GameOver { winner_id: 0, … }` |

---

## 9. Cargo.toml for battletris-web

```toml
[package]
name = "battletris-web"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "Window", "Document", "HtmlCanvasElement", "CanvasRenderingContext2d",
    "WebSocket", "MessageEvent", "CloseEvent", "ErrorEvent", "BinaryType",
    "KeyboardEvent", "EventTarget", "Event",
    "Location", "Performance",
    "ArrayBuffer", "Uint8Array",
] }
battletris-engine = { path = "../battletris-engine" }
bincode = { workspace = true }
serde = { workspace = true }
console_error_panic_hook = "0.1"

[profile.release]
opt-level = "s"
lto = true
```

---

## 10. Trunk.toml

```toml
[build]
target = "index.html"

[watch]
ignore = []
```

Release build command: `trunk build --release`
Dev command: `trunk serve` (hot reload at http://localhost:8080)

---

## 11. index.html

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>BattleTris</title>
    <style>
        body { background: #000; display: flex; justify-content: center; margin: 0; }
        canvas { display: block; image-rendering: pixelated; }
    </style>
</head>
<body>
    <canvas id="game-canvas" width="820" height="860"></canvas>
    <script type="module">
        import init from './battletris_web.js';
        init();
    </script>
</body>
</html>
```

Trunk replaces the script tag with the correct wasm-bindgen glue module on build.

---

## 12. Dependency on Unit A

Unit B connects to `ws://<origin>/game`. For local testing during development:
- `trunk serve` runs at `localhost:8080`
- The server must be running at the same host on port 7001
- The browser will connect to `ws://localhost:7001/game` (not `ws://localhost:8080/game`)

The JS origin will be `http://localhost:8080` when using trunk serve. To resolve this:
- During dev: pass the server URL as a query param (`?server=ws://localhost:7001`) OR hardcode a fallback
- For production (same host): use the page origin directly

**Design decision**: accept an optional `?server=<url>` query parameter, falling back to `<origin>/game` if absent. This allows trunk serve dev workflow without modifying server configuration.

```rust
fn ws_url_from_location() -> String {
    let window = web_sys::window().unwrap();
    let location = window.location();
    
    // Check for ?server=ws://... override
    if let Ok(search) = location.search() {
        if let Some(param) = search.split('&')
            .find(|s| s.starts_with("server=") || s.starts_with("?server="))
        {
            return param.splitn(2, '=').nth(1).unwrap_or("").to_string();
        }
    }
    
    // Default: same origin, /game path
    let origin = location.origin().unwrap_or_default();
    origin.replace("https://", "wss://").replace("http://", "ws://") + "/game"
}
```
