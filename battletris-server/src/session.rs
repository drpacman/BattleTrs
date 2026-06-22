use std::sync::{Arc, Mutex};
use std::time::Duration;

use battletris_engine::protocol::GameMessage;

use crate::conn::GameConn;
use crate::db::PlayerDb;
use crate::elo;

const RECONNECT_TIMEOUT_SECS: u64 = 15;

pub async fn run_session(
    mut a: Box<dyn GameConn>,
    name_a: String,
    mut b: Box<dyn GameConn>,
    name_b: String,
    db: Arc<Mutex<PlayerDb>>,
) {
    eprintln!("[SERVER] session start: {name_a} vs {name_b}");

    {
        let mut db = db.lock().unwrap();
        db.get_or_create(&name_a);
        db.get_or_create(&name_b);
    }

    let _ = a.write_frame(&GameMessage::GameStart).await;
    let _ = b.write_frame(&GameMessage::GameStart).await;

    let mut combined_lines: u32 = 0;
    let mut next_bazaar_threshold: u32 = 20;

    loop {
        tokio::select! {
            result = a.read_frame() => {
                match result {
                    Ok(msg) => {
                        if !handle_message(
                            msg, &name_a, &name_b,
                            &mut a, &mut b,
                            &mut combined_lines, &mut next_bazaar_threshold,
                            &db,
                        ).await { return; }
                    }
                    Err(e) => {
                        eprintln!("[SERVER] {name_a} disconnected: {e}");
                        handle_disconnect(&name_b, &mut b).await;
                        return;
                    }
                }
            }
            result = b.read_frame() => {
                match result {
                    Ok(msg) => {
                        if !handle_message(
                            msg, &name_b, &name_a,
                            &mut b, &mut a,
                            &mut combined_lines, &mut next_bazaar_threshold,
                            &db,
                        ).await { return; }
                    }
                    Err(e) => {
                        eprintln!("[SERVER] {name_b} disconnected: {e}");
                        handle_disconnect(&name_a, &mut a).await;
                        return;
                    }
                }
            }
        }
    }
}

async fn handle_message(
    msg: GameMessage,
    sender_name: &str,
    peer_name: &str,
    sender: &mut Box<dyn GameConn>,
    peer: &mut Box<dyn GameConn>,
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
                let _ = sender.write_frame(&GameMessage::BazaarOpen).await;
                let _ = peer.write_frame(&GameMessage::BazaarOpen).await;
            }
            let _ = peer.write_frame(&msg).await;
        }

        GameMessage::GameOver { .. } => {
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
            let _ = sender.write_frame(&enriched).await;
            let _ = peer.write_frame(&enriched).await;
            eprintln!(
                "[SERVER] ELO updated: {peer_name} +{} / {sender_name} {}",
                deltas.0, deltas.1
            );
            return false;
        }

        _ => {
            let _ = peer.write_frame(&msg).await;
        }
    }
    true
}

async fn handle_disconnect(peer_name: &str, peer: &mut Box<dyn GameConn>) {
    let _ = peer.write_frame(&GameMessage::PeerDisconnected).await;
    tokio::time::sleep(Duration::from_secs(RECONNECT_TIMEOUT_SECS)).await;
    let _ = peer.write_frame(&GameMessage::GameVoid).await;
    eprintln!("[SERVER] reconnect window expired for {peer_name} — game voided");
}
