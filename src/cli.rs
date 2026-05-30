use anyhow::Result;
use clap::{Parser, Subcommand};

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
        Commands::Get { .. } => println!("not implemented"),
        Commands::Search { .. } => println!("not implemented"),
        Commands::List => println!("not implemented"),
        Commands::Add { .. } => println!("not implemented"),
        Commands::Edit { .. } => println!("not implemented"),
        Commands::Remove { .. } => println!("not implemented"),
    }
    Ok(())
}
