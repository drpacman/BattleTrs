# Business Logic Model — Unit 3: network-and-db

## 1. Title Screen Mode Selection (Q1=B)

The title screen is extended with a two-item menu. The player uses arrow keys or number keys to choose:

```
1 - VS ERNIE   (local AI game — existing behaviour)
2 - NETWORK    (two-player LAN game)
```

Pressing Enter on option 1 starts the vs-Ernie game exactly as today.
Pressing Enter on option 2 transitions to the **Connection Screen**.

### Connection Screen Flow

The connection screen shows three editable text fields:
- Server IP (e.g., `192.168.1.10`)
- Port (default: `7000`)
- Your name (e.g., `Alice`)

Tab/Enter cycles focus between fields. Once all fields are populated, Enter confirms and attempts connection. The screen transitions to `Connecting...` status, then `Waiting for opponent...` once `Welcome` is received from the server.

The client navigates backwards:
- Escape on the connection screen → back to title menu.
- `NameTaken` response → show error "Name already in use" and allow re-entry.

---

## 2. Server Connection and Pairing

```
Client A connects (TCP)
  → sends Hello { name: "Alice" }
Server checks name uniqueness in active sessions:
  if taken → send NameTaken, close connection
  if free  → send Welcome { assigned_name: "Alice" }
           → add to pending list

Client B connects (TCP)
  → sends Hello { name: "Bob" }
Server:
  → send Welcome { assigned_name: "Bob" }
  → send GameStart to Client A
  → send GameStart to Client B
  → session state = Playing
```

Only exactly two clients are paired per session. If a third client tries to connect while a session is Playing, they wait in the pending list for the next available pairing slot.

---

## 3. Message Relay

During `Playing` state, the server forwards every `GameMessage` it receives from one client to the other client, **except**:
- `LinesCleared` — intercepted to update `combined_lines`; then relayed.
- `GameOver` — intercepted to trigger ELO update; then relayed (with ELO delta).
- `Hello` / `Welcome` / `GameStart` — connection-phase only; not relayed.

All other messages (`BoardUpdate`, `ScoreUpdate`, `WeaponLaunched`, `WeaponReflected`, `BazaarEnd`, `FundsReceived`, etc.) are forwarded transparently.

---

## 4. Server-Arbitrated Bazaar (Q2=B)

The server is the sole authority on when the bazaar opens.

```
Server receives LinesCleared { count } from either client:
  combined_lines += count
  relay LinesCleared to other client (normal relay)
  if combined_lines >= next_bazaar_threshold:
    next_bazaar_threshold += 20
    send BazaarOpen to Client A
    send BazaarOpen to Client B

Each client receives BazaarOpen:
  → open_bazaar_now() on own GameState
  → player shops and exits
  → send BazaarEnd to server

Server receives BazaarEnd from each client:
  → relay BazaarEnd to the other client (so their GameState sees ernie_bazaar_done())
```

**Key implication**: The client must NOT trigger bazaar from `LinesCleared` messages in network mode. The `add_op_lines()` path that currently triggers bazaar in the vs-Ernie client must be suppressed when connected to a server (the server provides `BazaarOpen` instead).

---

## 5. Disconnection and Reconnection (Q3=C)

```
Server detects TCP connection dropped for one client:
  session state = Reconnecting
  disconnect_deadline = now + 15 seconds
  send PeerDisconnected to remaining client

Remaining client:
  → shows "Opponent disconnected. Waiting 15s..." overlay
  → game continues ticking (pieces still fall) but opponent board freezes

If original client reconnects within deadline:
  → sends Hello { name } again
  → server matches name to active session
  → session state = Playing
  → send GameStart to reconnecting client (resumes)
  → send PeerReconnected (new variant) to remaining client (overlay dismissed)

If deadline passes:
  → session state = Finished (void)
  → send GameVoid to remaining client
  → no ELO update for either player
  → server removes session
```

**Note**: `PeerReconnected` is a small additional `GameMessage` variant needed to dismiss the overlay on the waiting client.

---

## 6. Game Over and ELO Update

```
Server receives GameOver { winner_id, ... } from either client:
  determine winner_name and loser_name from session
  compute ELO delta:
    expected = 1.0 / (1.0 + 10^((loser_elo - winner_elo) / 400))
    winner_delta = round(32.0 * (1.0 - expected))
    loser_delta  = round(32.0 * (0.0 - (1.0 - expected)))
  update PlayerDb:
    winner.elo += winner_delta; winner.wins += 1
    loser.elo  += loser_delta;  loser.losses += 1
  flush PlayerDb to players.json
  send GameOver (extended with elo_delta fields) to BOTH clients
  session state = Finished
```

**ELO formula**: Standard ELO, K=32, starting rating 1200.  
`winner_delta` is always positive; `loser_delta` is always negative (or zero in extreme mismatch).

---

## 7. Protocol Encoding

```
encode(msg: &GameMessage) -> Vec<u8>:
  payload = bincode::serialize(msg)
  prefix  = (payload.len() as u32).to_be_bytes()
  return prefix ++ payload

decode(buf: &[u8]) -> Result<GameMessage, ProtocolError>:
  if buf.len() < 4: return Err(NeedMoreData)
  len = u32::from_be_bytes(buf[0..4])
  if buf.len() < 4 + len: return Err(NeedMoreData)
  return bincode::deserialize(&buf[4..4+len])
```

The server and both clients use identical `encode`/`decode`. The server treats its TCP streams as byte streams and accumulates data in a read buffer until a complete frame is available.

---

## 8. Player Database

`PlayerDb` is a `HashMap<String, PlayerRecord>` held in an `Arc<Mutex<PlayerDb>>` shared across all server tasks.

- **Load**: On server start, read `players.json`; if absent, start with empty map.
- **Auto-create**: When a new name is seen during `Hello`, insert a default `PlayerRecord { elo: 1200, wins: 0, losses: 0, draws: 0 }` before the game starts.
- **Flush**: After every ELO update, serialise the entire map to `players.json`.
- **Query commands**: `players` lists all records sorted by ELO descending; `show <name>` prints one record.

---

## 9. Client Network Architecture

The client spawns two tokio tasks inside a tokio runtime (single-threaded is sufficient):

```
TcpStream
  |
  +-- recv_task:  reads frames from TCP → decodes GameMessage → sends to game_loop via mpsc
  +-- send_task:  receives GameMessage from game_loop via mpsc → encodes → writes to TCP

game_loop (std::thread):
  -- reads from recv channel (same interface as current Ernie channels)
  -- writes to send channel
```

The `ErnieChannels` struct in `game_loop` is replaced by a `NetChannels` struct with the same field names (`from_peer: Receiver<GameMessage>`, `to_peer: SyncSender<GameMessage>`). The game_loop code needs no structural changes — only the channel source changes from the Ernie thread to the net tasks.

The tokio runtime is created in `main` and lives for the duration of the network connection.
