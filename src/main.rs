mod cli;
mod hardware;
mod logger;
mod pironman5;
mod utils;
mod variants;
mod version;

use clap::Parser;

fn main() {
    // Parse command line arguments
    let cli = cli::Cli::parse();
    
    // Run the CLI
    if let Err(e) = cli::run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
