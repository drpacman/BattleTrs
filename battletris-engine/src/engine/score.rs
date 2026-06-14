use serde::{Deserialize, Serialize};

use crate::engine::board::BOARD_ROWS;

/// Live score state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Score {
    pub score: u32,
    pub lines: u32,
    /// i64 to accommodate Reagan Era negating funds (BR-E01).
    pub funds: i64,
    pub op_score: u32,
    pub op_lines: u32,
    pub op_funds: i64,
    /// Combined lines cleared by both players (bazaar trigger).
    pub combined_lines: u32,
}

impl Score {
    /// Record a hard drop. Score increases by (board_height − y_at_drop_start).
    pub fn add_hard_drop(&mut self, y_at_drop_start: i32) {
        let gain = (BOARD_ROWS as i32 - y_at_drop_start).max(0) as u32;
        self.score += gain;
    }

    /// Record earned funds after a line clear, applying Mondale tax if active.
    /// Returns (kept, taxed_away) pair.
    pub fn add_funds_taxed(&mut self, raw_funds: i32, mondale_rate: u8) -> (i64, i64) {
        let kept_pct = 100u64.saturating_sub(mondale_rate as u64);
        let kept = (raw_funds as i64 * kept_pct as i64) / 100;
        let taxed = raw_funds as i64 - kept;
        self.funds += kept;
        (kept, taxed)
    }

    /// Record earned funds with no tax (used for economic weapon transfers).
    pub fn add_funds(&mut self, funds: i64) {
        self.funds += funds;
    }

    /// Record cleared lines (player side). Returns true if bazaar should trigger.
    pub fn add_lines(&mut self, count: u32) -> bool {
        let prev_combined = self.combined_lines;
        self.lines += count;
        self.combined_lines += count;
        let prev_bucket = prev_combined / 20;
        let new_bucket = self.combined_lines / 20;
        new_bucket > prev_bucket
    }

    /// Record opponent line clear for combined bazaar counter.
    pub fn add_op_lines(&mut self, count: u32) -> bool {
        let prev_combined = self.combined_lines;
        self.op_lines += count;
        self.combined_lines += count;
        let prev_bucket = prev_combined / 20;
        let new_bucket = self.combined_lines / 20;
        new_bucket > prev_bucket
    }

    /// Update opponent stats from their score update.
    pub fn update_opponent(&mut self, op_score: u32, op_lines: u32, op_funds: i64) {
        self.op_score = op_score;
        self.op_lines = op_lines;
        self.op_funds = op_funds;
    }

    /// How many combined lines until the next bazaar.
    pub fn lines_until_bazaar(&self) -> u32 {
        20 - (self.combined_lines % 20)
    }

    /// Build a lightweight view for the renderer.
    pub fn view(&self) -> ScoreView {
        ScoreView {
            score: self.score,
            lines: self.lines,
            funds: self.funds,
            op_score: self.op_score,
            op_lines: self.op_lines,
            op_funds: self.op_funds,
            lines_until_bazaar: self.lines_until_bazaar(),
            show_op_funds: false,
        }
    }
}

/// Renderer-facing snapshot.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScoreView {
    pub score: u32,
    pub lines: u32,
    pub funds: i64,
    pub op_score: u32,
    pub op_lines: u32,
    pub op_funds: i64,
    pub lines_until_bazaar: u32,
    /// True when Ames/Ace/Condor is active and opponent funds should be shown.
    pub show_op_funds: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_funds_taxed_30pct() {
        let mut s = Score::default();
        let (kept, taxed) = s.add_funds_taxed(100, 30);
        assert_eq!(kept, 70);
        assert_eq!(taxed, 30);
        assert_eq!(s.funds, 70);
    }

    #[test]
    fn add_funds_taxed_51pct() {
        let mut s = Score::default();
        let (kept, taxed) = s.add_funds_taxed(100, 51);
        assert_eq!(kept, 49);
        assert_eq!(taxed, 51);
    }

    #[test]
    fn funds_can_be_negative() {
        let mut s = Score::default();
        s.funds = 300;
        s.add_funds(-600);
        assert!(s.funds < 0);
    }

    #[test]
    fn bazaar_triggers_at_20_combined() {
        let mut s = Score::default();
        s.combined_lines = 19;
        assert!(s.add_lines(1));
        assert!(!s.add_lines(1)); // 21 — no new bucket
    }

    #[test]
    fn op_lines_count_toward_bazaar() {
        let mut s = Score::default();
        s.combined_lines = 15;
        s.add_lines(3); // → 18, no trigger
        let trigger = s.add_op_lines(2); // → 20, trigger
        assert!(trigger);
    }
}
