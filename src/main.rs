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
use commands::{add, archive, dir, edit, index, list, show, tags, template};
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
    Add {
        /// Template name (e.g. 1on1, idea, todo, meeting)
        #[arg(long)]
        template: Option<String>,
    },
    /// Edit an existing memo by ID
    Edit { id: String },
    /// Show memo content by ID
    Show { id: String },
    /// List all memos
    List {
        /// Output in JSONL format
        #[arg(long)]
        json: bool,
        /// Filter by tag (e.g. @1on1)
        #[arg(long)]
        tag: Option<String>,
    },
    /// Show memo directory path
    Dir,
    /// Archive memos by ID, file path, or directory
    Archive { targets: Vec<String> },
    /// Build search index
    Index,
    /// Search memos
    Search { query: String },
    /// List all tags with counts
    Tags,
    /// Manage templates
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// Add a new template
    Add { name: String },
    /// Edit a template (creates from builtin if not exists)
    Edit { name: String },
    /// List available templates
    List,
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
        Commands::Add { template } => add::run(&memo_context, template.as_deref()),
        Commands::Edit { id } => edit::run(&memo_context, &id),
        Commands::Show { id } => show::run(&memo_context, &id),
        Commands::List { json, tag } => list::run(&memo_context, json, tag.as_deref()),
        Commands::Dir => dir::run(&memo_context),
        Commands::Archive { targets } => archive::run(&memo_context, &targets),
        Commands::Index => index::run_index(&memo_context),
        Commands::Search { query } => search_cmd::run_search(&memo_context, &query),
        Commands::Tags => tags::run(&memo_context),
        Commands::Template { command } => match command {
            TemplateCommands::Add { name } => template::run_add(&memo_context, &name),
            TemplateCommands::Edit { name } => template::run_edit(&memo_context, &name),
            TemplateCommands::List => template::run_list(&memo_context),
        },
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
