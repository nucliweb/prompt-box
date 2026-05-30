use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use crate::storage;

#[derive(Parser)]
#[command(name = "pbox", about = "A CLI/TUI prompt manager for developers and AI agents")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Output a prompt to stdout (or copy to clipboard with --copy)
    Get {
        id: String,
        #[arg(short, long, help = "Copy to clipboard instead of stdout")]
        copy: bool,
    },
    /// Search prompts by fuzzy query
    Search {
        query: String,
        #[arg(long, help = "Output results as JSON")]
        json: bool,
    },
    /// List all prompts
    List,
    /// Add a new prompt
    Add {
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        category: String,
        #[arg(long, default_value = "")]
        tags: String,
    },
    /// Edit an existing prompt in $EDITOR
    Edit { id: String },
    /// Remove a prompt
    Remove { id: String },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Get { id, copy: _ } => cmd_get(&id),
        Commands::List => cmd_list(),
        Commands::Search { .. } => {
            println!("not implemented");
            Ok(())
        }
        Commands::Add { .. } => {
            println!("not implemented");
            Ok(())
        }
        Commands::Edit { .. } => {
            println!("not implemented");
            Ok(())
        }
        Commands::Remove { .. } => {
            println!("not implemented");
            Ok(())
        }
    }
}

fn cmd_list() -> Result<()> {
    let prompts = storage::load_prompts()?;
    if prompts.is_empty() {
        println!("No prompts found.");
        return Ok(());
    }
    for p in &prompts {
        println!("[{}] {} — {}", p.category, p.id, p.description);
    }
    Ok(())
}

fn cmd_get(id: &str) -> Result<()> {
    let prompts = storage::load_prompts()?;
    let p = storage::find_by_id(&prompts, id)
        .ok_or_else(|| anyhow!("prompt '{}' not found", id))?;
    print!("{}", p.prompt);
    Ok(())
}
