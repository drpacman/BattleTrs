# Component Methods — BattleTrisRs

*Detailed business logic (rotation tables, weapon effect algorithms, AI scoring weights) is deferred to Functional Design per unit. This document defines method signatures and high-level contracts.*

---

## Engine — `battletris-engine`

### `Board`

```rust
// Construction
pub fn new(width: u32, height: u32) -> Board
pub fn clear(&mut self)

// Query
pub fn occupied(&self, x: i32, y: i32) -> bool   // weapon-aware (FALL_OUT extends bounds)
pub fn cell(&self, x: i32, y: i32) -> Option<CellColor>
pub fn height(&self) -> u32
pub fn width(&self) -> u32

// Piece interaction
pub fn can_place(&self, cells: &[(i32, i32)]) -> bool
pub fn place(&mut self, cells: &[(i32, i32)], color: CellColor)

// Line operations
pub fn check_lines(&mut self) -> LinesCleared   // returns lines cleared + die/happy info
pub fn insert_line(&mut self)                   // RISE_UP: push board up, insert garbage at bottom

// Weapon board transformations
pub fn flip_horizontal(&mut self)              // FLIP_OUT: mirror left↔right
pub fn flip_vertical(&mut self)                // UPBYSIDE: flip top↔bottom
pub fn apply_snapshot(&mut self, snap: &BoardSnapshot)  // SWAP: replace board from opponent

// Serialization
pub fn snapshot(&self) -> BoardSnapshot
```

### `PieceKind` (enum, 18 variants)

```rust
// Shape
pub fn cells(&self, rotation: u8) -> &'static [(i32, i32)]  // cell offsets from piece origin
pub fn num_rotations(&self) -> u8

// Appearance
pub fn color(&self) -> CellColor

// Classification
pub fn is_die(&self) -> Option<u8>    // Some(pips) if Die piece, else None
pub fn is_happy(&self) -> bool
pub fn is_weird(&self) -> bool        // true for Dog..LongDong variants

// Spawn
pub fn random(rng: &mut impl Rng, state: &WeaponState) -> PieceKind
// respects active weapons: FEARED_WEIRD → weird pieces; FOUR_BY_FOUR → FourByFour;
// MISSING → no Long; NO_DICE → no Die; NICE_DAY → Happy; BOTTLE → restricted
```

### `ActivePiece`

```rust
pub fn new(kind: PieceKind, board: &Board) -> ActivePiece  // spawns at top-centre
pub fn move_left(&self, board: &Board) -> Option<ActivePiece>
pub fn move_right(&self, board: &Board) -> Option<ActivePiece>
pub fn move_down(&self, board: &Board) -> Option<ActivePiece>
pub fn rotate(&self, board: &Board, reverse: bool) -> Option<ActivePiece>
pub fn hard_drop(&self, board: &Board) -> ActivePiece      // drop to lowest valid position
pub fn cells_absolute(&self) -> Vec<(i32, i32)>            // cells in board coordinates
pub fn has_landed(&self, board: &Board) -> bool
```

### `WeaponKind` (enum, 34 variants) + `WeaponDef`

```rust
// Static weapon data table
pub fn def(kind: WeaponKind) -> &'static WeaponDef   // name, description, price, duration

// WeaponDef fields (compiled-in constants)
pub struct WeaponDef {
    pub name: &'static str,
    pub description: &'static str,
    pub price: u32,
    pub duration: u32,   // measured in lines cleared by the affected player
}

// Effect dispatch — free function (one match arm per weapon)
pub fn apply_weapon_activate(kind: WeaponKind, state: &mut GameState)
pub fn apply_weapon_deactivate(kind: WeaponKind, state: &mut GameState)
pub fn apply_weapon_on_line_cleared(kind: WeaponKind, n: u32, state: &mut GameState)
// Called each time a line is cleared; used for duration countdown and per-line effects (LAWYERS, HATTER)
```

### `WeaponState`

```rust
pub fn new() -> WeaponState
pub fn activate(&mut self, kind: WeaponKind, duration: u32)
pub fn is_active(&self, kind: WeaponKind) -> bool
pub fn tick_line(&mut self, kind: WeaponKind) -> bool  // decrement duration; returns true if just expired
pub fn active_weapons(&self) -> impl Iterator<Item = WeaponKind>
```

### `Score`

```rust
pub fn new() -> Score
pub fn add_lines_cleared(&mut self, lines: &LinesCleared)
// increments score, funds (die pip sum × line multiplier, happy bonus), line count
pub fn deduct_funds(&mut self, amount: u32) -> bool   // returns false if insufficient
pub fn opponent_line_count(&self) -> u32
pub fn combined_lines(&self) -> u32                   // self + opponent; bazaar trigger at multiples of 20
pub fn update_opponent(&mut self, delta_score: i32, delta_lines: u32, delta_funds: i32)
```

### `GameState`

```rust
pub fn new(mode: GameMode) -> GameState   // GameMode: Local(ai_difficulty) | Networked
pub fn phase(&self) -> GamePhase
pub fn board(&self) -> &Board
pub fn current_piece(&self) -> Option<&ActivePiece>
pub fn next_piece(&self) -> PieceKind
pub fn score(&self) -> &Score
pub fn arsenal(&self) -> &Arsenal
pub fn weapon_state(&self) -> &WeaponState
pub fn bazaar_state(&self) -> Option<&BazaarState>

// Primary simulation entry point — called by game-tick thread
pub fn tick(&mut self, input: Option<PlayerInput>, elapsed_ms: u32) -> Vec<GameEvent>

// Network event ingestion — called by game-tick thread when net message arrives
pub fn apply_network_message(&mut self, msg: GameMessage) -> Vec<GameEvent>
```

### `LinesCleared`

```rust
pub struct LinesCleared {
    pub count: u32,                 // 0–4
    pub die_pips: Vec<u8>,          // pip values of die pieces in cleared lines
    pub happy_cleared: bool,        // true if happy-face piece cleared on its spawn turn
}
```

---

## AI — `battletris-engine::ai`

```rust
pub struct Ai {
    pub fn new(difficulty: AiDifficulty) -> Ai
    pub fn choose_placement(&self, board: &Board, piece: PieceKind, weapon_state: &WeaponState)
        -> Placement           // (col: i32, rotation: u8)
    pub fn evaluate_board(&self, board: &Board) -> f32
    // penalises: holes, height variance, covered holes, total height
    pub fn choose_bazaar_purchases(&self, available: &[WeaponKind], funds: u32, score: &Score)
        -> Vec<WeaponKind>     // weapons to buy; up to remaining arsenal slots
    pub fn choose_weapon_to_launch(&self, arsenal: &Arsenal, score: &Score) -> Option<u8>
    // returns arsenal slot index (1-based) or None
}

pub enum AiDifficulty { Easy, Medium, Hard, Super }

pub struct Placement { pub col: i32, pub rotation: u8 }
```

---

## Protocol — `battletris-engine::protocol`

```rust
#[derive(Serialize, Deserialize)]
pub enum GameMessage {
    // Lobby
    Ping,
    Busy { login: String },
    Challenge { login: String },
    Accept,
    Deny,
    Start { opponent_name: String },

    // Active game — game state sync
    Score { delta_score: i32 },
    OpScore { delta_score: i32 },
    Line { count: u32 },
    Board(BoardSnapshot),
    Arsenal(ArsenalSnapshot),
    Funds { delta: i32 },
    WeaponOn(WeaponKind),
    WeaponLaunch(WeaponKind),
    WeaponOff(WeaponKind),

    // Game flow
    Dead,
    StartBazaar,
    EndBazaar,
    GameOver,
    Pause,
    Unpause,
    Idiot,

    // Server ↔ client administration
    QueryResult(GameResult),   // client reports final result for ELO
    PlayerList(Vec<PlayerRecord>),
    RequestPlayerList,
}

// Encode/decode — length-prefixed bincode
pub fn encode(msg: &GameMessage) -> Vec<u8>
pub fn decode(buf: &[u8]) -> Result<GameMessage, ProtocolError>

#[derive(Serialize, Deserialize, Clone)]
pub struct BoardSnapshot {
    pub cells: Vec<Vec<Option<CellColor>>>,  // [row][col]
    pub motivation: Option<WeaponKind>,       // reason for update (SWAP, etc.)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ArsenalSnapshot {
    pub slots: Vec<Option<WeaponKind>>,   // up to 10 slots
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerRecord {
    pub username: String,
    pub elo: i32,
    pub wins: u32,
    pub losses: u32,
}

pub struct GameResult {
    pub winner: String,
    pub loser: String,
    pub winner_score: u32,
    pub loser_score: u32,
    pub lines: u32,
}
```

---

## Renderer — `battletris-client::renderer`

```rust
pub struct Renderer { /* owns sdl2::render::Canvas, font handles, color palette */ }

pub fn new(sdl_context: &sdl2::Sdl, title: &str) -> Result<Renderer, String>

// Top-level render — called from SDL2 thread on each RenderEvent
pub fn render(&mut self, event: &RenderEvent)

// Screen-level render helpers (called from render())
fn render_playing(&mut self, state: &PlayingView)
fn render_bazaar(&mut self, state: &BazaarView)
fn render_lobby(&mut self, state: &LobbyView)
fn render_game_over(&mut self, state: &GameOverView)

// Board rendering
fn render_board(&mut self, board: &BoardSnapshot, origin: (i32, i32), cell_px: u32)
fn render_active_piece(&mut self, piece: PieceKind, pos: (i32, i32), rotation: u8, origin: (i32, i32))
fn render_ghost_piece(&mut self, piece: PieceKind, pos: (i32, i32), rotation: u8, board: &BoardSnapshot, origin: (i32, i32))
fn render_next_piece(&mut self, piece: PieceKind, origin: (i32, i32))
fn render_score_panel(&mut self, score: &ScoreView, origin: (i32, i32))
fn render_arsenal(&mut self, arsenal: &ArsenalSnapshot, origin: (i32, i32))
fn render_opponent_mini(&mut self, snap: &BoardSnapshot, origin: (i32, i32))

// Weapon overlays
fn apply_blind_overlay(&mut self)      // black rectangle over board — BLIND
fn apply_upside_down(&mut self)        // flip SDL rendering context — UPBYSIDE
```

### `RenderEvent` (sent from game-tick thread to SDL2 thread via mpsc)

```rust
pub enum RenderEvent {
    Playing(PlayingView),
    Bazaar(BazaarView),
    Lobby(LobbyView),
    GameOver(GameOverView),
    Quit,
}
// Each view is a cheap snapshot of the state needed for rendering — no &GameState borrow
```

---

## NetworkClient — `battletris-client::net`

```rust
pub struct NetClient {
    pub async fn connect(addr: SocketAddr) -> Result<(NetSender, NetReceiver)>
    // Returns a write half (NetSender) and a tokio task handle feeding an mpsc Receiver
}

pub struct NetSender {
    pub async fn send(&mut self, msg: GameMessage) -> Result<()>
    pub async fn close(&mut self)
}

// Spawned tokio task — forwards server messages to game-tick thread
pub async fn recv_loop(
    stream: tokio::net::tcp::OwnedReadHalf,
    tx: mpsc::Sender<GameMessage>,
)

// Framing — called internally
async fn read_message(reader: &mut impl AsyncRead) -> Result<GameMessage>
async fn write_message(writer: &mut impl AsyncWrite, msg: &GameMessage) -> Result<()>
// Frame format: [u32 big-endian length][bincode payload]
```

---

## Server — `battletris-server`

```rust
// Binary entry point
#[tokio::main]
pub async fn main()   // parses CLI args (clap): `serve [--addr] [--db]` | `players` | `show <name>`

// Server runtime
pub async fn run_server(addr: SocketAddr, db: Arc<Mutex<PlayerDb>>) -> Result<()>
// Accepts connections in a loop; stores waiting client in Option; on second connect, spawns Session

// Session — pairs two clients and relays messages
pub struct Session { ... }
pub async fn run_session(
    client_a: TcpStream,
    client_b: TcpStream,
    db: Arc<Mutex<PlayerDb>>,
) -> Result<()>
// Relays GameMessage in both directions; intercepts QueryResult to update ELO

// Player database
pub struct PlayerDb { records: HashMap<String, PlayerRecord> }

pub fn load(path: &Path) -> Result<PlayerDb>
pub fn save(&self, path: &Path) -> Result<()>
pub fn get(&self, username: &str) -> Option<&PlayerRecord>
pub fn get_or_create(&mut self, username: &str) -> &mut PlayerRecord
pub fn update_elo(&mut self, result: &GameResult)

// ELO computation (K=32)
pub fn compute_elo_delta(winner_elo: i32, loser_elo: i32) -> i32
```
