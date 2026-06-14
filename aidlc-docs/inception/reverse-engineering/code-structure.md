# Code Structure — BattleTris

## Build System
- **Type**: autoconf / Sun make
- **Configuration**: `usr/src/configure.in` (autoconf), `usr/src/Makeinclude.in` (generated), per-directory Makefiles
- **Build Entry Point**: `cd BattleTris/usr/src && ./configure && make`
- **Install Layout**: builds into `usr/bin/`, `usr/lib/`, `usr/include/`

## Source Module Hierarchy

```
BattleTris/usr/src/
+-- game/          Main game client (~24K lines C++)
+-- daemons/       Server daemons (btserverd, btslaved, btserverd client)
+-- db/            Player database (BTDB, BTPlayer, BTNetworkEntry, BTGameStats)
+-- widget/        X11/Motif widget wrappers (BTDisplay, drawing areas, forms)
+-- sockets/       TCP socket abstraction (StreamSocket, PacketBuffer, Address)
+-- stdlib/        Custom template containers (List, Block, BTStack, BTRingNode)
+-- audio/         Solaris /dev/audio interface (DevAudio — stub on modern platforms)
+-- signals/       POSIX signal handling (SigHandler, SigReceiver)
+-- btref/         Admin CLI for player database (btref binary)
+-- share/         Game data: btweapons.db, BattleTris.ad, sounds/
+-- art/           PPM/XPM/XBM artwork
+-- man/           Unix man pages
```

## Key Classes / Modules

### game/ — Core Game Logic

| File | Class/Role |
|------|-----------|
| BattleTris.C/H | main(), X11/Motif app init, resource loading, startup |
| BTGame.C/H | Central game state machine; 5 timer callbacks; keyboard handler |
| BTBoardManager.C/H | 10x28 board grid; collision; line clearing; weapon effects |
| BTPiece.C/H | Piece hierarchy: 18 types (El, RevEl, SldLft, SldRt, Long, Plug, Box, Die, Happy, Dog, RevDog, Cap, Wall, Tower, Star, WeirdLong, FourByFour, LongDong) |
| BTPieceManager.C/H | Piece selection (weighted random), spawn, lifecycle |
| BTWeaponManager.C/H | BTActive[] flags; weapon duration countdown; arsenal UI |
| BTCommManager.C/H | BTRingNode bridge to network; send/recv score, board, weapons |
| BTNetManager.C/H | Lobby: server connection, roster, challenge/accept protocol |
| BTScoreManager.C/H | Score, lines, funds tracking and display |
| BTSoundManager.C/H | Audio playback (stub on non-Solaris) |
| BTBazaar.C/H | Weapons purchasing UI (shown every 20 combined lines) |
| BTArsenal.C/H | Player's active weapon list (up to BT_ARSENAL_SIZE=10) |
| BTComputer.C/H | AI opponent (Ernie): exhaustive placement search + weapon strategy |
| BTBoard.C/H | Board state snapshot for network transmission |
| BTBox.C/H | Individual board cell (color, identity) |
| BTScore.C/H | Score/lines/funds data structure |
| BTWeapon.H | Weapon data (token, name, description, price, duration) |
| BTProtocol.H | BTToken and BTWeaponToken enums — complete wire protocol |
| BTConstants.H | Game constants: board size, piece IDs, color IDs, timing |
| BTRingNode.C/H | Internal message bus: send/receive BTRingPacket between nodes |
| BTTimeOut.C/H | Xt-integrated timer callbacks (BT_TIMEOUT_CALLBACK macro) |
| BTMovePath.C/H | AI move path computation |
| BTCBoard.C/H | AI board simulation board |
| BTRecon.C/H | "Recon" opponent board display (spy weapon) |
| BTStartup.C/H | Startup/login UI, server connection, BTNetManager init |
| BTChallenge/BTChallengeDialog | Challenge issuance and response UI |
| BTBiff.C/H | Notification/status UI widget |
| BTList.C/H | Game-specific list utilities |
| BTScore.C/H | Score data structure and serialization |
| BTStopwatch.H | Timing utilities |
| Mem.C/H | Memory allocation utilities |
| PPMReader.C/H | PPM image loader for art assets |

### widget/ — X11/Motif UI Abstraction

| File | Purpose |
|------|---------|
| BTDisplay.C/H | X11 display context, GC management |
| BTXDisplay.C/H | X11-specific display implementation |
| BTDrawingAreaWidget.C/H | Motif drawing area widget wrapper |
| BTFormWidget.C/H | Motif form (layout container) wrapper |
| BTFrameWidget.C/H | Motif frame widget wrapper |
| BTLabelWidget.C/H | Text label widget |
| BTTextWidget.C/H | Text display widget |
| BTScrolledTextWidget.C/H | Scrollable text area |
| BTScrolledListWidget.C/H | Scrollable list (for player roster) |
| BTPushButtonWidget.C/H | Button widget |
| BTSliderWidget.C/H | Slider widget |
| BTPixmap.C/H | Pixmap (image) management |
| BTXPalette.C/H | Color palette management |
| BTMessageDlog.C/H | Message dialog |
| BTWidget.C/H | Base widget class |
| BTXmUtils.C/H | Motif utility functions |
| XtSocketCB.C/H | Xt-integrated socket event callbacks |

### sockets/ — Network Layer

| File | Purpose |
|------|---------|
| StreamSocket.C/H | TCP socket (connect, listen, accept, send, recv, sendfd, recvfd) |
| Socket.H | Abstract socket base class with callback (SocketCB) |
| PacketBuffer.C/H | Framed message buffering over stream socket |
| Address.C/H | InetAddress and UnixAddress abstractions |
| SocketCB.H | Socket callback interface |
| XtSocketCB.C/H | Xt event loop socket integration |

### db/ — Persistence

| File | Purpose |
|------|---------|
| BTDB.C/H | Hash-based flat-file database engine with R/W locking |
| BTDBLock/ReadLock/WriteLock.H | POSIX file locking wrappers |
| BTDBRecord.C/H | Base DB record (key, data buffer) |
| BTPlayer.C/H | Player record (ELO, wins, losses, stats) |
| BTPlayerRecord.C/H | Serialization for BTPlayer |
| BTNetworkEntry.C/H | Network presence record (hostname, status) |
| BTNetwork.H | Network database declarations |
| BTGameStats.C/H | Per-game stats (score, lines) for result recording |
| BTConfigFile.C/H | Config file parser |
| ParsedFile.C/H | Generic config file parsing |

### stdlib/ — Custom Containers

| File | Purpose |
|------|---------|
| List.C/H + ListElement + ListIter | Doubly-linked list template |
| AbsList.C/H + element + iter | Abstract base for list |
| Block.C/H | Dynamic array template (like std::vector) |
| BTStack.C/H | Stack template (used in AI path planning) |
| BTRingNode.C/H (game/) | Message bus node; objects can `send()` and `receive()` packets |

### daemons/

| File | Purpose |
|------|---------|
| btserverd.C/H | Master server daemon (main entry, listens on port 4404) |
| BTServer.C/H | Server logic: client auth, DB operations, slave spawning |
| btslaved.C/H | Per-connection slave daemon (main entry) |
| BTSlave.C/H | Slave logic: relay packets between paired clients |
| BTDBServer.C/H | Database-facing server operations |
| BTSDClient.C/H | Client-side handler for slave daemon messages |
| BTMDSlave.C/H | Master daemon's view of a slave daemon |

## Design Patterns

### BTRingNode — Internal Message Bus
- **Location**: game/BTRingNode.C/H; used by BTGame, BTBoardManager, BTCommManager, BTWeaponManager, BTComputer
- **Purpose**: Decouples game subsystems; subsystems broadcast events (BT_SCORE, BT_LINE, BT_WPN_ON, etc.) on the ring; each node receives and routes packets
- **Implementation**: Circular doubly-linked list of BTRingNode objects; `send(token, data)` broadcasts to all nodes; each node has a `receive(BTRingPacket*)` handler

### Xt Timer Callbacks — Game Loop
- **Location**: game/BTGame.C/H, BTTimeOut.H, BT_TIMEOUT_CALLBACK macro
- **Purpose**: Drives piece falling (DROP_TIMEOUT), horizontal sliding (SLIDE_TIMEOUT), and weapon effect timers
- **Implementation**: Wraps XtAppAddTimeOut / XtRemoveTimeOut; game loop is event-driven via Xt rather than a polling loop

### Polymorphic Pieces
- **Location**: game/BTPiece.C/H
- **Purpose**: Each piece type customizes its shape via overridden `construct()` and optionally `rotate()`
- **Implementation**: BTPiece base class with 18 concrete subclasses; map_ is a 2D array of BTBox pointers

## Existing Files — Complete Inventory

See the `find BattleTris/usr/src -name "*.C" -o -name "*.H"` listing in the project root for the full ~150-file list. Key modification candidates for the Rust port are in `game/`, `sockets/`, `db/`, and `daemons/`.

## Critical Dependencies

### X11 / Motif
- **Usage**: All rendering and event loop (XtApp, XmCreate*, etc.)
- **Purpose**: Entire UI layer — must be replaced in Rust port with a cross-platform library

### Xt Event Loop
- **Usage**: Socket callbacks (XtAppAddInput), timer callbacks (XtAppAddTimeOut)
- **Purpose**: Async I/O integration — must be replaced in Rust port

### POSIX TCP Sockets
- **Usage**: StreamSocket wraps socket/connect/listen/accept/send/recv
- **Purpose**: All game networking — directly portable to Rust's std::net or tokio

### Sun audio (/dev/audio)
- **Usage**: BTSoundManager opens /dev/audio for PCM playback
- **Purpose**: Sound effects — stub required for all non-Solaris platforms
