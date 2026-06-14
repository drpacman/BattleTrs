# Domain Entities — Unit 3: network-and-db

## PlayerRecord

Persisted per named player in the flat-file database.

| Field | Type | Notes |
|-------|------|-------|
| `name` | `String` | Unique username; key in the HashMap |
| `elo` | `i32` | ELO rating; starts at 1200 on first registration |
| `wins` | `u32` | Total ranked wins |
| `losses` | `u32` | Total ranked losses |
| `draws` | `u32` | Reserved; BattleTris has no draw mechanism (always 0) |

**Persistence**: `HashMap<String, PlayerRecord>` serialised as JSON to `players.json` in the server's working directory. Loaded on server start; flushed after every ELO update.

---

## GameSession (server-side, in memory only)

Tracks one paired game between two connected clients.

| Field | Type | Notes |
|-------|------|-------|
| `player_a_name` | `String` | Name of first-connected player |
| `player_b_name` | `String` | Name of second-connected player |
| `combined_lines` | `u32` | Sum of all `LinesCleared.count` from both clients this game |
| `next_bazaar_threshold` | `u32` | Next multiple of 20 that triggers `BazaarOpen` |
| `state` | `SessionState` | Enum: `WaitingForBoth`, `WaitingForOne`, `Playing`, `BazaarPending`, `Reconnecting`, `Finished` |
| `disconnect_deadline` | `Option<Instant>` | Set when a client drops; `Some(now + 15s)` |

---

## GameMessage (extended for network)

All existing variants are retained. The following variants are **added** for the network protocol:

| Variant | Direction | Fields | Purpose |
|---------|-----------|--------|---------|
| `Hello` | Client → Server | `name: String` | Client identifies itself on connect |
| `Welcome` | Server → Client | `assigned_name: String` | Server confirms accepted name |
| `GameStart` | Server → Both | *(none)* | Both clients begin the game simultaneously |
| `BazaarOpen` | Server → Both | *(none)* | Server-arbitrated bazaar trigger (Q2=B) |
| `PeerDisconnected` | Server → Remaining | *(none)* | Peer dropped; 15-second reconnect window started |
| `GameVoid` | Server → Remaining | *(none)* | Reconnect window expired; game cancelled, no ELO change |
| `NameTaken` | Server → Client | *(none)* | Duplicate username rejected; server closes the connection |

**Wire format**: 4-byte big-endian length prefix + `bincode`-encoded `GameMessage`.  
All `GameMessage` variants receive `#[derive(Serialize, Deserialize)]`.

---

## GameResult (used internally by server after GameOver)

| Field | Type | Notes |
|-------|------|-------|
| `winner_name` | `String` | Name of the winning player |
| `loser_name` | `String` | Name of the losing player |
| `winner_elo_before` | `i32` | For computing delta display |
| `loser_elo_before` | `i32` | For computing delta display |
| `winner_elo_after` | `i32` | After K=32 ELO update |
| `loser_elo_after` | `i32` | After K=32 ELO update |

`GameResult` is computed by `elo::compute_result` and used to update `PlayerDb` and to build the `GameOver` message sent back to both clients (extended with ELO delta fields).

---

## TitleMenuChoice (client-side UI state)

Represents what the player has selected on the title screen.

| Variant | Meaning |
|---------|---------|
| `VsErnie` | Launch local game against Ernie AI (existing behaviour) |
| `NetworkGame` | Show connection screen; prompt for server IP, port, and name |

---

## ConnectionScreenState (client-side UI state)

Tracks progress through the connection flow.

| Field | Type | Notes |
|-------|------|-------|
| `server_ip` | `String` | Typed by the player (editable text field) |
| `port` | `String` | Default "7000" |
| `player_name` | `String` | Typed by the player |
| `active_field` | `ConnectionField` | Which of the three fields currently has cursor |
| `status` | `ConnectionStatus` | `Idle`, `Connecting`, `WaitingForOpponent`, `NameTaken`, `Error(String)` |
