use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::ai::{Ai, difficulty_think_ms};
use crate::engine::game_state::{GameEvent, GameMode, GamePhase, GameState, PlayerInput};
use crate::engine::weapons::{check_mirror, MirrorResult, WeaponKind};
use crate::protocol::GameMessage;

pub struct ErnieSession {
    state: GameState,
    ai: Ai,
    rng: StdRng,
    difficulty: u8,
    pending_susan_reply: bool,
    think_elapsed_ms: u32,
    think_interval_ms: u32,
}

impl ErnieSession {
    pub fn new(seed: u64, difficulty: u8) -> Self {
        let think_interval_ms = difficulty_think_ms(difficulty).max(1) as u32;
        let mut state = GameState::new(seed.wrapping_add(1));
        state.mode = GameMode::VsComputer;
        let ai = Ai::new(difficulty);
        state.tick(Some(PlayerInput::StartGame), 0);

        Self {
            state,
            ai,
            rng: StdRng::seed_from_u64(seed.wrapping_add(0xDEAD_BEEF)),
            difficulty,
            pending_susan_reply: false,
            think_elapsed_ms: 0,
            think_interval_ms,
        }
    }

    /// Advance Ernie by `elapsed_ms`, processing `incoming` player messages.
    /// Returns messages to send back to the player.
    pub fn step(&mut self, incoming: &[GameMessage], elapsed_ms: u32) -> Vec<GameMessage> {
        let mut out = Vec::new();

        for msg in incoming {
            if matches!(msg, GameMessage::NewGame) {
                let new_seed = self.rng.gen::<u64>();
                self.state = GameState::new(new_seed);
                self.state.mode = GameMode::VsComputer;
                self.ai = Ai::new(self.difficulty);
                self.pending_susan_reply = false;
                self.think_elapsed_ms = 0;
                self.state.tick(Some(PlayerInput::StartGame), 0);
            } else {
                let bazaar_triggered = self.process_player_message(msg, &mut out);
                self.ai.update_op_lines(self.state.score.op_lines);
                if bazaar_triggered && self.state.phase == GamePhase::Playing {
                    self.state.open_bazaar_now();
                    self.do_bazaar(&mut out);
                }
            }
        }

        self.think_elapsed_ms = self.think_elapsed_ms.saturating_add(elapsed_ms);
        if self.think_elapsed_ms >= self.think_interval_ms {
            self.think_elapsed_ms = 0;
            self.ernie_think(&mut out);
        }

        out
    }

    fn ernie_think(&mut self, out: &mut Vec<GameMessage>) {
        if matches!(self.state.phase, GamePhase::GameOver { .. }) {
            return;
        }

        if self.state.phase == GamePhase::InBazaar {
            self.do_bazaar(out);
            return;
        }

        if self.state.phase != GamePhase::Playing {
            return;
        }

        let Some(kind) = self.state.active_piece.as_ref().map(|p| p.kind) else { return };

        let maybe_move = self.ai.decide(&self.state.board, kind, &self.state.weapon_state);
        let (col, rotation) = if let Some(m) = maybe_move { (m.col, m.rotation) } else { (4, 0) };

        let events = self.state.ai_place_piece(col, rotation);

        out.push(GameMessage::BoardUpdate { snapshot: self.state.board.snapshot() });
        out.push(GameMessage::ScoreUpdate {
            score: self.state.score.score,
            lines: self.state.score.lines,
            funds: self.state.score.funds,
        });

        self.process_own_events(&events, out);

        if self.state.phase == GamePhase::InBazaar {
            self.do_bazaar(out);
        }
    }

    fn do_bazaar(&mut self, out: &mut Vec<GameMessage>) {
        self.state.open_bazaar_now();
        self.ai.go_shopping(&mut self.state.score, &mut self.state.arsenal, &mut self.rng, &self.state.board);
        out.push(GameMessage::BazaarOpen);
        self.state.tick(Some(PlayerInput::BazaarExit), 0);
        out.push(GameMessage::ScoreUpdate {
            score: self.state.score.score,
            lines: self.state.score.lines,
            funds: self.state.score.funds,
        });
    }

    fn process_own_events(&mut self, events: &[GameEvent], out: &mut Vec<GameMessage>) {
        for event in events {
            match event {
                GameEvent::GameOver { won: false } | GameEvent::RiseUpTopOut => {
                    out.push(GameMessage::GameOver {
                        winner_id: 1,
                        final_score_p1: 0,
                        final_score_p2: self.state.score.score,
                        winner_name: String::new(),
                        elo_delta_winner: 0,
                        elo_delta_loser: 0,
                    });
                }
                GameEvent::WeaponFired { kind: WeaponKind::Susan, reflect: false } => {
                    self.pending_susan_reply = true;
                    out.push(GameMessage::ArsenalSwapped { arsenal: self.state.arsenal.clone() });
                }
                GameEvent::WeaponFired { kind, reflect: false } => {
                    out.push(GameMessage::WeaponLaunched { kind: *kind });
                }
                GameEvent::WeaponFired { kind, reflect: true } => {
                    out.push(GameMessage::WeaponReflected { kind: *kind });
                }
                GameEvent::FundsStolen(amount) => {
                    out.push(GameMessage::FundsReceived { amount: *amount });
                }
                GameEvent::LinesCleared(n) => {
                    out.push(GameMessage::LinesCleared { count: *n as u32, funds_earned: 0 });
                }
                _ => {}
            }
        }
    }

    fn launch_queued_weapons(&mut self, out: &mut Vec<GameMessage>) {
        let kinds: Vec<WeaponKind> = self.ai.weapons_to_launch(&self.state.arsenal)
            .into_iter()
            .map(|(_, kind)| kind)
            .collect();

        for kind in kinds {
            if let Some(idx) = self.state.arsenal.slots.iter().position(|s| s.kind == kind) {
                self.state.arsenal.remove_slot(idx);
            }
            if kind == WeaponKind::Susan {
                self.pending_susan_reply = true;
                out.push(GameMessage::ArsenalSwapped { arsenal: self.state.arsenal.clone() });
            } else {
                out.push(GameMessage::WeaponLaunched { kind });
            }
        }
    }

    /// Returns true if the player's line clears triggered Ernie's bazaar threshold.
    fn process_player_message(&mut self, msg: &GameMessage, out: &mut Vec<GameMessage>) -> bool {
        match msg {
            GameMessage::WeaponLaunched { kind } => {
                let kind = *kind;
                let (state, rng) = (&mut self.state, &mut self.rng);
                let events = state.apply_incoming_weapon(kind, rng);
                out.push(GameMessage::BoardUpdate { snapshot: self.state.board.snapshot() });
                for ev in &events {
                    match ev {
                        GameEvent::WeaponFired { kind: k, reflect: true } => {
                            out.push(GameMessage::WeaponReflected { kind: *k });
                        }
                        GameEvent::FundsStolen(amount) => {
                            out.push(GameMessage::FundsReceived { amount: *amount });
                        }
                        _ => {}
                    }
                }
                if matches!(self.state.phase, GamePhase::GameOver { .. }) {
                    out.push(GameMessage::GameOver {
                        winner_id: 1,
                        final_score_p1: 0,
                        final_score_p2: self.state.score.score,
                        winner_name: String::new(),
                        elo_delta_winner: 0,
                        elo_delta_loser: 0,
                    });
                }
            }

            GameMessage::WeaponReflected { kind } => {
                let kind = *kind;
                let (state, rng) = (&mut self.state, &mut self.rng);
                state.apply_incoming_weapon(kind, rng);
                out.push(GameMessage::BoardUpdate { snapshot: self.state.board.snapshot() });
            }

            GameMessage::ArsenalSwapped { arsenal } => {
                if self.pending_susan_reply {
                    self.state.arsenal = arsenal.clone();
                    self.pending_susan_reply = false;
                } else {
                    let mirror = check_mirror(WeaponKind::Susan, &self.state.weapon_state);
                    if mirror == MirrorResult::PassThrough {
                        let my_old = std::mem::replace(&mut self.state.arsenal, arsenal.clone());
                        out.push(GameMessage::ArsenalSwapped { arsenal: my_old });
                    } else {
                        out.push(GameMessage::ArsenalSwapped { arsenal: arsenal.clone() });
                    }
                }
            }

            GameMessage::LinesCleared { count, .. } => {
                let count = *count;
                if self.state.weapon_state.is_active(WeaponKind::Lawyers) {
                    let mut topped_out = false;
                    for _ in 0..count {
                        let (state, rng) = (&mut self.state, &mut self.rng);
                        if state.board.rise_up(rng) {
                            state.phase = GamePhase::GameOver { won: false };
                            topped_out = true;
                            break;
                        }
                    }
                    if topped_out {
                        out.push(GameMessage::GameOver {
                            winner_id: 1,
                            final_score_p1: 0,
                            final_score_p2: self.state.score.score,
                            winner_name: String::new(),
                            elo_delta_winner: 0,
                            elo_delta_loser: 0,
                        });
                        return false;
                    }
                    out.push(GameMessage::BoardUpdate { snapshot: self.state.board.snapshot() });
                }
                if self.state.score.add_op_lines(count) {
                    return true;
                }
            }

            GameMessage::FundsReceived { amount } => {
                self.state.score.funds += *amount;
                out.push(GameMessage::ScoreUpdate {
                    score: self.state.score.score,
                    lines: self.state.score.lines,
                    funds: self.state.score.funds,
                });
            }

            GameMessage::ScoreUpdate { score, lines, funds } => {
                self.state.score.update_opponent(*score, *lines, *funds);
            }

            GameMessage::BazaarEnd => {
                self.launch_queued_weapons(out);
            }

            GameMessage::GameOver { .. } => {
                self.state.phase = GamePhase::GameOver { won: true };
            }

            _ => {}
        }
        false
    }
}
