use std::collections::HashMap;
use std::path::{Path, PathBuf};

use battletris_engine::protocol::PlayerRecord;

pub struct PlayerDb {
    players: HashMap<String, PlayerRecord>,
    path: PathBuf,
}

impl PlayerDb {
    /// Load from `path` (JSON). Returns an empty database if the file is absent.
    pub fn load(path: &Path) -> Self {
        let players = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        PlayerDb { players, path: path.to_path_buf() }
    }

    /// Persist to disk. Called after every ELO update (BR-NET-12).
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.players)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&self.path, json)
    }

    /// Return a reference to the player record, creating a default entry (ELO=1200) if absent.
    pub fn get_or_create(&mut self, name: &str) -> &PlayerRecord {
        self.players
            .entry(name.to_string())
            .or_insert_with(|| PlayerRecord::new(name))
    }

    /// Apply game result: update wins/losses/ELO for both players, then flush.
    pub fn apply_result(&mut self, winner: &str, loser: &str, deltas: (i32, i32)) {
        let (gain, loss) = deltas;
        {
            let w = self.players.entry(winner.to_string())
                .or_insert_with(|| PlayerRecord::new(winner));
            w.elo += gain;
            w.wins += 1;
        }
        {
            let l = self.players.entry(loser.to_string())
                .or_insert_with(|| PlayerRecord::new(loser));
            l.elo = (l.elo + loss).max(100);
            l.losses += 1;
        }
        let _ = self.save();
    }

    /// All player records sorted by ELO descending.
    pub fn all_sorted(&self) -> Vec<&PlayerRecord> {
        let mut v: Vec<&PlayerRecord> = self.players.values().collect();
        v.sort_by(|a, b| b.elo.cmp(&a.elo));
        v
    }

    /// Look up one player by exact name.
    pub fn get(&self, name: &str) -> Option<&PlayerRecord> {
        self.players.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("bt_db_test_{name}.json"))
    }

    #[test]
    fn get_or_create_starts_at_1200() {
        let path = tmp_path("create");
        let _ = std::fs::remove_file(&path);
        let mut db = PlayerDb::load(&path);
        let rec = db.get_or_create("Alice");
        assert_eq!(rec.elo, 1200);
        assert_eq!(rec.wins, 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn apply_result_updates_wins_and_losses() {
        let path = tmp_path("result");
        let _ = std::fs::remove_file(&path);
        let mut db = PlayerDb::load(&path);
        db.get_or_create("Alice");
        db.get_or_create("Bob");
        db.apply_result("Alice", "Bob", (16, -16));
        assert_eq!(db.get("Alice").unwrap().wins, 1);
        assert_eq!(db.get("Bob").unwrap().losses, 1);
        assert!(db.get("Alice").unwrap().elo > 1200);
        assert!(db.get("Bob").unwrap().elo < 1200);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_and_reload_round_trip() {
        let path = tmp_path("reload");
        let _ = std::fs::remove_file(&path);
        {
            let mut db = PlayerDb::load(&path);
            db.get_or_create("Alice");
            db.apply_result("Alice", "Ghost", (16, -16));
        }
        let db2 = PlayerDb::load(&path);
        assert!(db2.get("Alice").is_some());
        assert_eq!(db2.get("Alice").unwrap().wins, 1);
        let _ = std::fs::remove_file(&path);
    }
}
