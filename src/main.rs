use clap::{Parser, Subcommand};

mod commands;
mod frontmatter;
mod utils;

use commands::{add, archive, dir, edit, list};

#[derive(Parser)]
#[command(name = "memo")]
#[command(about = "A simple memo management tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new memo
    Add,
    /// Edit an existing memo by ID
    Edit { id: String },
    /// List all memos
    List {
        /// Output in JSONL format
        #[arg(long)]
        json: bool,
    },
    /// Show memo directory path
    Dir,
    /// Archive memos by ID, file path, or directory
    Archive { targets: Vec<String> },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add => add::run(),
        Commands::Edit { id } => edit::run(&id),
        Commands::List { json } => list::run(json),
        Commands::Dir => dir::run(),
        Commands::Archive { targets } => archive::run(&targets),
    }
}
