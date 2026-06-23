mod conn;
mod db;
mod elo;
mod http_server;
mod server;
mod session;
mod ws_listener;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::{Parser, Subcommand};

use db::PlayerDb;
use server::SharedState;

const DEFAULT_DB_PATH: &str = "players.json";

#[derive(Parser)]
#[command(name = "battletris-server", about = "BattleTris relay server and admin tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to the player database JSON file.
    #[arg(long, default_value = DEFAULT_DB_PATH)]
    db: PathBuf,
}

#[derive(Subcommand)]
enum Command {
    /// Start the relay server.
    Serve {
        /// TCP port for desktop clients.
        #[arg(long, default_value_t = 7001)]
        port: u16,

        /// HTTP + WebSocket port for browser clients.
        #[arg(long, default_value_t = 80)]
        web_port: u16,

        /// Path to compiled battletris-web dist/ directory.
        #[arg(long, default_value = "./dist")]
        web_dir: PathBuf,
    },
    /// List all registered players sorted by ELO.
    Players,
    /// Show stats for a specific player.
    Show {
        /// Player name (case-sensitive).
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Serve { port, web_port, web_dir } => {
            if !web_dir.exists() {
                eprintln!(
                    "Error: web-dir '{}' does not exist. Run 'trunk build' in battletris-web/ first.",
                    web_dir.display()
                );
                std::process::exit(1);
            }

            let db = Arc::new(Mutex::new(PlayerDb::load(&cli.db)));
            let shared = Arc::new(SharedState::new(db));

            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(async {
                let tcp_shared = Arc::clone(&shared);
                let tcp_task = tokio::spawn(async move {
                    server::run_tcp_listener(port, tcp_shared).await;
                });

                let web_shared = Arc::clone(&shared);
                let web_task = tokio::spawn(async move {
                    server::run_web_server(web_port, web_dir, web_shared).await;
                });

                let (r1, r2) = tokio::join!(tcp_task, web_task);
                if let Err(e) = r1 { eprintln!("[SERVER] TCP task error: {e}"); }
                if let Err(e) = r2 { eprintln!("[SERVER] web task error: {e}"); }
            });
        }

        Command::Players => {
            let db = PlayerDb::load(&cli.db);
            let all = db.all_sorted();
            if all.is_empty() {
                println!("No players registered yet.");
            } else {
                println!("{:<20} {:>6} {:>6} {:>6}", "NAME", "ELO", "W", "L");
                println!("{}", "-".repeat(42));
                for r in all {
                    println!("{:<20} {:>6} {:>6} {:>6}", r.name, r.elo, r.wins, r.losses);
                }
            }
        }

        Command::Show { name } => {
            let db = PlayerDb::load(&cli.db);
            match db.get(&name) {
                Some(r) => {
                    println!("Player : {}", r.name);
                    println!("ELO    : {}", r.elo);
                    println!("Wins   : {}", r.wins);
                    println!("Losses : {}", r.losses);
                }
                None => println!("Player '{name}' not found."),
            }
        }
    }
}
