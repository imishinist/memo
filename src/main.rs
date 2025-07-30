use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod context;
mod error;
mod frontmatter;
mod memo;
mod repository;
mod utils;

use commands::{add, archive, dir, edit, list};
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

    // コンテキストを初期化
    let context = match MemoContext::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // メモディレクトリを確保
    if let Err(e) = context.ensure_memo_dir() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    let result = match cli.command {
        Commands::Add => add::run(&context),
        Commands::Edit { id } => edit::run(&context, &id),
        Commands::List { json } => list::run(&context, json),
        Commands::Dir => dir::run(&context),
        Commands::Archive { targets } => archive::run(&context, &targets),
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
