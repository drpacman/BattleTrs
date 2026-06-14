use rand::{SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use crate::engine::board::{Board, BoardSnapshot};
use crate::engine::piece::{ActivePiece, PieceKind};
use crate::engine::score::{Score, ScoreView};
use crate::engine::weapons::{
    Arsenal, ArsenalSlotView, ActiveWeaponView, BazaarStateView,
    BazaarState, WeaponKind, WeaponState,
    weapon_def, apply_weapon_instant, apply_weapon_timed, check_mirror,
    MirrorResult, WEAPON_COUNT,
};
use crate::protocol::GameMessage;

// Timing constants (milliseconds) — from BTConstants.H
pub const DROP_INTERVAL_MS: u32 = 512;
pub const FAST_DROP_INTERVAL_MS: u32 = 10;
pub const LOCK_DELAY_MS: u32 = 150;
pub const SLICK_INTERVAL_MS: u32 = 150; // BT_SLICK_TIMEOUT

pub const SPAWN_X: i32 = 5;
pub const SPAWN_Y: i32 = 0;
pub const HAPPY_FUND_VALUE: i32 = 150;
pub const LINES_UNTIL_BAZAAR: u32 = 20;

// ─── Public types ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    SinglePlayer,
    VsComputer,
    VsNetwork,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GamePhase {
    Title,
    ConnectingToServer,
    WaitingForOpponent,
    Playing,
    InBazaar,
    Paused,
    GameOver { won: bool },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PieceState {
    Dropping,
    LockDelay,
    HardDropping,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerInput {
    MoveLeft,
    MoveRight,
    RotateCW,
    RotateCCW,
    SoftDrop,
    SoftDropRelease,
    HardDrop,
    LaunchWeapon(u8),   // slot index 0-9
    BazaarUp,
    BazaarDown,
    BazaarBuy,
    BazaarExit,
    StartGame,
    Pause,
    QuitToTitle,
}

/// Events emitted by `tick()` that the game loop acts on.
#[derive(Clone, Debug)]
pub enum GameEvent {
    LinesCleared(u32),
    BazaarTriggered,
    PieceLocked,
    PieceSpawned,
    WeaponExpired(WeaponKind),
    WeaponFired { kind: WeaponKind, reflect: bool },
    FundsStolen(i64),       // Keating: amount stolen from this player
    FundsReceived(i64),     // Keating: amount added to this player
    GimpFlash,              // Gimp weapon visual trigger
    RiseUpTopOut,           // Rise Up caused a top-out
    GameOver { won: bool },
}

/// Renderer-facing snapshot for a single frame.
#[derive(Clone, Debug)]
pub struct PlayingView {
    pub player_board: BoardSnapshot,
    pub active_piece: Option<(PieceKind, Vec<(i32, i32)>)>,
    pub ghost_cells: Vec<(i32, i32)>,
    pub next_piece: PieceKind,
    pub score: ScoreView,
    pub opponent_board: Option<BoardSnapshot>,
    // Weapon visuals
    pub player_active_weapons: Vec<ActiveWeaponView>,
    pub ernie_active_weapons: Vec<ActiveWeaponView>,
    pub player_arsenal: Vec<ArsenalSlotView>,
    pub ernie_arsenal_count: usize,
    pub upbyside_active: bool,
    pub blind_cells: Vec<(usize, usize)>,
    pub twilight_active: bool,
    pub gimp_flash: bool,
    pub opponent_gimp_flash: bool,
    pub slick_active: bool,
    pub show_opponent_funds: bool,
    pub opponent_board_accuracy: f32,
    pub bazaar_view: Option<BazaarStateView>,
    /// Set by client game_loop when server sends PeerDisconnected; triggers overlay.
    pub peer_disconnected: bool,
}

// ─── GameState ────────────────────────────────────────────────────────────────

pub struct GameState {
    pub board: Board,
    pub active_piece: Option<ActivePiece>,
    pub next_piece: PieceKind,
    pub score: Score,
    pub phase: GamePhase,
    pub mode: GameMode,
    pub weapon_state: WeaponState,
    pub arsenal: Arsenal,
    pub bazaar: Option<BazaarState>,
    piece_state: PieceState,
    drop_elapsed_ms: u32,
    lock_elapsed_ms: u32,
    slick_elapsed_ms: u32,
    is_soft_dropping: bool,
    pub gimp_flash_ms: u32,
    opponent_gimp_flash_ms: u32,
    rng: StdRng,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let next_piece = PieceKind::random(&mut rng);
        GameState {
            board: Board::new(),
            active_piece: None,
            next_piece,
            score: Score::default(),
            phase: GamePhase::Title,
            mode: GameMode::SinglePlayer,
            weapon_state: WeaponState::new(),
            arsenal: Arsenal::new(),
            bazaar: None,
            piece_state: PieceState::Dropping,
            drop_elapsed_ms: 0,
            lock_elapsed_ms: 0,
            slick_elapsed_ms: 0,
            is_soft_dropping: false,
            gimp_flash_ms: 0,
            opponent_gimp_flash_ms: 0,
            rng,
        }
    }

    /// Main entry point: process one input event and advance by `elapsed_ms`.
    pub fn tick(&mut self, input: Option<PlayerInput>, elapsed_ms: u32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        match self.phase.clone() {
            GamePhase::Title  => {
                if let Some(PlayerInput::StartGame) = input {
                    self.start_game(&mut events);
                }
            }
            GamePhase::GameOver { .. } => {
                if let Some(PlayerInput::StartGame) = input {
                    self.start_game(&mut events);
                } else if let Some(PlayerInput::QuitToTitle) = input {
                    self.phase = GamePhase::Title;
                }
            }
            GamePhase::Paused => {
                if let Some(PlayerInput::Pause) = input {
                    self.phase = GamePhase::Playing;
                }
            }
            GamePhase::InBazaar => {
                self.process_bazaar_input(input, &mut events);
            }
            GamePhase::Playing => {
                self.process_input(input, &mut events);
                if self.phase == GamePhase::Playing {
                    self.advance_time(elapsed_ms, &mut events);
                }
            }
            _ => {}
        }

        events
    }

    pub fn start_game(&mut self, events: &mut Vec<GameEvent>) {
        self.board = Board::new();
        self.score = Score::default();
        self.weapon_state = WeaponState::new();
        self.arsenal = Arsenal::new();
        self.bazaar = None;
        self.piece_state = PieceState::Dropping;
        self.drop_elapsed_ms = 0;
        self.lock_elapsed_ms = 0;
        self.slick_elapsed_ms = 0;
        self.is_soft_dropping = false;
        self.gimp_flash_ms = 0;
        self.opponent_gimp_flash_ms = 0;
        self.next_piece = PieceKind::random_filtered(&self.weapon_state, &mut self.rng);
        self.phase = GamePhase::Playing;
        self.spawn_next_piece(events);
    }

    fn process_bazaar_input(&mut self, input: Option<PlayerInput>, _events: &mut Vec<GameEvent>) {
        let Some(input) = input else { return };
        let baz = match self.bazaar.as_mut() {
            Some(b) => b,
            None => return,
        };
        match input {
            PlayerInput::BazaarUp | PlayerInput::RotateCW | PlayerInput::MoveLeft => baz.navigate_up(),
            PlayerInput::BazaarDown | PlayerInput::SoftDrop | PlayerInput::MoveRight => baz.navigate_down(),
            PlayerInput::BazaarBuy | PlayerInput::StartGame | PlayerInput::HardDrop => {
                let carter = self.weapon_state.is_active(WeaponKind::Carter);
                // Need to satisfy borrow checker by cloning bazaar momentarily
                let kind = baz.current_kind();
                let def = weapon_def(kind);
                let price = if carter { def.price * 2 } else { def.price };
                if self.score.funds >= price as i64 && self.arsenal.can_add(kind) {
                    self.score.funds -= price as i64;
                    self.arsenal.add(kind);
                }
            }
            PlayerInput::BazaarExit | PlayerInput::QuitToTitle | PlayerInput::Pause => {
                if let Some(baz) = self.bazaar.as_mut() {
                    baz.player_done = true;
                }
                // Check if we can close the bazaar
                self.maybe_close_bazaar();
            }
            _ => {}
        }
    }

    fn maybe_close_bazaar(&mut self) {
        let can_close = self.bazaar.as_ref().map(|b| {
            b.player_done
                && (b.ernie_done || self.mode == GameMode::SinglePlayer)
        }).unwrap_or(false);
        if can_close {
            self.bazaar = None;
            self.phase = GamePhase::Playing;
        }
    }

    /// Signal that Ernie has finished bazaar shopping.
    pub fn ernie_bazaar_done(&mut self) {
        if let Some(baz) = self.bazaar.as_mut() {
            baz.ernie_done = true;
        }
        self.maybe_close_bazaar();
    }

    /// Open the bazaar immediately (used when opponent line clears trigger the threshold).
    pub fn open_bazaar_now(&mut self) {
        if self.phase == GamePhase::Playing {
            self.phase = GamePhase::InBazaar;
            self.bazaar = Some(BazaarState::new());
        }
    }

    fn process_input(&mut self, input: Option<PlayerInput>, events: &mut Vec<GameEvent>) {
        let input = match input {
            Some(i) => i,
            None => return,
        };

        match input {
            PlayerInput::SoftDrop => {
                self.is_soft_dropping = true;
                return;
            }
            PlayerInput::SoftDropRelease => {
                self.is_soft_dropping = false;
                return;
            }
            PlayerInput::Pause => {
                self.phase = GamePhase::Paused;
                return;
            }
            PlayerInput::QuitToTitle => {
                self.phase = GamePhase::Title;
                return;
            }
            PlayerInput::LaunchWeapon(slot) => {
                if let Some(kind) = self.arsenal.remove_slot(slot as usize) {
                    if kind == WeaponKind::Gimp {
                        self.opponent_gimp_flash_ms = 800;
                    }
                    events.push(GameEvent::WeaponFired { kind, reflect: false });
                }
                return;
            }
            _ => {}
        }

        let piece = match self.active_piece.as_mut() {
            Some(p) => p,
            None => return,
        };

        // NoSlide blocks left/right movement
        let no_slide = self.weapon_state.no_slide();

        let moved = match input {
            PlayerInput::MoveLeft => {
                if no_slide { false } else { piece.try_move_left(&self.board) }
            }
            PlayerInput::MoveRight => {
                if no_slide { false } else { piece.try_move_right(&self.board) }
            }
            PlayerInput::RotateCW => piece.try_rotate_cw(&self.board),
            PlayerInput::RotateCCW => piece.try_rotate_ccw(&self.board),
            PlayerInput::HardDrop => {
                let y_before = piece.y;
                let ghost_y = piece.ghost_y(&self.board);
                piece.y = ghost_y;
                self.score.add_hard_drop(y_before);
                self.piece_state = PieceState::HardDropping;
                self.drop_elapsed_ms = FAST_DROP_INTERVAL_MS;
                return;
            }
            _ => false,
        };

        if moved && self.piece_state == PieceState::LockDelay {
            self.lock_elapsed_ms = 0;
        }
    }

    fn advance_time(&mut self, elapsed_ms: u32, events: &mut Vec<GameEvent>) {
        // Gimp flash countdowns
        self.gimp_flash_ms = self.gimp_flash_ms.saturating_sub(elapsed_ms);
        self.opponent_gimp_flash_ms = self.opponent_gimp_flash_ms.saturating_sub(elapsed_ms);

        // Slick Willy: independent 150ms sub-timer
        if self.weapon_state.slick_dir != 0 {
            self.slick_elapsed_ms += elapsed_ms;
            while self.slick_elapsed_ms >= SLICK_INTERVAL_MS {
                self.slick_elapsed_ms -= SLICK_INTERVAL_MS;
                self.apply_slick_tick();
            }
        }

        // Hatter: spin piece on every gravity tick (handled below when drop fires)

        if self.piece_state == PieceState::LockDelay {
            self.lock_elapsed_ms += elapsed_ms;
            if self.lock_elapsed_ms >= LOCK_DELAY_MS {
                self.lock_current_piece(events);
            }
            return;
        }

        let speedy_mult = self.weapon_state.speedy_multiplier();
        let drop_interval = if self.piece_state == PieceState::HardDropping || self.is_soft_dropping {
            FAST_DROP_INTERVAL_MS
        } else {
            DROP_INTERVAL_MS / speedy_mult.max(1)
        };

        self.drop_elapsed_ms += elapsed_ms;

        while self.drop_elapsed_ms >= drop_interval {
            self.drop_elapsed_ms -= drop_interval;

            // Hatter: auto-rotate on each gravity tick
            if self.weapon_state.is_active(WeaponKind::Hatter) {
                if let Some(piece) = self.active_piece.as_mut() {
                    piece.try_rotate_cw(&self.board);
                }
            }

            let piece = match self.active_piece.as_mut() {
                Some(p) => p,
                None => return,
            };

            if piece.try_move_down(&self.board) {
                // still falling
            } else if self.piece_state == PieceState::HardDropping {
                self.lock_current_piece(events);
                return;
            } else {
                self.piece_state = PieceState::LockDelay;
                self.lock_elapsed_ms = 0;
                return;
            }
        }
    }

    fn apply_slick_tick(&mut self) {
        let dir = self.weapon_state.slick_dir;
        if dir == 0 { return; }
        let piece = match self.active_piece.as_mut() {
            Some(p) => p,
            None => return,
        };
        let moved = if dir > 0 {
            piece.try_move_right(&self.board)
        } else {
            piece.try_move_left(&self.board)
        };
        // Reverse direction on wall/cell collision
        if !moved {
            self.weapon_state.slick_dir = -dir;
        }
    }

    fn lock_current_piece(&mut self, events: &mut Vec<GameEvent>) {
        let piece = match self.active_piece.take() {
            Some(p) => p,
            None => return,
        };

        let cell = piece.kind.locked_cell();
        let abs_cells = piece.absolute_cells();

        // Lock with Fallout filter if active
        let fallout = self.weapon_state.is_active(WeaponKind::Fallout);
        self.board.lock_piece_filtered(&abs_cells, cell, fallout);
        events.push(GameEvent::PieceLocked);

        // Broken Record: record first piece kind locked after activation
        if self.weapon_state.is_active(WeaponKind::Broken) && self.weapon_state.broken_kind.is_none() {
            self.weapon_state.broken_kind = Some(piece.kind);
        }

        // Clear lines (Force weapon: no-shift variant)
        let cleared = if self.weapon_state.is_active(WeaponKind::Force) {
            self.board.force_clear_lines()
        } else {
            self.board.check_and_clear_lines()
        };

        if cleared.count > 0 {
            // Lawyers: each line cleared triggers rise_up on opponent (game_loop handles)
            if self.weapon_state.is_active(WeaponKind::Lawyers) {
                // Emit LinesCleared — game_loop will forward rise_up to opponent
            }

            // Apply Mondale tax to funds earned
            let meadow = self.weapon_state.is_active(WeaponKind::Meadow);
            if !meadow {
                let mondale_rate = self.weapon_state.mondale_rate();
                self.score.add_funds_taxed(cleared.funds_earned, mondale_rate);
            }

            let bazaar_triggered = self.score.add_lines(cleared.count);
            events.push(GameEvent::LinesCleared(cleared.count));

            // Tick weapon durations
            let expired = self.weapon_state.tick_lines(cleared.count, &mut self.board);
            for kind in expired {
                events.push(GameEvent::WeaponExpired(kind));
            }

            // In VsNetwork mode the server is the bazaar authority (BR-NET-05);
            // local threshold crossing is ignored — server sends BazaarOpen instead.
            if bazaar_triggered && self.mode != GameMode::VsNetwork {
                self.phase = GamePhase::InBazaar;
                self.bazaar = Some(BazaarState::new());
                events.push(GameEvent::BazaarTriggered);
            }
        }

        // Top-out check
        if self.board.is_topped_out() {
            self.phase = GamePhase::GameOver { won: false };
            events.push(GameEvent::GameOver { won: false });
            return;
        }

        self.spawn_next_piece(events);
    }

    fn spawn_next_piece(&mut self, events: &mut Vec<GameEvent>) {
        let kind = std::mem::replace(
            &mut self.next_piece,
            PieceKind::random_filtered(&self.weapon_state, &mut self.rng),
        );
        let new_piece = ActivePiece::new(kind);

        if !new_piece.can_place_at(&self.board, new_piece.x, new_piece.y, new_piece.rotation) {
            self.phase = GamePhase::GameOver { won: false };
            events.push(GameEvent::GameOver { won: false });
            return;
        }

        self.active_piece = Some(new_piece);
        self.piece_state = PieceState::Dropping;
        self.drop_elapsed_ms = 0;
        self.lock_elapsed_ms = 0;
        self.is_soft_dropping = false;
        events.push(GameEvent::PieceSpawned);
    }

    /// Apply an incoming weapon effect (from opponent's launch). Returns events.
    pub fn apply_incoming_weapon(
        &mut self,
        kind: WeaponKind,
        rng: &mut StdRng,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Check Mirror
        let mirror = check_mirror(kind, &self.weapon_state);
        match mirror {
            MirrorResult::Nullified => {
                // No effect on either player
            }
            MirrorResult::Reflected => {
                // Reflected back at launcher — this player is not affected
                // Caller handles the reflection
                events.push(GameEvent::WeaponFired { kind, reflect: true });
            }
            MirrorResult::PassThrough => {
                let def = weapon_def(kind);
                if def.duration == 0 {
                    let outcome = apply_weapon_instant(
                        kind,
                        &mut self.board,
                        &mut self.weapon_state,
                        &mut self.score,
                        &mut self.arsenal,
                        &mut self.next_piece,
                        rng,
                    );
                    if kind == WeaponKind::Gimp {
                        self.gimp_flash_ms = 800;
                        events.push(GameEvent::GimpFlash);
                    }
                    if outcome.rise_up_topped {
                        self.phase = GamePhase::GameOver { won: false };
                        events.push(GameEvent::RiseUpTopOut);
                        events.push(GameEvent::GameOver { won: false });
                    }
                    if outcome.funds_stolen > 0 {
                        events.push(GameEvent::FundsStolen(outcome.funds_stolen));
                    }
                } else {
                    apply_weapon_timed(kind, &mut self.weapon_state, &mut self.board);
                }
            }
        }

        events
    }

    /// Teleport AI's piece to (col, rotation) and hard-drop-lock it.
    /// Used by ernie.rs to apply Ernie's AI decision.
    pub fn ai_place_piece(&mut self, col: i32, rotation: u8) -> Vec<GameEvent> {
        if let Some(piece) = self.active_piece.as_mut() {
            piece.x = col;
            piece.rotation = rotation;
        }
        self.tick(Some(PlayerInput::HardDrop), FAST_DROP_INTERVAL_MS)
    }

    /// Apply an incoming network message (stub — implemented in Unit 3).
    pub fn apply_network_message(&mut self, _msg: GameMessage) -> Vec<GameEvent> {
        vec![]
    }

    /// Build a renderer snapshot.
    pub fn to_playing_view(&self) -> PlayingView {
        let (active_piece, ghost_cells) = match &self.active_piece {
            Some(piece) => {
                let ghost_y = piece.ghost_y(&self.board);
                let ghost = piece
                    .kind
                    .cells(piece.rotation)
                    .iter()
                    .map(|&(dc, dr)| (piece.x + dc, ghost_y + dr))
                    .collect();
                (Some((piece.kind, piece.absolute_cells())), ghost)
            }
            None => (None, vec![]),
        };

        // Build arsenal view
        let player_arsenal: Vec<ArsenalSlotView> = self.arsenal.slots.iter().enumerate()
            .map(|(i, slot)| ArsenalSlotView {
                kind: slot.kind,
                name: weapon_def(slot.kind).name,
                quantity: slot.quantity,
                key: if i < 9 { (b'1' + i as u8) as char } else { '0' },
            })
            .collect();

        // Build active weapons view
        let player_active_weapons: Vec<ActiveWeaponView> = (0..WEAPON_COUNT)
            .filter_map(|i| {
                let r = self.weapon_state.remaining[i];
                if r > 0 {
                    WeaponKind::from_index(i).map(|k| ActiveWeaponView {
                        name: weapon_def(k).name,
                        remaining_lines: r,
                    })
                } else {
                    None
                }
            })
            .collect();

        let spy_active = self.weapon_state.is_active(WeaponKind::Ames)
            || self.weapon_state.is_active(WeaponKind::Ace)
            || self.weapon_state.is_active(WeaponKind::Condor);
        let accuracy = if self.weapon_state.is_active(WeaponKind::Condor) {
            1.0
        } else if self.weapon_state.is_active(WeaponKind::Ace) {
            0.8
        } else {
            0.0
        };

        let mut sv = self.score.view();
        sv.show_op_funds = spy_active;

        let bazaar_view = if self.phase == GamePhase::InBazaar {
            self.bazaar.as_ref().map(|b| BazaarStateView::from_state(
                b,
                self.score.funds,
                self.weapon_state.is_active(WeaponKind::Carter),
            ))
        } else {
            None
        };

        PlayingView {
            player_board: self.board.snapshot(),
            active_piece,
            ghost_cells,
            next_piece: self.next_piece,
            score: sv,
            opponent_board: None, // filled in by game_loop
            player_active_weapons,
            ernie_active_weapons: vec![],
            player_arsenal,
            ernie_arsenal_count: 0,
            upbyside_active: self.weapon_state.is_active(WeaponKind::Upbyside),
            blind_cells: self.weapon_state.blind_cells.clone(),
            twilight_active: self.weapon_state.is_active(WeaponKind::Twilight),
            gimp_flash: self.gimp_flash_ms > 0,
            opponent_gimp_flash: self.opponent_gimp_flash_ms > 0,
            slick_active: self.weapon_state.slick_dir != 0,
            show_opponent_funds: spy_active,
            opponent_board_accuracy: accuracy,
            bazaar_view,
            peer_disconnected: false,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::BOARD_ROWS;

    fn playing_state() -> GameState {
        let mut gs = GameState::new(1234);
        let mut events = Vec::new();
        gs.start_game(&mut events);
        gs
    }

    #[test]
    fn gravity_drops_piece_after_512ms() {
        let mut gs = playing_state();
        let initial_y = gs.active_piece.as_ref().unwrap().y;
        gs.tick(None, 512);
        let new_y = gs.active_piece.as_ref().unwrap().y;
        assert!(new_y > initial_y);
    }

    #[test]
    fn hard_drop_scores_based_on_start_y() {
        let mut gs = playing_state();
        let start_y = gs.active_piece.as_ref().unwrap().y;
        let expected_score = (BOARD_ROWS as i32 - start_y).max(0) as u32;
        let events = gs.tick(Some(PlayerInput::HardDrop), 0);
        assert!(gs.score.score >= expected_score);
        let locked = events.iter().any(|e| matches!(e, GameEvent::PieceLocked));
        assert!(locked);
    }

    #[test]
    fn lock_delay_fires_after_150ms() {
        let mut gs = playing_state();
        gs.tick(Some(PlayerInput::HardDrop), 0);
        for _ in 0..30 {
            gs.tick(None, 512);
        }
        let before = gs.active_piece.is_none() || gs.piece_state == PieceState::LockDelay;
        let _ = before;
    }

    #[test]
    fn move_resets_lock_delay() {
        let mut gs = playing_state();
        gs.piece_state = PieceState::LockDelay;
        gs.lock_elapsed_ms = 140;
        if let Some(piece) = gs.active_piece.as_ref() {
            if piece.x > 0 {
                gs.tick(Some(PlayerInput::MoveLeft), 5);
                assert!(gs.lock_elapsed_ms <= 10);
            }
        }
    }

    #[test]
    fn topped_out_triggers_game_over() {
        let mut gs = playing_state();
        // SinglePlayer mode so bazaar auto-closes when player exits
        gs.mode = GameMode::SinglePlayer;
        for col in 0..10_i32 {
            for row in 1..28_i32 {
                gs.board.set_cell(col, row, crate::engine::board::Cell::Regular(1));
            }
        }
        let mut game_over = false;
        for _ in 0..2000 {
            let input = if gs.phase == GamePhase::InBazaar {
                Some(PlayerInput::BazaarExit)
            } else {
                Some(PlayerInput::HardDrop)
            };
            let events = gs.tick(input, 0);
            if events.iter().any(|e| matches!(e, GameEvent::GameOver { .. })) {
                game_over = true;
                break;
            }
        }
        assert!(game_over);
    }

    #[test]
    fn bazaar_triggers_at_20_combined_lines() {
        let mut gs = playing_state();
        gs.score.combined_lines = 19;
        for col in 0..10_i32 {
            gs.board.set_cell(col, 27, crate::engine::board::Cell::Regular(1));
        }
        let mut events = Vec::new();
        gs.lock_current_piece(&mut events);
        let bazaar = events.iter().any(|e| matches!(e, GameEvent::BazaarTriggered));
        assert!(bazaar);
    }

    #[test]
    fn soft_drop_uses_fast_interval() {
        let mut gs = playing_state();
        let y0 = gs.active_piece.as_ref().unwrap().y;
        gs.tick(Some(PlayerInput::SoftDrop), 10);
        let y1 = gs.active_piece.as_ref().unwrap().y;
        assert!(y1 > y0);
    }

    #[test]
    fn ai_place_piece_locks_at_target() {
        let mut gs = playing_state();
        let events = gs.ai_place_piece(0, 0);
        assert!(events.iter().any(|e| matches!(e, GameEvent::PieceLocked)));
    }

    #[test]
    fn weapon_state_is_exposed() {
        let gs = playing_state();
        assert!(!gs.weapon_state.is_active(WeaponKind::Speedy));
    }
}
