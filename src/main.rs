use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod context;
mod display;
mod error;
mod front_matter;
mod memo;
mod memo_id;
mod repository;
mod search;
mod utils;

use commands::search as search_cmd;
use commands::{add, archive, dir, edit, index, list, show};
use context::MemoContext;
use error::MemoError;

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
    /// Show memo content by ID
    Show { id: String },
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
    /// Build search index
    Index,
    /// Search memos
    Search { query: String },
}

fn main() {
    let cli = Cli::parse();

    // コンテキストを初期化
    let memo_context = match MemoContext::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // メモディレクトリを確保
    if let Err(e) = memo_context.ensure_memo_dir() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    let result = match cli.command {
        Commands::Add => add::run(&memo_context),
        Commands::Edit { id } => edit::run(&memo_context, &id),
        Commands::Show { id } => show::run(&memo_context, &id),
        Commands::List { json } => list::run(&memo_context, json),
        Commands::Dir => dir::run(&memo_context),
        Commands::Archive { targets } => archive::run(&memo_context, &targets),
        Commands::Index => index::run_index(&memo_context),
        Commands::Search { query } => search_cmd::run_search(&memo_context, &query),
    };

    if let Err(e) = result {
        match e {
            MemoError::MemoNotFound(_) | MemoError::InvalidId(_) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            _ => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}
