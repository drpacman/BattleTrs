# Browser Client — Requirements Clarification Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your intent, choose the last option (X) and describe your preference.

---

## Question 1
How should the game logic run in the browser?

The existing `battletris-engine` crate is pure Rust with no platform dependencies — it can be compiled to WebAssembly (WASM) so the full game loop runs locally in the browser. Alternatively, the browser can be a thin client where all game logic stays on the server.

A) Compile `battletris-engine` to WASM — game logic runs in the browser (offline solo practice possible; reduces server load)
B) Thin-client browser — browser sends inputs to server and receives rendered state; all game logic stays on server
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
What technology should render the game in the browser?

A) Rust/WASM + HTML5 Canvas 2D via `web-sys` — stays closest to the existing SDL2 pixel-level rendering approach
B) Rust/WASM + a component framework such as Yew or Leptos — DOM-based rendering with reactive components
C) Plain TypeScript/JavaScript canvas — no WASM, browser client is JS-only
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 3
How should the browser connect to the server?

Browsers cannot use raw TCP sockets. The current `battletris-server` uses a custom TCP binary protocol. This needs to change for browser clients.

A) Add WebSocket support to the existing `battletris-server` — one server handles both desktop (TCP) and browser (WS) clients simultaneously
B) Add WebSocket-only support to the server and also migrate the desktop client from TCP to WebSocket (single unified protocol)
C) Create a separate lightweight WebSocket bridge/proxy that sits in front of the existing TCP server
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 4
Which player matchup combinations should be supported?

A) Browser vs browser only — two browser clients can play each other
B) Browser vs desktop — a browser client and a desktop (SDL2) client can be matched together
C) All combinations — browser/browser, desktop/desktop, and browser/desktop all work
X) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Question 5
Should solo practice mode work in the browser without a server connection?

If the engine runs as WASM (Q1=A), solo practice can run entirely offline. If the engine stays on the server (Q1=B), a server connection is always required.

A) Yes — solo practice should work offline in the browser (engine in WASM, no server needed)
B) Solo practice in the browser requires a server connection (simpler, consistent with thin-client approach)
C) No solo practice in the browser — network play only
X) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Question 6
What Rust WASM build tooling should be used?

A) Trunk — the standard all-in-one Rust WASM build tool (`trunk serve`, `trunk build`, automatic asset bundling)
B) `wasm-pack` + a JS bundler (Vite, webpack) — more control, more configuration
C) Minimal — `wasm-bindgen` CLI only, no meta-tooling
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 7
How should the browser client be served?

A) The `battletris-server` binary also serves the browser client static files (single binary deployment — integrated)
B) The browser client static files are served separately (nginx, a CDN, or `cargo run` on a separate static server)
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 8
What is the expected feature scope of the browser client?

A) Full parity with the desktop client — all 34 weapons, bazaar, ELO rankings, all modes
B) Network play only for the browser client initially — vs-computer (Ernie) and solo practice are desktop-only features
C) Core network play + weapons; ELO and bazaar can be added in a follow-on
X) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 9
What crate name should be used for the new browser client?

A) `battletris-web` — new crate alongside existing `battletris-client` and `battletris-server`
B) `battletris-browser`
C) Keep using `battletris-client` with feature flags to select SDL2 or WASM output
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 10: Security Extension
The browser environment introduces web-specific security concerns (WebSocket origin validation, CORS headers, input sanitisation). Should security extension rules be enforced for this work?

A) Yes — enforce SECURITY rules as blocking constraints
B) No — this is a game, not a production web service; skip security rules
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 11: Property-Based Testing Extension
Should property-based testing rules be enforced for the new browser client and any server changes?

A) Yes — enforce PBT rules as blocking constraints
B) No — skip PBT rules
X) Other (please describe after [Answer]: tag below)

[Answer]: B
