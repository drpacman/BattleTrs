use std::sync::mpsc::{Receiver, SyncSender};
use std::time::{Duration, Instant};

use rand::SeedableRng;
use rand::rngs::StdRng;

use battletris_engine::ai::Ai;
use battletris_engine::engine::game_state::{GamePhase, GameState, PlayerInput};
use battletris_engine::engine::weapons::WeaponKind;
use battletris_engine::protocol::GameMessage;

const THINK_INTERVAL_MS: u64 = 750;

pub fn run_ernie(
    from_player: Receiver<GameMessage>,
    to_player: SyncSender<GameMessage>,
    seed: u64,
) {
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(0xDEAD_BEEF));
    let mut state = GameState::new(seed.wrapping_add(1));
    let mut ai = Ai::new(1);

    state.tick(Some(PlayerInput::StartGame), 0);
    eprintln!("[ERNIE] started, seed={seed:#x}");

    let mut last_think = Instant::now();

    loop {
        while let Ok(msg) = from_player.try_recv() {
            if matches!(msg, GameMessage::NewGame) {
                state = GameState::new(rand::random::<u64>());
                ai = Ai::new(1);
                state.tick(Some(PlayerInput::StartGame), 0);
                last_think = Instant::now();
                eprintln!("[ERNIE] reset for new game");
            } else {
                let bazaar_triggered = process_player_message(&mut state, &mut rng, &to_player, msg);
                if bazaar_triggered && state.phase == GamePhase::Playing {
                    eprintln!("[ERNIE] bazaar triggered by player line clears (combined={})", state.score.combined_lines);
                    do_bazaar(&mut state, &ai, &mut rng, &to_player, "op_lines");
                }
            }
        }

        if last_think.elapsed() >= Duration::from_millis(THINK_INTERVAL_MS) {
            last_think = Instant::now();
            ernie_think(&mut state, &ai, &mut rng, &to_player);
        }

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn is_game_over(state: &GameState) -> bool {
    matches!(state.phase, GamePhase::GameOver { .. })
}

fn ernie_think(
    state: &mut GameState,
    ai: &Ai,
    rng: &mut StdRng,
    to_player: &SyncSender<GameMessage>,
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

    process_ernie_events(state, &events, rng, to_player);

    if state.phase == GamePhase::InBazaar {
        eprintln!("[ERNIE] bazaar triggered inline after piece lock (combined={})", state.score.combined_lines);
        do_bazaar(state, ai, rng, to_player, "piece_lock");
    }
}

/// Shared bazaar handling: shop, fire all purchased weapons, exit bazaar.
fn do_bazaar(
    state: &mut GameState,
    ai: &Ai,
    rng: &mut StdRng,
    to_player: &SyncSender<GameMessage>,
    trigger: &str,
) {
    // Open Ernie's own bazaar (no-op if already InBazaar from own line clears).
    state.open_bazaar_now();
    // Tell the player to open their bazaar — Ernie is the authority, like a server.
    let _ = to_player.try_send(GameMessage::BazaarOpen);

    let funds_before = state.score.funds;
    let bought = ai.go_shopping(&mut state.score, &mut state.arsenal, rng);
    let spent = funds_before - state.score.funds;

    if bought.is_empty() {
        eprintln!("[ERNIE] bazaar ({trigger}): funds=${funds_before} — nothing affordable, skipped shopping");
    } else {
        eprintln!("[ERNIE] bazaar ({trigger}): funds=${funds_before} → ${} (spent ${spent})", state.score.funds);
        for kind in &bought {
            eprintln!("[ERNIE]   bought {kind:?} (${}/ea)", battletris_engine::engine::weapons::weapon_def(*kind).price);
        }
    }

    launch_queued_weapons(state, ai, to_player);
    state.tick(Some(PlayerInput::BazaarExit), 0);
    send_score(state, to_player);
}

fn process_ernie_events(
    state: &mut GameState,
    events: &[battletris_engine::engine::game_state::GameEvent],
    _rng: &mut StdRng,
    to_player: &SyncSender<GameMessage>,
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
            GameEvent::WeaponFired { kind, reflect: false } => {
                let _ = to_player.try_send(GameMessage::WeaponLaunched { kind: *kind });
            }
            GameEvent::WeaponFired { kind, reflect: true } => {
                let _ = to_player.try_send(GameMessage::WeaponReflected { kind: *kind });
            }
            GameEvent::FundsStolen(amount) => {
                let _ = to_player.try_send(GameMessage::FundsReceived { amount: *amount });
            }
            _ => {}
        }
    }
}

fn launch_queued_weapons(state: &mut GameState, ai: &Ai, to_player: &SyncSender<GameMessage>) {
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
        let _ = to_player.try_send(GameMessage::WeaponLaunched { kind });
    }
}

/// Returns true if player line clears triggered Ernie's bazaar threshold.
fn process_player_message(
    state: &mut GameState,
    rng: &mut StdRng,
    to_player: &SyncSender<GameMessage>,
    msg: GameMessage,
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

        GameMessage::LinesCleared { count, .. } => {
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

        GameMessage::ArsenalSwapped => {}

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
