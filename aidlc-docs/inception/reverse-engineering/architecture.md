# System Architecture — BattleTris

## System Overview

BattleTris is a two-player competitive networked Tetris game from 1994 (Brown University CS32 final project). The system has two major runtime components: (1) the game client binary (`BattleTris`) run by each player, and (2) the server daemon pair (`btserverd` + `btslaved`) that routes traffic and maintains the player database.

The original implementation uses X11/Motif for rendering (Unix-specific), POSIX TCP sockets with Xt event loop integration, and custom pre-STL C++ data structures. For the Rust port, X11/Motif must be replaced with a cross-platform renderer and the Xt event loop replaced with a Rust game loop or async runtime.

## Architecture Diagram

```
+-----------------------------------------------------------------------+
|  CLIENT A (Mac)                    CLIENT B (Windows)                 |
|                                                                       |
|  +----------------------+          +----------------------+          |
|  |   BattleTris Client  |          |   BattleTris Client  |          |
|  | BTGame               |          | BTGame               |          |
|  | BTBoardManager       |          | BTBoardManager       |          |
|  | BTPieceManager       |          | BTPieceManager       |          |
|  | BTWeaponManager      |          | BTWeaponManager      |          |
|  | BTScoreManager       |          | BTScoreManager       |          |
|  | BTCommManager        |          | BTCommManager        |          |
|  | BTComputer (opt)     |          | BTComputer (opt)     |          |
|  | Renderer (X11/Motif) |          | Renderer (TBD-Rust)  |          |
|  +----------+-----------+          +----------+-----------+          |
|             |  TCP port 4404                  |  TCP port 4404       |
|             +----------------+----------------+                      |
|                              |                                        |
|                    +---------v----------+                            |
|                    |    btslaved        |                            |
|                    |  (relay daemon)    |                            |
|                    +---------^----------+                            |
|                              |  Unix socket or TCP                   |
|                    +---------v----------+                            |
|                    |    btserverd       |                            |
|                    |  (master daemon)   |                            |
|                    |   + BTDB           |                            |
|                    |  (player database) |                            |
|                    +--------------------+                            |
+-----------------------------------------------------------------------+
```

## Component Descriptions

### BTGame
- **Purpose**: Central game state machine
- **Responsibilities**: Drives game loop via 5 timer callbacks (DROP, SLIDE, SLICK, HATTER, JEOPARDY), handles keyboard input, coordinates all subsystems, manages bazaar synchronization with opponent
- **Dependencies**: BTBoardManager, BTPieceManager, BTScoreManager, BTSoundManager, BTCommManager, BTWeaponManager, BTBazaar, BTComputer
- **Type**: Application — core game logic

### BTBoardManager
- **Purpose**: Manages the 10x28 game grid
- **Responsibilities**: Tracks placed boxes via 2D pointer array, collision detection (`occupied()`), line clearing (`checkLines()`), board state effects (flip on horizontal/vertical, insert line), weapon-aware bounds (FALL_OUT weapon expands effective board)
- **Dependencies**: BTWeaponManager, BTBoxManager
- **Type**: Application — core game logic

### BTPiece / BTPieceManager
- **Purpose**: Piece spawning, movement, rotation, placement
- **Responsibilities**: 18 piece types (7 standard + die + happy + 9 weird) with polymorphic rotation; random selection weighted by weapon state; piece movement and collision; landing detection
- **Dependencies**: BTBoardManager
- **Type**: Application — core game logic

### BTWeaponManager
- **Purpose**: Active weapon state tracking and effect dispatching
- **Responsibilities**: Maintains `BTActive[BT_MAX_WEAPONS]` array of per-weapon active flags and `remaining_[]` line counters, updates arsenal UI, launches weapons via `launchWeapon()`, receives weapon-on/off events from network
- **Dependencies**: BTCommManager, BTArsenal
- **Type**: Application — game feature

### BTCommManager
- **Purpose**: Active-game network communication bridge
- **Responsibilities**: Sends score updates, board state snapshots, weapon events, arsenal state over TCP; receives same from opponent; implements BTRingNode message bus; supports both remote (StreamSocket) and local sibling CommManager mode (for player-vs-AI)
- **Dependencies**: StreamSocket, PacketBuffer, BTBoard, BTScore, BTWeapon, BTArsenal
- **Type**: Application — networking

### BTNetManager
- **Purpose**: Lobby and server connection management
- **Responsibilities**: Connects to btserverd, retrieves network roster and player database, issues challenges, handles challenge accept/deny, bridges to BTCommManager when game starts
- **Dependencies**: StreamSocket, PacketBuffer, BTCommManager, BTNetworkEntry, BTPlayer
- **Type**: Application — lobby/networking

### BTComputer (Ernie)
- **Purpose**: AI opponent
- **Responsibilities**: Exhaustive search of all piece rotations/positions, scores states by penalizing holes and height variance (`computeValue()`), plans weapon purchases via `BTCOrders` queue, participates fully on the BTRingNode message bus
- **Dependencies**: BTBoardManager, BTPieceManager, BTScoreManager, BTWeaponManager, BTCommManager
- **Type**: Application — AI

### BTDisplay / widget/
- **Purpose**: X11/Motif rendering and UI layer
- **Responsibilities**: Board drawing, piece rendering, score display, bazaar UI, dialogs; X11 color/palette management; Xt event loop integration for async socket callbacks (XtSocketCB)
- **Dependencies**: X11, Motif (libXm), XQuartz (macOS) / system X11 (Linux)
- **Type**: Platform-specific — must be replaced in Rust port

### btserverd / btslaved
- **Purpose**: Server-side relay and database management
- **Responsibilities**: btserverd (port 4404) authenticates clients with magic cookies, spawns btslaved per connected client pair; btslaved relays game packets between the two paired clients via Unix socket / TCP
- **Dependencies**: StreamSocket, BTDB, BTNetworkEntry
- **Type**: Server daemon

### BTDB
- **Purpose**: Persistent player statistics and network presence database
- **Responsibilities**: Hash-based flat-file DB with read/write locking (BTDBLock/BTDBReadLock/BTDBWriteLock); stores BTPlayer (ELO, wins, losses) and BTNetworkEntry (online presence) records
- **Dependencies**: POSIX file I/O
- **Type**: Data persistence

## Data Flow — Network Game Session

```
Client A             btslaved              Client B
   |                    |                      |
   |-- BT_QUER_CONN --->|                      |
   |-- BT_QUER_NETDB -->|                      |
   |<- BT_RESP_NETDB ---|                      |
   |-- BT_CHALL(B) ---->|--- BT_CHALL(A) ----->|
   |<- BT_ACCPT --------|<-- BT_ACCPT ----------|
   |-- BT_START ------->|--- BT_START --------->|
   |                    |                      |
   |  [GAME LOOP]       |     [GAME LOOP]      |
   |-- BT_SCORE ------->|--- BT_OP_SCORE ------>|
   |-- BT_BOARD ------->|--- BT_BOARD --------->|
   |-- BT_LINE -------->|--- BT_BOARD --------->|
   |-- BT_WPN_LAUNCH -->|--- BT_WPN_ON -------->|
   |-- BT_WPN_OFF ----->|--- BT_WPN_OFF -------->|
   |-- BT_START_BAZ --->|--- BT_START_BAZ ------>|
   |-- BT_END_BAZ ----->|--- BT_END_BAZ -------->|
   |-- BT_DEAD -------->|--- BT_DEAD ----------->|
   |-- BT_QUER_RESULT ->|                      |
```

## Integration Points

- **Server**: btserverd at configurable host:port (default `poptart.eng.sun.com:4404`, port 4404)
- **Audio**: /dev/audio (Solaris) — stubbed on all modern platforms
- **Rendering**: X11/Motif (Unix only; replaced in Rust port)
- **Database**: Flat files on server filesystem (no external DB engine)

## Infrastructure Components

- **Deployment**: Single btserverd process + per-connection btslaved child processes (Unix fork/spawn model)
- **Protocol**: Custom binary framing over TCP (BTToken values, fixed-size structs)
- **Default Port**: 4404 TCP
