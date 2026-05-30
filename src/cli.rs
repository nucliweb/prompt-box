use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::io::Read;

use crate::storage::{self, Prompt};

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
    /// Add a new prompt (reads prompt text from stdin)
    Add {
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        category: String,
        #[arg(long, default_value = "")]
        tags: String,
        #[arg(long, default_value = "")]
        description: String,
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
        Commands::Add { id, title, category, tags, description } => {
            cmd_add(&id, &title, &category, &tags, &description)
        }
        Commands::Remove { id } => cmd_remove(&id),
        Commands::Search { .. } => {
            println!("not implemented");
            Ok(())
        }
        Commands::Edit { .. } => {
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

fn cmd_add(id: &str, title: &str, category: &str, tags: &str, description: &str) -> Result<()> {
    let mut prompts = storage::load_prompts()?;

    if storage::find_by_id(&prompts, id).is_some() {
        return Err(anyhow!("prompt '{}' already exists", id));
    }

    let mut prompt_text = String::new();
    std::io::stdin().read_to_string(&mut prompt_text)?;
    let prompt_text = prompt_text.trim().to_string();

    let tags: Vec<String> = if tags.is_empty() {
        vec![]
    } else {
        tags.split(',').map(|t| t.trim().to_string()).collect()
    };

    let now = Utc::now().to_rfc3339();
    prompts.push(Prompt {
        id: id.to_string(),
        title: title.to_string(),
        category: category.to_string(),
        description: description.to_string(),
        prompt: prompt_text,
        tags,
        created_at: now.clone(),
        updated_at: now,
    });

    storage::save_prompts(&prompts)?;
    println!("Added: {}", id);
    Ok(())
}

fn cmd_remove(id: &str) -> Result<()> {
    let mut prompts = storage::load_prompts()?;
    let len_before = prompts.len();
    prompts.retain(|p| p.id != id);

    if prompts.len() == len_before {
        return Err(anyhow!("prompt '{}' not found", id));
    }

    storage::save_prompts(&prompts)?;
    println!("Removed: {}", id);
    Ok(())
}
