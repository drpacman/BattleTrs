mod db;
mod elo;
mod server;
mod session;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::{Parser, Subcommand};

use db::PlayerDb;

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
        /// TCP port to listen on.
        #[arg(short, long, default_value_t = 7001)]
        port: u16,
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
        Command::Serve { port } => {
            let db = Arc::new(Mutex::new(PlayerDb::load(&cli.db)));
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(server::run_server(port, db));
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
