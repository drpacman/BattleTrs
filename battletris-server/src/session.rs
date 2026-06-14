use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use battletris_engine::protocol::{self, GameMessage, ProtocolError};

use crate::db::PlayerDb;
use crate::elo;

const RECONNECT_TIMEOUT_SECS: u64 = 15;
const MAX_FRAME_BYTES: usize = 1_048_576; // 1 MB sanity limit

// ─── Frame helpers ───────────────────────────────────────────────────────────

async fn read_frame(stream: &mut TcpStream, buf: &mut Vec<u8>) -> std::io::Result<GameMessage> {
    loop {
        match protocol::decode(buf) {
            Ok(msg) => {
                let consumed = protocol::frame_len(buf);
                buf.drain(..consumed);
                return Ok(msg);
            }
            Err(ProtocolError::NeedMoreData) => {}
            Err(e) => {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()));
            }
        }
        let mut chunk = [0u8; 4096];
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "connection closed"));
        }
        if buf.len() + n > MAX_FRAME_BYTES {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"));
        }
        buf.extend_from_slice(&chunk[..n]);
    }
}

async fn write_frame(stream: &mut TcpStream, msg: &GameMessage) -> std::io::Result<()> {
    let bytes = protocol::encode(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    stream.write_all(&bytes).await
}

// ─── Session ─────────────────────────────────────────────────────────────────

pub async fn run_session(
    mut a: TcpStream,
    name_a: String,
    mut b: TcpStream,
    name_b: String,
    db: Arc<Mutex<PlayerDb>>,
) {
    eprintln!("[SERVER] session start: {name_a} vs {name_b}");

    // Pre-create player records so ELO is available immediately.
    {
        let mut db = db.lock().unwrap();
        db.get_or_create(&name_a);
        db.get_or_create(&name_b);
    }

    // Signal both clients to start.
    let _ = write_frame(&mut a, &GameMessage::GameStart).await;
    let _ = write_frame(&mut b, &GameMessage::GameStart).await;

    let mut buf_a: Vec<u8> = Vec::new();
    let mut buf_b: Vec<u8> = Vec::new();
    let mut combined_lines: u32 = 0;
    let mut next_bazaar_threshold: u32 = 20;

    loop {
        tokio::select! {
            result = read_frame(&mut a, &mut buf_a) => {
                match result {
                    Ok(msg) => {
                        if !handle_message(
                            msg, &name_a, &name_b,
                            &mut a, &mut b,
                            &mut combined_lines, &mut next_bazaar_threshold,
                            &db,
                        ).await { return; }
                    }
                    Err(_) => {
                        eprintln!("[SERVER] {name_a} disconnected");
                        if !handle_disconnect(&name_b, &mut b).await { return; }
                        return;
                    }
                }
            }
            result = read_frame(&mut b, &mut buf_b) => {
                match result {
                    Ok(msg) => {
                        if !handle_message(
                            msg, &name_b, &name_a,
                            &mut b, &mut a,
                            &mut combined_lines, &mut next_bazaar_threshold,
                            &db,
                        ).await { return; }
                    }
                    Err(_) => {
                        eprintln!("[SERVER] {name_b} disconnected");
                        if !handle_disconnect(&name_a, &mut a).await { return; }
                        return;
                    }
                }
            }
        }
    }
}

/// Handle one message from `sender_name` → relay to `peer`, intercept game-logic messages.
/// Returns false if the session should end.
async fn handle_message(
    msg: GameMessage,
    sender_name: &str,
    peer_name: &str,
    _sender: &mut TcpStream,
    peer: &mut TcpStream,
    combined_lines: &mut u32,
    next_threshold: &mut u32,
    db: &Arc<Mutex<PlayerDb>>,
) -> bool {
    match &msg {
        GameMessage::LinesCleared { count, .. } => {
            *combined_lines += count;
            while *combined_lines >= *next_threshold {
                *next_threshold += 20;
                eprintln!("[SERVER] combined_lines={combined_lines} — sending BazaarOpen");
                let _ = write_frame(_sender, &GameMessage::BazaarOpen).await;
                let _ = write_frame(peer, &GameMessage::BazaarOpen).await;
            }
            // Relay to peer regardless.
            let _ = write_frame(peer, &msg).await;
        }

        GameMessage::GameOver { .. } => {
            // sender's board topped out → sender is the LOSER, peer is the WINNER.
            eprintln!("[SERVER] {sender_name} lost — {peer_name} wins");
            let (winner_elo, loser_elo) = {
                let db = db.lock().unwrap();
                let we = db.get(peer_name).map(|r| r.elo).unwrap_or(1200);
                let le = db.get(sender_name).map(|r| r.elo).unwrap_or(1200);
                (we, le)
            };
            let deltas = elo::compute_elo_delta(winner_elo, loser_elo);
            {
                let mut db = db.lock().unwrap();
                db.apply_result(peer_name, sender_name, deltas);
            }
            let enriched = GameMessage::GameOver {
                winner_id: 1,
                final_score_p1: 0,
                final_score_p2: 0,
                winner_name: peer_name.to_string(),
                elo_delta_winner: deltas.0,
                elo_delta_loser: deltas.1,
            };
            let _ = write_frame(_sender, &enriched).await;
            let _ = write_frame(peer, &enriched).await;
            eprintln!("[SERVER] ELO updated: {peer_name} +{} / {sender_name} {}", deltas.0, deltas.1);
            return false;
        }

        // Transparent relay for all other messages.
        _ => {
            let _ = write_frame(peer, &msg).await;
        }
    }
    true
}

/// One client dropped. Notify peer; wait up to 15s. Session always ends (void — no ELO).
async fn handle_disconnect(peer_name: &str, peer: &mut TcpStream) -> bool {
    let _ = write_frame(peer, &GameMessage::PeerDisconnected).await;
    // Wait briefly to let the peer receive the message, then void.
    tokio::time::sleep(Duration::from_secs(RECONNECT_TIMEOUT_SECS)).await;
    let _ = write_frame(peer, &GameMessage::GameVoid).await;
    eprintln!("[SERVER] reconnect window expired for {peer_name} — game voided");
    false
}
