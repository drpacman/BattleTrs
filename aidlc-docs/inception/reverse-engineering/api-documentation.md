# API Documentation — BattleTris

## Network Wire Protocol

The entire game uses a custom binary framing protocol over TCP. All messages are identified by a `BTToken` byte, followed by optional data.

### BTToken Values (BTProtocol.H)

#### Game-to-Game Messages (via btslaved relay)

| Token | Value | Description | Data |
|-------|-------|-------------|------|
| BT_NULL | 0 | Connection terminated | none |
| BT_ERR | 1 | Network error | none |
| BT_SCORE | 10 | Score update | short (score delta) |
| BT_OP_SCORE | 11 | Opponent score update | short |
| BT_LINE | 12 | Line cleared (increase count) | none |
| BT_BOARD | 13 | Board state snapshot | BTBoard struct |
| BT_ARSENAL | 14 | Arsenal state | BTArsenal struct |
| BT_FUNDS | 15 | Funds update | short |
| BT_WPN_ON | 16 | Weapon activated on me | BTWeaponToken |
| BT_WPN_LAUNCH | 17 | I'm launching a weapon | BTWeaponToken |
| BT_WPN_OFF | 18 | Weapon deactivated | BTWeaponToken |
| BT_DEAD | 19 | Opponent died — you win | none |
| BT_START_BAZ | 20 | Entering bazaar | none |
| BT_END_BAZ | 21 | Done with bazaar | none |
| BT_GAME_OVER | 22 | My game is over | none |
| BT_AIRSLIDE | 23 | Air slide move made | none |
| BT_LAWYER | 24 | Lawyers' Delite event | none |

#### Lobby/Challenge Messages

| Token | Value | Description | Data |
|-------|-------|-------------|------|
| BT_PING | 30 | Request current status | none |
| BT_BUSY | 31 | Currently in game | login string |
| BT_CHALL | 32 | Challenge issued | login string |
| BT_ACCPT | 33 | Challenge accepted | none |
| BT_DENY | 34 | Challenge declined | none |
| BT_START | 35 | Game start signal | none |
| BT_PAUSE | 50 | Game paused | none |
| BT_IDIOT | 51 | Idiot move detected | none |
| BT_CONDOR_OFF | 52 | Condor weapon off | none |

#### Server-Client Protocol (btserverd <-> btslaved <-> client)

| Token | Value | Description | Data |
|-------|-------|-------------|------|
| BT_LOCAL | 60 | Local client connecting | none |
| BT_REMOTE | 61 | Remote client connecting | none |
| BT_COOKIE_GOOD | 62 | Auth cookie accepted | none |
| BT_COOKIE_BAD | 63 | Auth cookie rejected | none |
| BT_ACCEPTED | 64 | Client accepted | none |
| BT_REJECTED | 65 | Client rejected | none |
| BT_OBEY_ME | 66 | Server to slave: obey | none |
| BT_I_OBEY | 67 | Slave to server: will comply | none |
| BT_NEWCLIENT | 68 | Server assigns new client to slave | none |
| BT_CLIENTOK | 69 | Slave accepted client | none |
| BT_CLIENTBAD | 70 | Slave rejected client | none |
| BT_DISCONNECT | 71 | Client disconnecting | none |
| BT_HARIKARI | 72 | Server tells slave to exit | none |
| BT_QUER_COOKIE | 73 | Server requests cookie from client | none |
| BT_QUER_CONN | 74 | Client connection request | BTNetworkEntry |
| BT_QUER_NETDB | 75 | Client requests network roster | none |
| BT_QUER_PLYDB | 76 | Client requests player database | none |
| BT_QUER_VERIFY | 77 | Client verifies network entry | key |
| BT_QUER_UPDATE | 78 | Client updates network status | none |
| BT_QUER_RESULT | 79 | Client records game result | BTGameStats |
| BT_RESP_COOKIE | 80 | Server sends cookie | BT_COOKIELEN bytes |
| BT_RESP_VERIFY | 81 | Server verification response | unsigned short |
| BT_RESP_DBLEN | 82 | Number of DB records | unsigned long |
| BT_RESP_NETDB | 83 | Network roster records | buffer of BTNetworkEntry |
| BT_RESP_PLYDB | 84 | Player database records | buffer of BTPlayer |

### BTWeaponToken Values (weapon identifiers)

| Token | Value | Effect |
|-------|-------|--------|
| BT_FEARED_WEIRD | 0 | Opponent gets weird pieces |
| BT_FOUR_BY_FOUR | 1 | Opponent gets 4x4 pieces |
| BT_HATTER | 2 | Remove opponent's pieces |
| BT_UPBYSIDE | 3 | Flip opponent's board upside-down |
| BT_FALL_OUT | 4 | Extend opponent's board boundaries |
| BT_SWAP | 5 | Swap boards between players |
| BT_LAWYERS | 6 | Drain opponent's funds |
| BT_RISE_UP | 7 | Fill opponent's bottom rows |
| BT_FLIP_OUT | 8 | Mirror opponent's board |
| BT_SPEEDY | 9 | Speed up opponent's drop rate |
| BT_MISSING | 10 | Deprive opponent of long pieces |
| BT_PIECE_IT | 11 | Give opponent disjointed pieces |
| BT_BLIND | 12 | Black screen (opponent can't see board) |
| BT_MONDALE | 13 | Named political weapon |
| BT_KEATING | 14 | Named political weapon |
| BT_CARTER | 15 | Tax opponent's funds |
| BT_REAGAN | 16 | Named political weapon |
| BT_AMES | 17 | Named political weapon |
| BT_ACE | 18 | Named weapon |
| BT_CONDOR | 19 | Mystery named weapon |
| BT_NICE_DAY | 20 | Give opponent happy-face pieces (ironic) |
| BT_SO_LONG | 21 | Named weapon |
| BT_NO_DICE | 22 | Remove die pieces from opponent's board |
| BT_BUG | 23 | Named weapon |
| BT_BOTTLE | 24 | Bottle neck piece restriction |
| BT_NO_SLIDE | 25 | Disable opponent's horizontal sliding |
| BT_SUSAN | 26 | Named weapon |
| BT_MEADOW | 27 | Named weapon |
| BT_MIRROR | 28 | Mirror opponent's controls |
| BT_TWILIGHT | 29 | Named weapon |
| BT_SLICK | 30 | Slick board (pieces slide) |
| BT_BROKEN | 31 | Break opponent's pieces apart |
| BT_FORCE | 32 | Force weapon |
| BT_GIMP | 33 | Gimp weapon |

## Internal APIs (Key C++ Classes)

### BTGame (game/BTGame.H)
Key public methods:
- `startGame()` — begin the game
- `endGame()` — end the game cleanly
- `pause(no_send)` / `unpause()` — pause/resume
- `keyPressed(char c)` — handle keyboard input
- `drop()` — hard drop current piece
- `moveLeft()` / `moveRight()` / `rotate()` — piece movement
- `lawyers(int nlines)` — apply Lawyers' Delite weapon effect
- `condor()` — condor weapon effect
- `receive(BTRingPacket*)` — process BTRingNode messages

### BTBoardManager (game/BTBoardManager.H)
Key public methods:
- `occupied(x, y)` — returns 1 if cell is occupied (inline, weapon-aware)
- `fill(x, y, BTBox*)` — place a box at position
- `checkLines()` — check for and clear complete lines; returns lines cleared
- `landed(x, y)` — notify that a piece landed at position
- `redraw()` — redraw board
- `clear()` — clear board
- `newBoard(BTBoard*)` — replace board state from network snapshot
- `receive(BTRingPacket*)` — process weapon events

### BTCommManager (game/BTCommManager.H)
Key public methods:
- `startGame(StreamSocket*, char* opponentName)` — begin networked game
- `startGame(BTCommManager* sibling)` — begin local (vs-AI) game
- `sendScore(BTScore*)` — transmit score update
- `sendWeapon(BTWeapon*)` — transmit weapon event
- `sendBoard(BTBoard*)` — transmit board snapshot
- `sendArsenal(BTArsenal*)` — transmit arsenal state
- `gameOver()` — signal game end
- `receive(BTRingPacket*)` — process internal messages for network relay

### BTNetManager (game/BTNetManager.H)
Key public methods:
- `challenge(BTNetworkEntry*)` — issue a challenge to a player
- `challengeComputer(int avail)` — start player-vs-AI game
- `recordStats(int won, BTGameStats*)` — record game result to server
- `gameOver()` — notify server of game end
- `netentry(int index)` — get network roster entry
- `netupdate()` — refresh network roster from server
- `plyentry(char* key)` — get player by key
- `plyupdate()` — refresh player database from server

### BTComputer (game/BTComputer.H)
Key public methods:
- `run()` — start the AI
- `reset(int)` — reset AI state
- `receive(BTRingPacket*)` — process game events
- `deciding()` — returns 1 if AI is currently computing its move
- `name()` — returns "Ernie" (AI name)
- `nLevels()` / `levelName(int)` — AI difficulty levels

## Data Models

### BTScore (game/BTScore.H)
```
score_     : unsigned long  — player's total score
op_score_  : unsigned long  — opponent's total score
lines_     : unsigned long  — player's lines cleared
op_lines_  : unsigned long  — opponent's lines cleared
funds_     : long           — player's available funds
op_funds_  : long           — opponent's funds
```

### BTWeapon (game/BTWeapon.H)
```
token_      : BTWeaponToken — weapon identifier
duration_   : unsigned short — duration in lines
price_      : unsigned short — cost in funds
name_       : char*         — weapon name
description_: char*         — weapon description
```

### BTBoard (game/BTBoard.H)
```
motivation_ : BTWeaponToken — reason for this board update
height_     : int           — board height
width_      : int           — board width
rep_        : Block<int>    — board cell data (flattened 2D array of box IDs)
```

### BTPlayer (db/BTPlayer.H)
- Player identity, ELO rating, win/loss stats, login, hostname

### BTNetworkEntry (db/BTNetworkEntry.H)
- Player network presence: login key, hostname, online status

### BTGameStats (db/BTGameStats.H)
- Per-game statistics sent to server: score, lines cleared, result
