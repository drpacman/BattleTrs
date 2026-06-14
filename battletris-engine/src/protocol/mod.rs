use serde::{Deserialize, Serialize};

use crate::engine::board::BoardSnapshot;
use crate::engine::weapons::WeaponKind;

/// Network + inter-thread message protocol.
///
/// Used both for the local vs-computer channel (Unit 2) and the TCP relay (Unit 3).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameMessage {
    // ── Session management ──────────────────────────────────────────────────
    /// Client → Server: identify on connect.
    Hello { name: String },
    /// Server → Client: name accepted.
    Welcome { assigned_name: String },
    /// Server → Both: both players connected; start the game.
    GameStart,
    /// Server → Both: server-arbitrated bazaar trigger (Q2=B).
    BazaarOpen,
    /// Server → Remaining: peer dropped; 15-second reconnect window active.
    PeerDisconnected,
    /// Server → Remaining: peer reconnected within window; resume.
    PeerReconnected,
    /// Server → Remaining: reconnect window expired; game voided, no ELO.
    GameVoid,
    /// Server → Client: name already in use; server closes connection.
    NameTaken,

    // ── Game flow ─────────────────────────────────────────────────────────────
    /// Signals Ernie to reset its game state for a new match.
    NewGame,

    // ── Game state sync ─────────────────────────────────────────────────────
    BoardUpdate { snapshot: BoardSnapshot },
    ScoreUpdate { score: u32, lines: u32, funds: i64 },
    LinesCleared { count: u32, funds_earned: i64 },

    // ── Weapons ─────────────────────────────────────────────────────────────
    /// Launcher fires a weapon at the recipient of this message.
    WeaponLaunched { kind: WeaponKind },
    /// Weapon was reflected by Mirror — apply it back to the original launcher.
    WeaponReflected { kind: WeaponKind },
    /// Keating: funds were stolen — add this amount to the recipient's funds.
    FundsReceived { amount: i64 },
    /// Arsenal swapped (Susan weapon).
    ArsenalSwapped,

    // ── Bazaar ──────────────────────────────────────────────────────────────
    BazaarEnd,

    // ── Game result ──────────────────────────────────────────────────────────
    /// Game over. In Ernie games winner_name is empty and ELO deltas are 0.
    GameOver {
        winner_id: u32,
        final_score_p1: u32,
        final_score_p2: u32,
        winner_name: String,
        elo_delta_winner: i32,
        elo_delta_loser: i32,
    },

    // ── Ping/pong ────────────────────────────────────────────────────────────
    Ping { seq: u32 },
    Pong { seq: u32 },
}

/// Error returned by encode/decode.
#[derive(Debug)]
pub enum ProtocolError {
    NeedMoreData,
    EncodeError(String),
    DecodeError(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::NeedMoreData => write!(f, "need more data"),
            ProtocolError::EncodeError(e) => write!(f, "encode error: {e}"),
            ProtocolError::DecodeError(e) => write!(f, "decode error: {e}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Encode a `GameMessage` as a 4-byte big-endian length prefix + bincode payload.
pub fn encode(msg: &GameMessage) -> Result<Vec<u8>, ProtocolError> {
    let payload = bincode::serialize(msg)
        .map_err(|e| ProtocolError::EncodeError(e.to_string()))?;
    let len = payload.len() as u32;
    let mut out = Vec::with_capacity(4 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Decode a `GameMessage` from a framed byte buffer.
/// Returns `Err(NeedMoreData)` if the buffer does not yet contain a full frame.
/// On success the caller should advance past `4 + frame_len` bytes.
pub fn decode(buf: &[u8]) -> Result<GameMessage, ProtocolError> {
    if buf.len() < 4 {
        return Err(ProtocolError::NeedMoreData);
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if buf.len() < 4 + len {
        return Err(ProtocolError::NeedMoreData);
    }
    bincode::deserialize(&buf[4..4 + len])
        .map_err(|e| ProtocolError::DecodeError(e.to_string()))
}

/// Returns the total framed byte count for a successfully decoded message.
/// Use after a successful `decode` to know how many bytes to consume.
pub fn frame_len(buf: &[u8]) -> usize {
    let payload_len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    4 + payload_len
}

/// Player ELO / stats record, persisted in the server's PlayerDb.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayerRecord {
    pub name: String,
    pub elo: i32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

impl PlayerRecord {
    pub fn new(name: &str) -> Self {
        PlayerRecord { name: name.to_string(), elo: 1200, ..Default::default() }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::{Board, BoardSnapshot};
    use crate::engine::weapons::WeaponKind;

    fn round_trip(msg: GameMessage) -> GameMessage {
        let encoded = encode(&msg).expect("encode failed");
        let decoded = decode(&encoded).expect("decode failed");
        decoded
    }

    #[test]
    fn encode_decode_board_update() {
        let board = Board::new();
        let msg = GameMessage::BoardUpdate { snapshot: board.snapshot() };
        match round_trip(msg) {
            GameMessage::BoardUpdate { .. } => {}
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn encode_decode_weapon_launched() {
        let msg = GameMessage::WeaponLaunched { kind: WeaponKind::Gimp };
        match round_trip(msg) {
            GameMessage::WeaponLaunched { kind: WeaponKind::Gimp } => {}
            other => panic!("wrong: {other:?}"),
        }
    }

    #[test]
    fn encode_decode_game_over_with_elo() {
        let msg = GameMessage::GameOver {
            winner_id: 1,
            final_score_p1: 1000,
            final_score_p2: 800,
            winner_name: "Alice".to_string(),
            elo_delta_winner: 16,
            elo_delta_loser: -16,
        };
        match round_trip(msg) {
            GameMessage::GameOver { winner_name, elo_delta_winner, .. } => {
                assert_eq!(winner_name, "Alice");
                assert_eq!(elo_delta_winner, 16);
            }
            other => panic!("wrong: {other:?}"),
        }
    }

    #[test]
    fn encode_decode_bazaar_open() {
        match round_trip(GameMessage::BazaarOpen) {
            GameMessage::BazaarOpen => {}
            other => panic!("wrong: {other:?}"),
        }
    }

    #[test]
    fn decode_truncated_returns_need_more_data() {
        let encoded = encode(&GameMessage::BazaarOpen).unwrap();
        // Feed only 3 bytes (less than the 4-byte header)
        assert!(matches!(decode(&encoded[..3]), Err(ProtocolError::NeedMoreData)));
        // Feed header but not full payload
        assert!(matches!(decode(&encoded[..encoded.len() - 1]), Err(ProtocolError::NeedMoreData)));
    }
}
