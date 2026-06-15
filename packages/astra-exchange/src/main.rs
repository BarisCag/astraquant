use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Process live event streams
    Run,
    /// Drive the FullReplayEngine on an offline journal directory
    Replay {
        #[arg(short, long)]
        journal_dir: String,
    },
    /// Execute a rapid offline replay and output metrics
    Benchmark {
        #[arg(short, long)]
        journal_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run => {
            println!("Live running is not fully implemented in the orchestration layer yet.");
        }
        Commands::Replay { journal_dir } => {
            use astra_exchange::replay::FullReplayEngine;
            use astra_exchange::runtime::ExchangeRuntime;
            use astra_risk::engine::RiskEngine;

            println!("Starting Replay from: {}", journal_dir);
            let runtime = ExchangeRuntime::new(RiskEngine::new());
            let mut engine = FullReplayEngine::new(runtime);
            match engine.replay_directory(std::path::Path::new(journal_dir)) {
                Ok(hash) => println!("Replay successful. Global Hash: {:?}", hash),
                Err(e) => eprintln!("Replay failed: {}", e),
            }
        }
        Commands::Benchmark { journal_dir: _ } => {
            println!("Benchmarking mode is stubbed.");
        }
    }
}
