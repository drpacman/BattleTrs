use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use battletris_engine::engine::board::BoardSnapshot;
use battletris_engine::engine::game_state::{
    GameEvent, GameMode, GamePhase, GameState, PlayerInput,
};
use battletris_engine::engine::score::ScoreView;
use battletris_engine::protocol::GameMessage;

use crate::input::InputHandler;
use crate::renderer::{CanvasRenderer};
use crate::renderer::overlay::{draw_bazaar, draw_quit_confirm};
use crate::renderer::playing::draw_playing;
use crate::renderer::screens::{
    draw_connecting, draw_disconnected, draw_game_over, draw_name_taken, draw_waiting,
};
use crate::transport::WsTransport;

#[derive(Debug, PartialEq, Clone, Copy)]
enum WebPhase {
    Connecting,
    WaitingForOpponent,
    InGame,
    NameTaken,
    Disconnected,
}

pub struct WasmApp {
    state: GameState,
    transport: WsTransport,
    input: InputHandler,
    renderer: CanvasRenderer,

    web_phase: WebPhase,
    my_name: String,

    peer_board: Option<BoardSnapshot>,
    peer_score: Option<ScoreView>,
    peer_disconnected: bool,
    network_result: Option<(bool, String, i32)>,
    did_send_game_over: bool,

    in_quit_confirm: bool,
    last_ts: f64,
}

impl WasmApp {
    pub fn new() -> Result<Self, wasm_bindgen::JsValue> {
        let window = web_sys::window().unwrap();
        let name = window
            .prompt_with_message("Enter your player name:")?
            .unwrap_or_default();
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(wasm_bindgen::JsValue::from_str("player name required"));
        }

        let ws_url = ws_url_from_location();
        let transport = WsTransport::connect(&ws_url, &name)?;
        let input = InputHandler::new();
        let renderer = CanvasRenderer::new()?;

        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        let mut state = GameState::new(seed);
        state.mode = GameMode::VsNetwork;

        Ok(Self {
            state,
            transport,
            input,
            renderer,
            web_phase: WebPhase::Connecting,
            my_name: name,
            peer_board: None,
            peer_score: None,
            peer_disconnected: false,
            network_result: None,
            did_send_game_over: false,
            in_quit_confirm: false,
            last_ts: 0.0,
        })
    }

    pub fn tick(&mut self, ts: f64) {
        let elapsed_ms = if self.last_ts == 0.0 {
            0
        } else {
            (ts - self.last_ts).max(0.0).min(100.0) as u32
        };
        self.last_ts = ts;

        // Update connection state
        if self.web_phase == WebPhase::Connecting && self.transport.is_connected() {
            self.web_phase = WebPhase::WaitingForOpponent;
        }
        if self.transport.is_disconnected()
            && !matches!(self.web_phase, WebPhase::NameTaken | WebPhase::Disconnected)
        {
            self.web_phase = WebPhase::Disconnected;
        }

        // Process incoming WS messages
        for msg in self.transport.drain_incoming() {
            self.process_message(msg);
        }

        let in_bazaar = matches!(self.state.phase, GamePhase::InBazaar);
        let in_game_over = matches!(self.state.phase, GamePhase::GameOver { .. });

        // Drain keys
        let keys = self.input.drain();

        // Quit confirm: Y/N intercept
        if self.in_quit_confirm {
            for key in &keys {
                if key == "KeyY" {
                    let _ = web_sys::window().unwrap().location().reload();
                    return;
                } else if key == "KeyN" || key == "Escape" {
                    self.in_quit_confirm = false;
                }
            }
            self.render();
            return;
        }

        // Game over: reload on Enter/Space
        if in_game_over {
            for key in &keys {
                if key == "Enter" || key == "NumpadEnter" || key == "Space" {
                    let _ = web_sys::window().unwrap().location().reload();
                    return;
                }
            }
        }

        // Detect Escape in Playing → enter quit confirm
        if self.web_phase == WebPhase::InGame
            && matches!(self.state.phase, GamePhase::Playing)
        {
            for key in &keys {
                if key == "Escape" {
                    self.in_quit_confirm = true;
                    self.render();
                    return;
                }
            }
        }

        // Map keys to PlayerInput (last non-null wins)
        let mut input: Option<PlayerInput> = None;
        for key in &keys {
            if let Some(pi) = key_to_input(key, in_bazaar) {
                input = Some(pi);
            }
        }

        // Tick game engine when in-game
        if self.web_phase == WebPhase::InGame {
            let was_playing =
                matches!(self.state.phase, GamePhase::Playing | GamePhase::InBazaar);
            let events = self.state.tick(input, elapsed_ms);
            let now_loss = matches!(self.state.phase, GamePhase::GameOver { won: false });

            if was_playing && now_loss && !self.did_send_game_over {
                self.did_send_game_over = true;
                self.transport.send(&GameMessage::GameOver {
                    winner_id: 0,
                    final_score_p1: self.state.score.score,
                    final_score_p2: 0,
                    winner_name: String::new(),
                    elo_delta_winner: 0,
                    elo_delta_loser: 0,
                });
            }

            self.forward_events(&events);

            if matches!(self.state.phase, GamePhase::Playing | GamePhase::InBazaar) {
                self.transport.send(&GameMessage::ScoreUpdate {
                    score: self.state.score.score,
                    lines: self.state.score.lines,
                    funds: self.state.score.funds,
                });
            }
        }

        self.render();
    }

    fn process_message(&mut self, msg: GameMessage) {
        match msg {
            GameMessage::Welcome { assigned_name } => {
                self.my_name = assigned_name;
            }
            GameMessage::NameTaken => {
                self.web_phase = WebPhase::NameTaken;
            }
            GameMessage::GameStart => {
                self.web_phase = WebPhase::InGame;
                self.peer_disconnected = false;
                self.state.tick(Some(PlayerInput::StartGame), 0);
            }
            GameMessage::BazaarOpen => {
                self.state.open_bazaar_now();
                self.state.ernie_bazaar_done();
            }
            GameMessage::PeerDisconnected => {
                self.peer_disconnected = true;
            }
            GameMessage::PeerReconnected => {
                self.peer_disconnected = false;
            }
            GameMessage::GameVoid => {
                self.state.phase = GamePhase::Title;
                self.web_phase = WebPhase::WaitingForOpponent;
            }
            GameMessage::BoardUpdate { snapshot } => {
                self.peer_board = Some(snapshot);
            }
            GameMessage::ScoreUpdate { score, lines, funds } => {
                let mut sv = ScoreView::default();
                sv.score = score;
                sv.lines = lines;
                sv.funds = funds;
                self.peer_score = Some(sv);
            }
            GameMessage::WeaponLaunched { kind } => {
                let mut rng = StdRng::from_entropy();
                let events = self.state.apply_incoming_weapon(kind, &mut rng);
                for ev in &events {
                    if let GameEvent::WeaponFired { kind: k, reflect: true } = ev {
                        self.transport.send(&GameMessage::WeaponReflected { kind: *k });
                    }
                }
            }
            GameMessage::WeaponReflected { kind } => {
                let mut rng = StdRng::from_entropy();
                self.state.apply_incoming_weapon(kind, &mut rng);
            }
            GameMessage::FundsReceived { amount } => {
                self.state.score.funds += amount;
            }
            GameMessage::GameOver { winner_name, elo_delta_winner, elo_delta_loser, .. } => {
                let i_won = self.my_name == winner_name;
                let my_delta = if i_won { elo_delta_winner } else { elo_delta_loser };
                self.network_result = Some((i_won, winner_name, my_delta));
                self.state.phase = GamePhase::GameOver { won: i_won };
            }
            _ => {}
        }
    }

    fn forward_events(&mut self, events: &[GameEvent]) {
        for event in events {
            match event {
                GameEvent::WeaponFired { kind, reflect: false } => {
                    self.transport.send(&GameMessage::WeaponLaunched { kind: *kind });
                }
                GameEvent::LinesCleared(n) => {
                    self.transport.send(&GameMessage::LinesCleared {
                        count: *n as u32,
                        funds_earned: 0,
                    });
                }
                GameEvent::PieceLocked => {
                    self.transport.send(&GameMessage::BoardUpdate {
                        snapshot: self.state.board.snapshot(),
                    });
                }
                _ => {}
            }
        }
    }

    fn render(&mut self) {
        self.renderer.clear();
        // Render based on web phase
        match self.web_phase {
            WebPhase::Connecting => draw_connecting(&self.renderer.ctx),
            WebPhase::WaitingForOpponent => draw_waiting(&self.renderer.ctx),
            WebPhase::NameTaken => draw_name_taken(&self.renderer.ctx),
            WebPhase::Disconnected => draw_disconnected(&self.renderer.ctx),
            WebPhase::InGame => self.render_in_game(),
        }
    }

    fn render_in_game(&mut self) {
        let in_baz = matches!(self.state.phase, GamePhase::InBazaar);
        let phase = self.state.phase.clone();

        match phase {
            GamePhase::Playing | GamePhase::InBazaar => {
                let mut view = self.state.to_playing_view();
                view.opponent_board =
                    apply_board_visibility(&self.peer_board, view.opponent_board_accuracy);
                view.peer_disconnected = self.peer_disconnected;
                if let Some(ref sv) = self.peer_score {
                    view.score.op_score = sv.score;
                    view.score.op_lines = sv.lines;
                    if view.score.show_op_funds {
                        view.score.op_funds = sv.funds;
                    }
                }
                if in_baz {
                    if let Some(ref bv) = view.bazaar_view {
                        draw_bazaar(&self.renderer.ctx, bv);
                    }
                } else {
                    draw_playing(&self.renderer.ctx, &view);
                    if self.in_quit_confirm {
                        draw_quit_confirm(&self.renderer.ctx);
                    }
                }
            }
            GamePhase::GameOver { won } => {
                let (winner_name, elo_delta) = match &self.network_result {
                    Some((_, name, delta)) => (Some(name.as_str()), Some(*delta)),
                    None => (None, None),
                };
                draw_game_over(
                    &self.renderer.ctx,
                    won,
                    self.state.score.score,
                    self.state.score.lines,
                    winner_name,
                    elo_delta,
                );
            }
            _ => draw_waiting(&self.renderer.ctx),
        }
    }
}

fn key_to_input(code: &str, in_bazaar: bool) -> Option<PlayerInput> {
    if in_bazaar {
        return match code {
            "ArrowUp" | "PageUp" => Some(PlayerInput::BazaarUp),
            "ArrowDown" | "PageDown" => Some(PlayerInput::BazaarDown),
            "Enter" | "NumpadEnter" => Some(PlayerInput::BazaarBuy),
            "Escape" => Some(PlayerInput::BazaarExit),
            _ => None,
        };
    }
    match code {
        "ArrowLeft" => Some(PlayerInput::MoveLeft),
        "ArrowRight" => Some(PlayerInput::MoveRight),
        "ArrowUp" => Some(PlayerInput::RotateCW),
        "KeyZ" => Some(PlayerInput::RotateCCW),
        "ArrowDown" => Some(PlayerInput::SoftDrop),
        "UP:ArrowDown" => Some(PlayerInput::SoftDropRelease),
        "Space" => Some(PlayerInput::HardDrop),
        "Enter" | "NumpadEnter" => Some(PlayerInput::StartGame),
        "KeyP" => Some(PlayerInput::Pause),
        "KeyB" => Some(PlayerInput::OpenBazaar),
        "Digit0" => Some(PlayerInput::LaunchWeapon(9)),
        "Digit1" => Some(PlayerInput::LaunchWeapon(0)),
        "Digit2" => Some(PlayerInput::LaunchWeapon(1)),
        "Digit3" => Some(PlayerInput::LaunchWeapon(2)),
        "Digit4" => Some(PlayerInput::LaunchWeapon(3)),
        "Digit5" => Some(PlayerInput::LaunchWeapon(4)),
        "Digit6" => Some(PlayerInput::LaunchWeapon(5)),
        "Digit7" => Some(PlayerInput::LaunchWeapon(6)),
        "Digit8" => Some(PlayerInput::LaunchWeapon(7)),
        "Digit9" => Some(PlayerInput::LaunchWeapon(8)),
        _ => None,
    }
}

fn apply_board_visibility(board: &Option<BoardSnapshot>, accuracy: f32) -> Option<BoardSnapshot> {
    use battletris_engine::engine::board::Cell;
    if accuracy <= 0.0 {
        return None;
    }
    let Some(snapshot) = board else {
        return None;
    };
    if accuracy >= 1.0 {
        return Some(snapshot.clone());
    }
    let mut rng = rand::thread_rng();
    let mut cells = snapshot.cells;
    for row in cells.iter_mut() {
        for cell in row.iter_mut() {
            if !cell.is_empty() && rng.gen::<f32>() > accuracy {
                *cell = Cell::Empty;
            }
        }
    }
    Some(BoardSnapshot { cells })
}

fn ws_url_from_location() -> String {
    let window = web_sys::window().unwrap();
    let location = window.location();

    if let Ok(search) = location.search() {
        for part in search.split('&') {
            let part = part.trim_start_matches('?');
            if part.starts_with("server=") {
                return part["server=".len()..].to_string();
            }
        }
    }

    let origin = location.origin().unwrap_or_default();
    origin.replace("https://", "wss://").replace("http://", "ws://") + "/game"
}
