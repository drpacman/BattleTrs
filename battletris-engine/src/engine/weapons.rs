use rand::{Rng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use crate::engine::board::{Board, Cell};
use crate::engine::piece::PieceKind;
use crate::engine::score::Score;

pub const WEAPON_COUNT: usize = 34;
pub const ARSENAL_SIZE: usize = 10;

// ─── WeaponKind ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(usize)]
pub enum WeaponKind {
    FW       = 0,
    FBF      = 1,
    Hatter   = 2,
    Upbyside = 3,
    Fallout  = 4,
    Swap     = 5,
    Lawyers  = 6,
    RiseUp   = 7,
    FlipOut  = 8,
    Speedy   = 9,
    Missing  = 10,
    PieceIt  = 11,
    Blind    = 12,
    Mondale  = 13,
    Keating  = 14,
    Carter   = 15,
    Reagan   = 16,
    Ames     = 17,
    Ace      = 18,
    Condor   = 19,
    NiceDay  = 20,
    SoLong   = 21,
    NoDice   = 22,
    Bug      = 23,
    Bottle   = 24,
    NoSlide  = 25,
    Susan    = 26,
    Meadow   = 27,
    Mirror   = 28,
    Twilight = 29,
    Slick    = 30,
    Broken   = 31,
    Force    = 32,
    Gimp     = 33,
}

impl WeaponKind {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0  => Some(WeaponKind::FW),
            1  => Some(WeaponKind::FBF),
            2  => Some(WeaponKind::Hatter),
            3  => Some(WeaponKind::Upbyside),
            4  => Some(WeaponKind::Fallout),
            5  => Some(WeaponKind::Swap),
            6  => Some(WeaponKind::Lawyers),
            7  => Some(WeaponKind::RiseUp),
            8  => Some(WeaponKind::FlipOut),
            9  => Some(WeaponKind::Speedy),
            10 => Some(WeaponKind::Missing),
            11 => Some(WeaponKind::PieceIt),
            12 => Some(WeaponKind::Blind),
            13 => Some(WeaponKind::Mondale),
            14 => Some(WeaponKind::Keating),
            15 => Some(WeaponKind::Carter),
            16 => Some(WeaponKind::Reagan),
            17 => Some(WeaponKind::Ames),
            18 => Some(WeaponKind::Ace),
            19 => Some(WeaponKind::Condor),
            20 => Some(WeaponKind::NiceDay),
            21 => Some(WeaponKind::SoLong),
            22 => Some(WeaponKind::NoDice),
            23 => Some(WeaponKind::Bug),
            24 => Some(WeaponKind::Bottle),
            25 => Some(WeaponKind::NoSlide),
            26 => Some(WeaponKind::Susan),
            27 => Some(WeaponKind::Meadow),
            28 => Some(WeaponKind::Mirror),
            29 => Some(WeaponKind::Twilight),
            30 => Some(WeaponKind::Slick),
            31 => Some(WeaponKind::Broken),
            32 => Some(WeaponKind::Force),
            33 => Some(WeaponKind::Gimp),
            _  => None,
        }
    }
}

// ─── WeaponDef ───────────────────────────────────────────────────────────────

pub struct WeaponDef {
    pub kind: WeaponKind,
    pub name: &'static str,
    pub description: &'static str,
    pub price: u32,
    pub duration: u32,
}

pub static WEAPONS: [WeaponDef; WEAPON_COUNT] = [
    WeaponDef { kind: WeaponKind::FW,       name: "The Feared Weird",        description: "Forces opponent to receive only bizarre, hard-to-place pieces.",               price: 400,  duration: 3  },
    WeaponDef { kind: WeaponKind::FBF,      name: "Four-by-Four",            description: "Every Box piece becomes a massive hollow 4x4 square.",                        price: 425,  duration: 10 },
    WeaponDef { kind: WeaponKind::Hatter,   name: "The Mad Hatter",          description: "Opponent's current piece rotates automatically every tick.",                  price: 375,  duration: 5  },
    WeaponDef { kind: WeaponKind::Upbyside, name: "Upbyside-down",           description: "Flips opponent's board upside-down - pieces fall from the top.",              price: 125,  duration: 10 },
    WeaponDef { kind: WeaponKind::Fallout,  name: "Fallout",                 description: "Columns 2-7 become black holes - pieces that land there vanish.",             price: 250,  duration: 10 },
    WeaponDef { kind: WeaponKind::Swap,     name: "Swap Meet",               description: "Instantly exchange your board with your opponent's.",                         price: 1200, duration: 0  },
    WeaponDef { kind: WeaponKind::Lawyers,  name: "Lawyer's Delite",         description: "Each line you clear also adds a junk row to your opponent.",                  price: 350,  duration: 5  },
    WeaponDef { kind: WeaponKind::RiseUp,   name: "Rise Up",                 description: "Adds one junk row to the bottom of opponent's board.",                        price: 75,   duration: 0  },
    WeaponDef { kind: WeaponKind::FlipOut,  name: "Flip Out",                description: "Mirrors opponent's board horizontally.",                                      price: 15,   duration: 0  },
    WeaponDef { kind: WeaponKind::Speedy,   name: "Speedy Gonzales",         description: "Speeds up opponent's piece drop rate dramatically.",                          price: 275,  duration: 10 },
    WeaponDef { kind: WeaponKind::Missing,  name: "Missing Pieces",          description: "Removes a random cell from opponent's board.",                                price: 50,   duration: 0  },
    WeaponDef { kind: WeaponKind::PieceIt,  name: "Piece It Together",       description: "Drops a random piece directly onto opponent's board.",                        price: 100,  duration: 0  },
    WeaponDef { kind: WeaponKind::Blind,    name: "The Blind Cleric",        description: "Permanently blinds several cells on opponent's board.",                       price: 400,  duration: 0  },
    WeaponDef { kind: WeaponKind::Mondale,  name: "Mondale '96",             description: "Taxes 30% of all funds opponent earns for 50 lines.",                        price: 150,  duration: 50 },
    WeaponDef { kind: WeaponKind::Keating,  name: "Keating Five",            description: "Steals all of opponent's funds instantly.",                                   price: 425,  duration: 0  },
    WeaponDef { kind: WeaponKind::Carter,   name: "Carter Years",            description: "Doubles weapon prices in opponent's bazaar for 20 lines.",                   price: 250,  duration: 20 },
    WeaponDef { kind: WeaponKind::Reagan,   name: "Reagan Era",              description: "Negates opponent's funds - turns positive funds negative.",                   price: 425,  duration: 0  },
    WeaponDef { kind: WeaponKind::Ames,     name: "William Ames",            description: "Reveals opponent's exact funds to you for 20 lines.",                        price: 50,   duration: 20 },
    WeaponDef { kind: WeaponKind::Ace,      name: "Ace of Spies",            description: "Shows opponent's board at 80% accuracy for 30 lines.",                       price: 100,  duration: 30 },
    WeaponDef { kind: WeaponKind::Condor,   name: "The Condor",              description: "Reveals opponent's full board and funds for 40 lines.",                      price: 225,  duration: 40 },
    WeaponDef { kind: WeaponKind::NiceDay,  name: "Have a Nice Day",         description: "Forces a Happy (smiley) piece as opponent's next piece.",                    price: 50,   duration: 0  },
    WeaponDef { kind: WeaponKind::SoLong,   name: "So Long",                 description: "Removes the Long piece from opponent's pool for 10 lines.",                  price: 100,  duration: 10 },
    WeaponDef { kind: WeaponKind::NoDice,   name: "No Dice",                 description: "Removes the Die piece from opponent's pool for 35 lines.",                   price: 600,  duration: 35 },
    WeaponDef { kind: WeaponKind::Bug,      name: "Bug Report",              description: "Places invisible solid cells on opponent's board.",                           price: 320,  duration: 0  },
    WeaponDef { kind: WeaponKind::Bottle,   name: "Bottle Neck",             description: "Narrows opponent's playfield to 4 columns in the middle zone.",              price: 150,  duration: 10 },
    WeaponDef { kind: WeaponKind::NoSlide,  name: "Slide Denied",            description: "Opponent cannot slide pieces left or right.",                                price: 125,  duration: 10 },
    WeaponDef { kind: WeaponKind::Susan,    name: "Lazy Susan",              description: "Swaps your entire arsenal with your opponent's arsenal.",                    price: 600,  duration: 0  },
    WeaponDef { kind: WeaponKind::Meadow,   name: "Meadow",                  description: "Opponent earns no funds from line clears.",                                  price: 475,  duration: 10 },
    WeaponDef { kind: WeaponKind::Mirror,   name: "Mirror Mirror",           description: "Reflects or nullifies incoming weapons for 10 lines.",                       price: 500,  duration: 10 },
    WeaponDef { kind: WeaponKind::Twilight, name: "The Twilight Zone",       description: "Turns all of opponent's existing cells invisible.",                           price: 450,  duration: 0  },
    WeaponDef { kind: WeaponKind::Slick,    name: "Slick Willy",             description: "Opponent's piece slides sideways automatically.",                            price: 650,  duration: 3  },
    WeaponDef { kind: WeaponKind::Broken,   name: "Broken Record",           description: "Opponent receives the same piece repeatedly for 5 lines.",                  price: 325,  duration: 5  },
    WeaponDef { kind: WeaponKind::Force,    name: "The Force",               description: "Cleared lines are zeroed in place - rows above don't shift down.",          price: 325,  duration: 5  },
    WeaponDef { kind: WeaponKind::Gimp,     name: "The Gimp",                description: "Flashes an embarrassing overlay on your opponent's board briefly.",         price: 25,   duration: 0  },
];

pub fn weapon_def(kind: WeaponKind) -> &'static WeaponDef {
    &WEAPONS[kind.index()]
}

// ─── Arsenal ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArsenalSlot {
    pub kind: WeaponKind,
    pub quantity: u8,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Arsenal {
    pub slots: Vec<ArsenalSlot>,
}

impl Arsenal {
    pub fn new() -> Self {
        Arsenal { slots: Vec::new() }
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn is_full(&self) -> bool {
        self.slots.len() >= ARSENAL_SIZE
    }

    pub fn can_add(&self, kind: WeaponKind) -> bool {
        // Can stack if already have this kind, or have empty slot
        self.slots.iter().any(|s| s.kind == kind) || self.slots.len() < ARSENAL_SIZE
    }

    pub fn add(&mut self, kind: WeaponKind) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.kind == kind) {
            slot.quantity = slot.quantity.saturating_add(1);
            return true;
        }
        if self.slots.len() >= ARSENAL_SIZE {
            return false;
        }
        self.slots.push(ArsenalSlot { kind, quantity: 1 });
        true
    }

    /// Remove one from slot at index. Returns kind if successful.
    pub fn remove_slot(&mut self, slot_index: usize) -> Option<WeaponKind> {
        if slot_index >= self.slots.len() {
            return None;
        }
        let kind = self.slots[slot_index].kind;
        self.slots[slot_index].quantity -= 1;
        if self.slots[slot_index].quantity == 0 {
            self.slots.remove(slot_index);
        }
        Some(kind)
    }

    pub fn clear(&mut self) {
        self.slots.clear();
    }
}

// ─── WeaponState ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeaponState {
    /// Lines remaining for each timed weapon (index = WeaponKind::index()).
    pub remaining: Vec<u32>,
    /// Slick Willy drift direction: 1=right, -1=left, 0=inactive.
    pub slick_dir: i32,
    /// Piece locked in by Broken Record.
    pub broken_kind: Option<PieceKind>,
    /// Permanently blinded cell positions (row, col).
    pub blind_cells: Vec<(usize, usize)>,
    /// Active Mondale stack count (each launch = +1 stack, stacks when remaining > 0).
    pub mondale_stacks: u8,
    /// Next piece should be Happy (NiceDay weapon).
    pub nice_day_pending: bool,
}

impl Default for WeaponState {
    fn default() -> Self {
        WeaponState {
            remaining: vec![0u32; WEAPON_COUNT],
            slick_dir: 0,
            broken_kind: None,
            blind_cells: Vec::new(),
            mondale_stacks: 0,
            nice_day_pending: false,
        }
    }
}

impl WeaponState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_active(&self, kind: WeaponKind) -> bool {
        self.remaining[kind.index()] > 0
    }

    /// Activate a timed weapon: add its duration to remaining, apply board-level side effects.
    pub fn activate(&mut self, kind: WeaponKind, board: &mut Board) {
        let def = weapon_def(kind);
        if def.duration > 0 {
            self.remaining[kind.index()] += def.duration;
            if matches!(kind, WeaponKind::Mondale) {
                self.mondale_stacks = self.mondale_stacks.saturating_add(1);
            }
            match kind {
                WeaponKind::Bottle => board.fill_bottle_walls(),
                WeaponKind::Slick => {
                    if self.slick_dir == 0 {
                        self.slick_dir = 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// Deactivate a weapon when its remaining reaches 0.
    pub fn deactivate(&mut self, kind: WeaponKind, board: &mut Board) {
        match kind {
            WeaponKind::Bottle => board.clear_bottle_walls(),
            WeaponKind::Slick => {
                if self.remaining[WeaponKind::Slick.index()] == 0 {
                    self.slick_dir = 0;
                }
            }
            WeaponKind::Broken => {
                if self.remaining[WeaponKind::Broken.index()] == 0 {
                    self.broken_kind = None;
                }
            }
            WeaponKind::Mondale => {
                if self.remaining[WeaponKind::Mondale.index()] == 0 {
                    self.mondale_stacks = 0;
                }
            }
            _ => {}
        }
    }

    /// Decrement all active weapon durations by `lines`. Returns expired weapon kinds.
    pub fn tick_lines(&mut self, lines: u32, board: &mut Board) -> Vec<WeaponKind> {
        let mut expired = Vec::new();
        for i in 0..WEAPON_COUNT {
            if self.remaining[i] > 0 {
                self.remaining[i] = self.remaining[i].saturating_sub(lines);
                if self.remaining[i] == 0 {
                    if let Some(kind) = WeaponKind::from_index(i) {
                        self.deactivate(kind, board);
                        expired.push(kind);
                    }
                }
            }
        }
        expired
    }

    /// Effective Mondale tax rate as a percentage (0–90).
    pub fn mondale_rate(&self) -> u8 {
        match self.mondale_stacks {
            0 => 0,
            1 => 30,
            2 => 51,  // 1 - 0.7^2 ≈ 51%
            _ => 90,  // cap at 90%
        }
    }

    /// Drop interval multiplier when Speedy is active (1 = normal, 5 = 5× faster).
    pub fn speedy_multiplier(&self) -> u32 {
        if self.is_active(WeaponKind::Speedy) { 5 } else { 1 }
    }

    /// True if NoSlide is active (left/right input blocked).
    pub fn no_slide(&self) -> bool {
        self.is_active(WeaponKind::NoSlide)
    }
}

// ─── Mirror check ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MirrorResult {
    PassThrough,
    Nullified,
    Reflected,
}

/// Check whether the target's Mirror weapon affects the incoming weapon.
pub fn check_mirror(kind: WeaponKind, target_ws: &WeaponState) -> MirrorResult {
    if !target_ws.is_active(WeaponKind::Mirror) {
        return MirrorResult::PassThrough;
    }
    // Weapons nullified by Mirror (not reflected, just no effect)
    if matches!(kind,
        WeaponKind::Swap | WeaponKind::Mondale | WeaponKind::Keating |
        WeaponKind::Ames | WeaponKind::Ace | WeaponKind::Condor |
        WeaponKind::NiceDay | WeaponKind::Susan | WeaponKind::Mirror)
    {
        return MirrorResult::Nullified;
    }
    // All other weapons are reflected back at the launcher
    MirrorResult::Reflected
}

// ─── Weapon launch outcome ───────────────────────────────────────────────────

/// Result of attempting to apply a weapon to a target.
pub struct LaunchOutcome {
    pub mirror: MirrorResult,
    /// Funds stolen from target (Keating). Applied by caller to source funds.
    pub funds_stolen: i64,
    /// True if target was topped-out by a RiseUp that this weapon triggered.
    pub rise_up_topped: bool,
}

impl Default for LaunchOutcome {
    fn default() -> Self {
        LaunchOutcome {
            mirror: MirrorResult::PassThrough,
            funds_stolen: 0,
            rise_up_topped: false,
        }
    }
}

// ─── Instant weapon effects ───────────────────────────────────────────────────

/// Apply an instant (duration=0) weapon to the TARGET player's state.
/// Does NOT check Mirror — caller must do that first.
/// Does NOT handle Swap, Susan (cross-player, handled by caller).
/// Returns a LaunchOutcome with stolen funds and rise_up_topped flag.
pub fn apply_weapon_instant(
    kind: WeaponKind,
    tgt_board: &mut Board,
    tgt_ws: &mut WeaponState,
    tgt_score: &mut Score,
    _tgt_arsenal: &mut Arsenal,
    tgt_next: &mut PieceKind,
    rng: &mut StdRng,
) -> LaunchOutcome {
    let mut outcome = LaunchOutcome::default();

    match kind {
        WeaponKind::RiseUp => {
            outcome.rise_up_topped = tgt_board.rise_up(rng);
        }
        WeaponKind::FlipOut => {
            tgt_board.flip_out();
        }
        WeaponKind::Missing => {
            tgt_board.remove_random_cell(rng);
        }
        WeaponKind::PieceIt => {
            // Drop a plug-shaped piece at a random column on target board
            let col = rng.gen_range(1..7i32);
            let row = 0i32;
            let piece_cells = PieceKind::Plug.cells(0);
            // Place as Regular cells
            tgt_board.add_piece_at(piece_cells, col, row, Cell::Regular(rng.gen_range(1u8..=8)));
        }
        WeaponKind::Bug => {
            // Place 4 invisible Bug cells in a 2×2 at a random upper-mid column
            let col = rng.gen_range(1..8i32);
            let row = rng.gen_range(8..18i32);
            let bug_cells = [(0i32, 0i32), (1, 0), (0, 1), (1, 1)];
            tgt_board.add_piece_at(&bug_cells, col, row, Cell::Bug);
        }
        WeaponKind::Blind => {
            // Blind 6 random non-empty cells (or random cells if board is sparse)
            for _ in 0..6 {
                let r = rng.gen_range(0..28usize);
                let c = rng.gen_range(0..10usize);
                if !tgt_ws.blind_cells.contains(&(r, c)) {
                    tgt_ws.blind_cells.push((r, c));
                }
            }
        }
        WeaponKind::Twilight => {
            tgt_board.apply_twilight();
        }
        WeaponKind::Keating => {
            // Steal ALL of target's funds
            let stolen = tgt_score.funds.max(0);
            tgt_score.funds -= stolen;
            outcome.funds_stolen = stolen;
        }
        WeaponKind::Reagan => {
            // Negate target's funds (positive → negative)
            if tgt_score.funds > 0 {
                tgt_score.funds = -tgt_score.funds;
            }
        }
        WeaponKind::NiceDay => {
            // Target's next piece becomes Happy
            *tgt_next = PieceKind::Happy;
        }
        WeaponKind::Gimp => {
            // Visual flash only — no board state change; caller handles render flag
        }
        _ => {
            // Non-instant weapons should not reach this function
        }
    }

    outcome
}

/// Apply a timed weapon to the target (set active state + board effects).
/// Must be called in addition to weapon_state.activate().
pub fn apply_weapon_timed(kind: WeaponKind, tgt_ws: &mut WeaponState, tgt_board: &mut Board) {
    tgt_ws.activate(kind, tgt_board);
}

/// Swap boards and weapon states between two players (Swap Meet weapon).
pub fn do_swap(a_board: &mut Board, a_ws: &mut WeaponState, b_board: &mut Board, b_ws: &mut WeaponState) {
    std::mem::swap(a_board, b_board);
    // Weapon states stay attached to the BOARD, not the player (BR-BE03)
    std::mem::swap(a_ws, b_ws);
}

/// Swap arsenals between two players (Lazy Susan weapon).
pub fn do_susan(a_arsenal: &mut Arsenal, b_arsenal: &mut Arsenal) {
    std::mem::swap(a_arsenal, b_arsenal);
}

// ─── BazaarState ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct BazaarState {
    /// All 34 weapons sorted by price ascending (BR-B04).
    pub weapons: Vec<WeaponKind>,
    pub selected: usize,
    pub player_done: bool,
    pub ernie_done: bool,
}

impl BazaarState {
    pub fn new() -> Self {
        let mut kinds: Vec<WeaponKind> = (0..WEAPON_COUNT)
            .filter_map(WeaponKind::from_index)
            .collect();
        kinds.sort_by_key(|&k| weapon_def(k).price);
        BazaarState {
            weapons: kinds,
            selected: 0,
            player_done: false,
            ernie_done: false,
        }
    }

    pub fn navigate_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected < WEAPON_COUNT - 1 {
            self.selected += 1;
        }
    }

    pub fn current_kind(&self) -> WeaponKind {
        self.weapons[self.selected]
    }

    /// Attempt to buy the selected weapon. Returns true on success.
    pub fn try_buy(&self, score: &mut Score, arsenal: &mut Arsenal, carter_active: bool) -> bool {
        let kind = self.current_kind();
        let def = weapon_def(kind);
        let price = if carter_active { def.price * 2 } else { def.price };
        if score.funds >= price as i64 && arsenal.can_add(kind) {
            score.funds -= price as i64;
            arsenal.add(kind);
            true
        } else {
            false
        }
    }
}

impl Default for BazaarState {
    fn default() -> Self {
        BazaarState::new()
    }
}

// ─── Renderer view types ──────────────────────────────────────────────────────

/// Renderer-safe view of one arsenal slot.
#[derive(Clone, Debug)]
pub struct ArsenalSlotView {
    pub kind: WeaponKind,
    pub name: &'static str,
    pub quantity: u8,
    pub key: char,
}

/// Renderer-safe view of an active weapon indicator.
#[derive(Clone, Debug)]
pub struct ActiveWeaponView {
    pub name: &'static str,
    pub remaining_lines: u32,
}

/// Renderer-safe view of bazaar state.
#[derive(Clone, Debug)]
pub struct BazaarStateView {
    pub weapons: Vec<WeaponKind>,
    pub selected: usize,
    pub player_funds: i64,
    pub carter_active: bool,
}

impl BazaarStateView {
    pub fn from_state(state: &BazaarState, funds: i64, carter_active: bool) -> Self {
        BazaarStateView {
            weapons: state.weapons.clone(),
            selected: state.selected,
            player_funds: funds,
            carter_active,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn test_rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    fn empty_board() -> Board {
        Board::new()
    }

    fn empty_score() -> Score {
        Score::default()
    }

    fn empty_arsenal() -> Arsenal {
        Arsenal::new()
    }

    #[test]
    fn weapon_kind_round_trip() {
        for i in 0..WEAPON_COUNT {
            let kind = WeaponKind::from_index(i).unwrap();
            assert_eq!(kind.index(), i);
        }
    }

    #[test]
    fn weapons_array_all_match_index() {
        for (i, def) in WEAPONS.iter().enumerate() {
            assert_eq!(def.kind.index(), i, "WEAPONS[{i}].kind has wrong index");
        }
    }

    #[test]
    fn arsenal_add_and_stack() {
        let mut a = Arsenal::new();
        assert!(a.add(WeaponKind::RiseUp));
        assert!(a.add(WeaponKind::RiseUp)); // stack
        assert_eq!(a.slots[0].quantity, 2);
        assert_eq!(a.slot_count(), 1);
    }

    #[test]
    fn arsenal_full_blocks_new_kind() {
        let mut a = Arsenal::new();
        for i in 0..WEAPON_COUNT {
            if let Some(k) = WeaponKind::from_index(i) {
                if a.slots.len() < ARSENAL_SIZE {
                    a.add(k);
                }
            }
        }
        assert!(a.is_full());
        // Adding another distinct kind should fail
        if let Some(extra) = WeaponKind::from_index(33) {
            if !a.slots.iter().any(|s| s.kind == extra) {
                assert!(!a.can_add(extra));
            }
        }
    }

    #[test]
    fn arsenal_remove_slot() {
        let mut a = Arsenal::new();
        a.add(WeaponKind::FlipOut);
        a.add(WeaponKind::FlipOut);
        let removed = a.remove_slot(0);
        assert_eq!(removed, Some(WeaponKind::FlipOut));
        assert_eq!(a.slots[0].quantity, 1);
        let removed2 = a.remove_slot(0);
        assert_eq!(removed2, Some(WeaponKind::FlipOut));
        assert!(a.slots.is_empty()); // slot removed when quantity 0
    }

    #[test]
    fn weapon_state_activate_sets_remaining() {
        let mut ws = WeaponState::new();
        let mut board = empty_board();
        ws.activate(WeaponKind::Speedy, &mut board);
        assert!(ws.is_active(WeaponKind::Speedy));
        assert_eq!(ws.remaining[WeaponKind::Speedy.index()], 10);
    }

    #[test]
    fn weapon_state_mondale_stacks() {
        let mut ws = WeaponState::new();
        let mut board = empty_board();
        ws.activate(WeaponKind::Mondale, &mut board);
        ws.activate(WeaponKind::Mondale, &mut board);
        assert_eq!(ws.mondale_stacks, 2);
        assert_eq!(ws.mondale_rate(), 51);
        // Duration stacks too
        assert_eq!(ws.remaining[WeaponKind::Mondale.index()], 100);
    }

    #[test]
    fn weapon_state_tick_lines_expires() {
        let mut ws = WeaponState::new();
        let mut board = empty_board();
        ws.activate(WeaponKind::FW, &mut board); // duration 3
        let expired = ws.tick_lines(3, &mut board);
        assert!(expired.contains(&WeaponKind::FW));
        assert!(!ws.is_active(WeaponKind::FW));
    }

    #[test]
    fn tick_lines_decrements_partial() {
        let mut ws = WeaponState::new();
        let mut board = empty_board();
        ws.activate(WeaponKind::Slick, &mut board); // duration 3
        let expired = ws.tick_lines(2, &mut board);
        assert!(expired.is_empty());
        assert_eq!(ws.remaining[WeaponKind::Slick.index()], 1);
    }

    #[test]
    fn bottle_weapon_fills_walls_on_activate() {
        let mut ws = WeaponState::new();
        let mut board = empty_board();
        ws.activate(WeaponKind::Bottle, &mut board);
        assert_eq!(board.cell(0, 7), Cell::Struct_);
        assert_eq!(board.cell(9, 20), Cell::Struct_);
        // Expire Bottle
        ws.tick_lines(10, &mut board);
        assert!(board.cell(0, 7).is_empty());
    }

    #[test]
    fn mirror_nullifies_keating() {
        let mut target_ws = WeaponState::new();
        let mut board = empty_board();
        target_ws.activate(WeaponKind::Mirror, &mut board);
        assert_eq!(check_mirror(WeaponKind::Keating, &target_ws), MirrorResult::Nullified);
    }

    #[test]
    fn mirror_reflects_riseup() {
        let mut target_ws = WeaponState::new();
        let mut board = empty_board();
        target_ws.activate(WeaponKind::Mirror, &mut board);
        assert_eq!(check_mirror(WeaponKind::RiseUp, &target_ws), MirrorResult::Reflected);
    }

    #[test]
    fn mirror_passthrough_when_inactive() {
        let target_ws = WeaponState::new();
        assert_eq!(check_mirror(WeaponKind::RiseUp, &target_ws), MirrorResult::PassThrough);
    }

    #[test]
    fn keating_steals_funds() {
        let mut board = empty_board();
        let mut ws = WeaponState::new();
        let mut score = empty_score();
        let mut arsenal = empty_arsenal();
        let mut next = PieceKind::El;
        let mut rng = test_rng();

        score.funds = 500;
        let outcome = apply_weapon_instant(WeaponKind::Keating, &mut board, &mut ws, &mut score, &mut arsenal, &mut next, &mut rng);
        assert_eq!(score.funds, 0);
        assert_eq!(outcome.funds_stolen, 500);
    }

    #[test]
    fn reagan_negates_funds() {
        let mut board = empty_board();
        let mut ws = WeaponState::new();
        let mut score = empty_score();
        let mut arsenal = empty_arsenal();
        let mut next = PieceKind::El;
        let mut rng = test_rng();

        score.funds = 300;
        apply_weapon_instant(WeaponKind::Reagan, &mut board, &mut ws, &mut score, &mut arsenal, &mut next, &mut rng);
        assert_eq!(score.funds, -300);
    }

    #[test]
    fn mondale_tax_rate_single_stack() {
        let mut ws = WeaponState::new();
        let mut board = empty_board();
        ws.activate(WeaponKind::Mondale, &mut board);
        assert_eq!(ws.mondale_rate(), 30);
    }

    #[test]
    fn bazaar_state_sorted_by_price() {
        let baz = BazaarState::new();
        let prices: Vec<u32> = baz.weapons.iter().map(|&k| weapon_def(k).price).collect();
        for i in 1..prices.len() {
            assert!(prices[i] >= prices[i-1], "Bazaar not sorted at index {i}");
        }
    }

    #[test]
    fn bazaar_try_buy_deducts_funds() {
        let baz = BazaarState::new();
        let mut score = empty_score();
        let mut arsenal = empty_arsenal();
        score.funds = 10000;
        let bought = baz.try_buy(&mut score, &mut arsenal, false);
        assert!(bought);
        assert!(score.funds < 10000);
        assert_eq!(arsenal.slot_count(), 1);
    }

    #[test]
    fn bazaar_try_buy_fails_if_insufficient_funds() {
        let baz = BazaarState::new();
        let mut score = empty_score();
        let mut arsenal = empty_arsenal();
        score.funds = 0;
        let bought = baz.try_buy(&mut score, &mut arsenal, false);
        assert!(!bought);
    }
}
