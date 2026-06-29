use std::sync::mpsc::{Receiver, SyncSender};
use std::time::{Duration, Instant};

use battletris_engine::ernie::ErnieSession;
use battletris_engine::protocol::GameMessage;

pub fn run_ernie(
    from_player: Receiver<GameMessage>,
    to_player: SyncSender<GameMessage>,
    seed: u64,
    difficulty: u8,
) {
    let mut ernie = ErnieSession::new(seed, difficulty);
    let mut last = Instant::now();

    eprintln!("[ERNIE] started, seed={seed:#x}, difficulty={difficulty}");

    loop {
        let now = Instant::now();
        let elapsed_ms = now.duration_since(last).as_millis() as u32;
        last = now;

        let mut incoming: Vec<GameMessage> = Vec::new();
        while let Ok(msg) = from_player.try_recv() {
            incoming.push(msg);
        }

        for msg in ernie.step(&incoming, elapsed_ms) {
            let _ = to_player.try_send(msg);
        }

        std::thread::sleep(Duration::from_millis(16));
    }
}
