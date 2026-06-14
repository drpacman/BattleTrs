const K: f64 = 32.0;
const ELO_FLOOR: i32 = 100;

/// Compute ELO delta for a completed game.
/// Returns `(winner_gain, loser_loss)` where winner_gain >= 0 and loser_loss <= 0.
/// Both values are clamped so neither player's rating drops below ELO_FLOOR.
pub fn compute_elo_delta(winner_elo: i32, loser_elo: i32) -> (i32, i32) {
    let expected_winner = 1.0 / (1.0 + 10_f64.powf((loser_elo - winner_elo) as f64 / 400.0));
    // ELO is zero-sum: loser's expected score = 1 - winner's expected score.
    let gain = (K * (1.0 - expected_winner)).round() as i32;
    let loss = -gain;

    // Floor: loser cannot drop below ELO_FLOOR
    let clamped_loss = loss.max(ELO_FLOOR - loser_elo);
    (gain, clamped_loss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winner_gains_loser_loses() {
        let (gain, loss) = compute_elo_delta(1200, 1200);
        assert!(gain > 0, "winner should gain rating");
        assert!(loss < 0, "loser should lose rating");
    }

    #[test]
    fn equal_players_roughly_sixteen() {
        let (gain, _) = compute_elo_delta(1200, 1200);
        assert_eq!(gain, 16);
    }

    #[test]
    fn strong_beats_weak_small_gain() {
        let (gain_strong, loss_weak) = compute_elo_delta(1600, 1000);
        let (gain_weak, _) = compute_elo_delta(1000, 1600);
        assert!(gain_strong < gain_weak, "upset should earn more ELO");
        assert!(loss_weak > -16, "loss to much stronger opponent is small");
    }

    #[test]
    fn elo_floor_clamped() {
        // Loser at exactly ELO_FLOOR should not drop further
        let (_, loss) = compute_elo_delta(1200, ELO_FLOOR);
        assert_eq!(loss, 0, "should not drop below floor");
    }
}
