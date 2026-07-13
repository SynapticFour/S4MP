use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Xtask {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify workspace crate dependency tiers
    CheckArch,
}

fn main() {
    let xtask = Xtask::parse();
    match xtask.command {
        Commands::CheckArch => {
            println!("run: cargo test -p s4mp-arch-test");
        }
    }
}
