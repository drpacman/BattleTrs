use std::sync::mpsc::{Receiver, SyncSender};
use std::time::{Duration, Instant};

use rand::SeedableRng;
use rand::rngs::StdRng;

use battletris_engine::ai::{Ai, difficulty_think_ms};
use battletris_engine::engine::game_state::{GameMode, GamePhase, GameState, PlayerInput};
use battletris_engine::engine::weapons::{check_mirror, MirrorResult, WeaponKind};
use battletris_engine::protocol::GameMessage;

pub fn run_ernie(
    from_player: Receiver<GameMessage>,
    to_player: SyncSender<GameMessage>,
    seed: u64,
    difficulty: u8,
) {
    let think_interval_ms = difficulty_think_ms(difficulty).max(1);
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(0xDEAD_BEEF));
    let mut state = GameState::new(seed.wrapping_add(1));
    state.mode = GameMode::VsComputer;
    let mut ai = Ai::new(difficulty);
    // True after Ernie fires Susan — the next ArsenalSwapped is the player's reply.
    let mut pending_susan_reply = false;

    state.tick(Some(PlayerInput::StartGame), 0);
    eprintln!("[ERNIE] started, seed={seed:#x}, difficulty={difficulty} ({}ms)", think_interval_ms);

    let mut last_think = Instant::now();

    loop {
        while let Ok(msg) = from_player.try_recv() {
            if matches!(msg, GameMessage::NewGame) {
                state = GameState::new(rand::random::<u64>());
                state.mode = GameMode::VsComputer;
                ai = Ai::new(difficulty);
                pending_susan_reply = false;
                state.tick(Some(PlayerInput::StartGame), 0);
                last_think = Instant::now();
                eprintln!("[ERNIE] reset for new game");
            } else {
                let bazaar_triggered = process_player_message(
                    &mut state, &ai, &mut rng, &to_player, msg, &mut pending_susan_reply,
                );
                ai.update_op_lines(state.score.op_lines);
                if bazaar_triggered && state.phase == GamePhase::Playing {
                    eprintln!("[ERNIE] bazaar triggered by player line clears (combined={})", state.score.combined_lines);
                    do_bazaar(&mut state, &mut ai, &mut rng, &to_player, "op_lines");
                }
            }
        }

        if last_think.elapsed() >= Duration::from_millis(think_interval_ms) {
            last_think = Instant::now();
            ernie_think(&mut state, &mut ai, &mut rng, &to_player, &mut pending_susan_reply);
        }

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn is_game_over(state: &GameState) -> bool {
    matches!(state.phase, GamePhase::GameOver { .. })
}

fn ernie_think(
    state: &mut GameState,
    ai: &mut Ai,
    rng: &mut StdRng,
    to_player: &SyncSender<GameMessage>,
    pending_susan_reply: &mut bool,
) {
    if is_game_over(state) {
        return;
    }

    if state.phase == GamePhase::InBazaar {
        eprintln!("[ERNIE] bazaar triggered by own line clears (combined={})", state.score.combined_lines);
        do_bazaar(state, ai, rng, to_player, "own_lines");
        return;
    }

    if state.phase != GamePhase::Playing {
        return;
    }

    let Some(kind) = state.active_piece.as_ref().map(|p| p.kind) else { return };

    let maybe_move = ai.decide(&state.board, kind, &state.weapon_state);
    let (col, rotation) = if let Some(m) = maybe_move {
        (m.col, m.rotation)
    } else {
        (4, 0)
    };

    let events = state.ai_place_piece(col, rotation);

    let _ = to_player.try_send(GameMessage::BoardUpdate {
        snapshot: state.board.snapshot(),
    });
    send_score(state, to_player);

    process_ernie_events(state, &events, rng, to_player, pending_susan_reply);

    if state.phase == GamePhase::InBazaar {
        eprintln!("[ERNIE] bazaar triggered inline after piece lock (combined={})", state.score.combined_lines);
        do_bazaar(state, ai, rng, to_player, "piece_lock");
    }
}

/// Shared bazaar handling: shop, fire all purchased weapons, exit bazaar.
fn do_bazaar(
    state: &mut GameState,
    ai: &mut Ai,
    rng: &mut StdRng,
    to_player: &SyncSender<GameMessage>,
    trigger: &str,
) {
    // Open Ernie's own bazaar (no-op if already InBazaar from own line clears).
    state.open_bazaar_now();

    let funds_before = state.score.funds;
    let bought = ai.go_shopping(&mut state.score, &mut state.arsenal, rng, &state.board);
    let spent = funds_before - state.score.funds;

    if bought.is_empty() {
        eprintln!("[ERNIE] bazaar ({trigger}): funds=${funds_before} — nothing affordable, skipped shopping");
    } else {
        eprintln!("[ERNIE] bazaar ({trigger}): funds=${funds_before} → ${} (spent ${spent})", state.score.funds);
        for kind in &bought {
            eprintln!("[ERNIE]   bought {kind:?} (${}/ea)", battletris_engine::engine::weapons::weapon_def(*kind).price);
        }
    }

    // Tell the player to open their bazaar. Weapons are fired later, after the
    // player sends BazaarEnd, so they land during normal play not during shopping.
    let _ = to_player.try_send(GameMessage::BazaarOpen);

    state.tick(Some(PlayerInput::BazaarExit), 0);
    send_score(state, to_player);
}

fn process_ernie_events(
    state: &mut GameState,
    events: &[battletris_engine::engine::game_state::GameEvent],
    _rng: &mut StdRng,
    to_player: &SyncSender<GameMessage>,
    pending_susan_reply: &mut bool,
) {
    use battletris_engine::engine::game_state::GameEvent;

    for event in events {
        match event {
            GameEvent::GameOver { won: false } | GameEvent::RiseUpTopOut => {
                let _ = to_player.try_send(GameMessage::GameOver {
                    winner_id: 1,
                    final_score_p1: 0,
                    final_score_p2: state.score.score,
                    winner_name: String::new(),
                    elo_delta_winner: 0,
                    elo_delta_loser: 0,
                });
            }
            GameEvent::WeaponFired { kind: WeaponKind::Susan, reflect: false } => {
                // Susan: start the arsenal-swap handshake instead of a bare WeaponLaunched.
                *pending_susan_reply = true;
                let _ = to_player.try_send(GameMessage::ArsenalSwapped {
                    arsenal: state.arsenal.clone(),
                });
            }
            GameEvent::WeaponFired { kind, reflect: false } => {
                let _ = to_player.try_send(GameMessage::WeaponLaunched { kind: *kind });
            }
            GameEvent::WeaponFired { kind, reflect: true } => {
                let _ = to_player.try_send(GameMessage::WeaponReflected { kind: *kind });
            }
            GameEvent::FundsStolen(amount) => {
                let _ = to_player.try_send(GameMessage::FundsReceived { amount: *amount });
            }
            GameEvent::LinesCleared(n) => {
                // Tell the player about Ernie's line clears so their combined-line
                // counter (and therefore bazaar trigger) tracks both players.
                let _ = to_player.try_send(GameMessage::LinesCleared { count: *n as u32, funds_earned: 0 });
            }
            _ => {}
        }
    }
}

fn launch_queued_weapons(
    state: &mut GameState,
    ai: &Ai,
    to_player: &SyncSender<GameMessage>,
    pending_susan_reply: &mut bool,
) {
    let kinds: Vec<WeaponKind> = ai.weapons_to_launch(&state.arsenal)
        .into_iter()
        .map(|(_, kind)| kind)
        .collect();

    if kinds.is_empty() {
        eprintln!("[ERNIE] no weapons to fire (arsenal: {} slots)", state.arsenal.slots.len());
    }

    for kind in kinds {
        if let Some(slot_idx) = state.arsenal.slots.iter().position(|s| s.kind == kind) {
            state.arsenal.remove_slot(slot_idx);
        }
        eprintln!("[ERNIE] fired {kind:?} at player");
        if kind == WeaponKind::Susan {
            *pending_susan_reply = true;
            let _ = to_player.try_send(GameMessage::ArsenalSwapped {
                arsenal: state.arsenal.clone(),
            });
        } else {
            let _ = to_player.try_send(GameMessage::WeaponLaunched { kind });
        }
    }
}

/// Returns true if player line clears triggered Ernie's bazaar threshold.
fn process_player_message(
    state: &mut GameState,
    ai: &Ai,
    rng: &mut StdRng,
    to_player: &SyncSender<GameMessage>,
    msg: GameMessage,
    pending_susan_reply: &mut bool,
) -> bool {
    match msg {
        GameMessage::WeaponLaunched { kind } => {
            eprintln!("[ERNIE] received weapon {kind:?} from player — applying");
            let events = state.apply_incoming_weapon(kind, rng);
            let _ = to_player.try_send(GameMessage::BoardUpdate {
                snapshot: state.board.snapshot(),
            });
            use battletris_engine::engine::game_state::GameEvent;
            for ev in &events {
                if let GameEvent::WeaponFired { kind: k, reflect: true } = ev {
                    eprintln!("[ERNIE] weapon {k:?} reflected back at player");
                    let _ = to_player.try_send(GameMessage::WeaponReflected { kind: *k });
                }
                if let GameEvent::FundsStolen(amount) = ev {
                    let _ = to_player.try_send(GameMessage::FundsReceived { amount: *amount });
                }
            }
            if is_game_over(state) {
                let _ = to_player.try_send(GameMessage::GameOver {
                    winner_id: 1,
                    final_score_p1: 0,
                    final_score_p2: state.score.score,
                    winner_name: String::new(),
                    elo_delta_winner: 0,
                    elo_delta_loser: 0,
                });
            }
        }

        GameMessage::WeaponReflected { kind } => {
            eprintln!("[ERNIE] own weapon {kind:?} reflected back — applying to self");
            let _events = state.apply_incoming_weapon(kind, rng);
            let _ = to_player.try_send(GameMessage::BoardUpdate {
                snapshot: state.board.snapshot(),
            });
        }

        GameMessage::ArsenalSwapped { arsenal } => {
            if *pending_susan_reply {
                // Reply to Ernie's own Susan — take the player's old arsenal.
                eprintln!("[ERNIE] Susan reply received — taking player's arsenal ({} slots)", arsenal.slots.len());
                state.arsenal = arsenal;
                *pending_susan_reply = false;
            } else {
                // Player fired Susan at Ernie.
                let mirror = check_mirror(WeaponKind::Susan, &state.weapon_state);
                if mirror == MirrorResult::PassThrough {
                    eprintln!("[ERNIE] Susan received from player — swapping arsenals");
                    let my_old = std::mem::replace(&mut state.arsenal, arsenal);
                    let _ = to_player.try_send(GameMessage::ArsenalSwapped { arsenal: my_old });
                } else {
                    eprintln!("[ERNIE] Susan nullified by Mirror — echoing back");
                    let _ = to_player.try_send(GameMessage::ArsenalSwapped { arsenal });
                }
            }
        }

        GameMessage::LinesCleared { count, .. } => {
            // If Ernie has Lawyers active, each player line clear pushes Ernie's board up.
            if state.weapon_state.is_active(WeaponKind::Lawyers) {
                for _ in 0..count {
                    if state.board.rise_up(rng) {
                        eprintln!("[ERNIE] topped out from Lawyers rise_up");
                        state.phase = GamePhase::GameOver { won: false };
                        let _ = to_player.try_send(GameMessage::GameOver {
                            winner_id: 1,
                            final_score_p1: 0,
                            final_score_p2: state.score.score,
                            winner_name: String::new(),
                            elo_delta_winner: 0,
                            elo_delta_loser: 0,
                        });
                        return false;
                    }
                }
                let _ = to_player.try_send(GameMessage::BoardUpdate {
                    snapshot: state.board.snapshot(),
                });
            }
            if state.score.add_op_lines(count) {
                eprintln!("[ERNIE] player's {count} line(s) pushed combined to {} — bazaar threshold", state.score.combined_lines);
                return true;
            }
        }

        GameMessage::FundsReceived { amount } => {
            state.score.funds += amount;
            send_score(state, to_player);
        }

        GameMessage::ScoreUpdate { score, lines, funds } => {
            state.score.update_opponent(score, lines, funds);
        }

        GameMessage::BazaarEnd => {
            // Player has finished shopping and returned to play — fire queued weapons now.
            launch_queued_weapons(state, ai, to_player, pending_susan_reply);
        }

        GameMessage::GameOver { .. } => {
            // Player reported their own loss — stop Ernie from continuing to play.
            state.phase = GamePhase::GameOver { won: true };
        }

        _ => {}
    }
    false
}

fn send_score(state: &GameState, to_player: &SyncSender<GameMessage>) {
    let _ = to_player.try_send(GameMessage::ScoreUpdate {
        score: state.score.score,
        lines: state.score.lines,
        funds: state.score.funds,
    });
}
