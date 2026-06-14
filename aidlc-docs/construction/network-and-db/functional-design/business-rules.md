# Business Rules — Unit 3: network-and-db

## BR-NET-01: Username Uniqueness (Q5=A)
A player's username must be unique among all currently-connected clients in the server process.
- If a `Hello { name }` arrives and `name` is already held by a live connection in any active or pending session, the server sends `NameTaken` and closes the incoming TCP connection immediately.
- Usernames from finished sessions are released and may be reused by new connections.
- Comparison is case-sensitive (`"Alice"` and `"alice"` are different names).

## BR-NET-02: Minimum Name Length
A username must be between 1 and 16 characters (inclusive). Names outside this range cause the server to close the connection without sending `NameTaken` (connection is simply dropped as invalid).

## BR-NET-03: Server Pairs Exactly Two Players
A game session requires exactly two connected players. The server does not support spectators or three-player games. If more than two players are connected, the first two are paired; additional clients wait until the current session finishes.

## BR-NET-04: GameStart Is the Exclusive Game-Start Signal
A client must not start game logic until it receives `GameStart` from the server. `GamePhase` must not advance from `WaitingForOpponent` to `Playing` for any other reason in network mode.

## BR-NET-05: Server Is Bazaar Authority (Q2=B)
In network mode, clients must not trigger bazaar internally from `LinesCleared` messages. The bazaar opens only when the client receives `BazaarOpen` from the server. The client's `add_op_lines()` return value must be ignored in network mode (or the call skipped entirely).

## BR-NET-06: Combined Lines Threshold
The server sends `BazaarOpen` to both clients every time `combined_lines` crosses a new multiple of 20.
- Threshold 1: combined_lines >= 20 → send BazaarOpen, next threshold = 40
- Threshold 2: combined_lines >= 40 → send BazaarOpen, next threshold = 60
- (and so on)
- `LinesCleared` messages from both clients contribute equally to `combined_lines`.

## BR-NET-07: BazaarEnd Must Be Relayed
When the server receives `BazaarEnd` from one client, it relays `BazaarEnd` to the other client. This mirrors the vs-Ernie behaviour where `ernie_bazaar_done()` is called on receiving `BazaarEnd`.

## BR-NET-08: Disconnection Reconnection Window (Q3=C)
If the TCP connection for one client drops during an active game:
- The server starts a 15-second timer.
- The server sends `PeerDisconnected` to the remaining client.
- If the disconnected client reconnects (TCP + `Hello { same_name }`) within 15 seconds, the game resumes.
- If the timer expires, the server sends `GameVoid` to the remaining client. No ELO update occurs.
- Voluntary quit (client sends `QuitToTitle` then closes TCP) is treated identically to an accidental drop.

## BR-NET-09: ELO Update Only on Clean GameOver
ELO ratings are updated only when the server intercepts a `GameOver` message during a `Playing` session. ELO is **not** updated for:
- Voided games (disconnection timeout)
- Games where either client never advanced past `WaitingForOpponent`

## BR-NET-10: ELO Floor
A player's ELO rating must never fall below 100. If the computed new rating is below 100, it is clamped to 100.

## BR-NET-11: Auto-Create Player Record
When a `Hello { name }` is accepted and `name` does not exist in `PlayerDb`, a new `PlayerRecord` is created with `elo = 1200, wins = 0, losses = 0, draws = 0` before the game begins.

## BR-NET-12: PlayerDb Flush is Synchronous
After every ELO update, `PlayerDb` must be written to disk before sending `GameOver` to the clients. This prevents ELO loss if the server crashes immediately after a game.

## BR-NET-13: Protocol Frame Completeness
The server and clients must buffer incoming bytes until a complete frame is available (4-byte length prefix + full payload). Partial frames must never be passed to `decode`. A frame larger than 1 MB is rejected as malformed (connection closed).

## BR-NET-14: Weapon Events Are Relayed Transparently
`WeaponLaunched`, `WeaponReflected`, and `FundsReceived` messages from one client are forwarded to the other client without modification. The server does not interpret weapon semantics.

## BR-NET-15: Board Sync on Lock Only (Q4=A)
Clients send `BoardUpdate { snapshot }` only when a piece locks (same as vs-Ernie). The server relays this to the opponent. No intermediate piece-position messages are sent over the network.

## BR-NET-16: ScoreUpdate Relay
`ScoreUpdate { score, lines, funds }` sent by one client is relayed to the opponent so the opponent's stats panel can show live score/lines data (funds shown only if spy weapon active, as per existing logic).

## BR-NET-17: Server Has No Game Logic
The server does not evaluate board states, apply weapon effects, or make gameplay decisions. Its sole game-aware actions are:
1. Counting `combined_lines` to emit `BazaarOpen`
2. Intercepting `GameOver` to compute ELO
3. Starting the 15-second disconnect timer

Everything else is transparent relay.
