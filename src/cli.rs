use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
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
    List {
        #[arg(long, help = "Filter by category (case-insensitive)")]
        category: Option<String>,
    },
    /// Add a new prompt (reads prompt text from stdin or $EDITOR, or use --prompt)
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
        #[arg(long, help = "Prompt body (skips stdin and editor)")]
        prompt: Option<String>,
    },
    /// Edit an existing prompt in $EDITOR
    Edit { id: String },
    /// Remove a prompt
    Remove { id: String },
    /// Generate shell completion scripts
    Completions { shell: Shell },
    /// Duplicate a prompt under a new id
    Duplicate { id: String, new_id: String },
    /// Export all prompts as JSON
    Export {
        #[arg(long, help = "Write to file instead of stdout")]
        output: Option<std::path::PathBuf>,
    },
    /// Import prompts from a JSON file (or stdin)
    Import {
        #[arg(help = "JSON file to import (reads stdin if omitted)")]
        file: Option<std::path::PathBuf>,
        #[arg(long, help = "Replace existing prompts with matching ids")]
        overwrite: bool,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Get { id, copy } => cmd_get(&id, copy),
        Commands::List { category } => cmd_list(category.as_deref()),
        Commands::Add { id, title, category, tags, description, prompt } => {
            cmd_add(&id, &title, &category, &tags, &description, prompt.as_deref())
        }
        Commands::Remove { id } => cmd_remove(&id),
        Commands::Search { query, json } => cmd_search(&query, json),
        Commands::Edit { id } => cmd_edit(&id),
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Duplicate { id, new_id } => cmd_duplicate(&id, &new_id),
        Commands::Export { output } => cmd_export(output.as_deref()),
        Commands::Import { file, overwrite } => cmd_import(file.as_deref(), overwrite),
    }
}

fn cmd_duplicate(id: &str, new_id: &str) -> Result<()> {
    let mut prompts = storage::load_prompts()?;

    if storage::find_by_id(&prompts, new_id).is_some() {
        return Err(anyhow!("prompt '{}' already exists", new_id));
    }

    let source = storage::find_by_id(&prompts, id)
        .ok_or_else(|| anyhow!("prompt '{}' not found", id))?
        .clone();

    let now = Utc::now().to_rfc3339();
    prompts.push(Prompt {
        id: new_id.to_string(),
        created_at: now.clone(),
        updated_at: now,
        ..source
    });

    storage::save_prompts(&prompts)?;
    println!("Duplicated: {} → {}", id, new_id);
    Ok(())
}

fn cmd_export(output: Option<&std::path::Path>) -> Result<()> {
    let prompts = storage::load_prompts()?;
    let json = serde_json::to_string_pretty(&prompts)?;
    match output {
        Some(path) => {
            std::fs::write(path, &json)?;
            println!("Exported: {} prompts to {}", prompts.len(), path.display());
        }
        None => print!("{}", json),
    }
    Ok(())
}

fn cmd_import(file: Option<&std::path::Path>, overwrite: bool) -> Result<()> {
    let json = match file {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| anyhow!("cannot read '{}': {}", path.display(), e))?,
        None => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            s
        }
    };

    let incoming: Vec<Prompt> =
        serde_json::from_str(&json).map_err(|e| anyhow!("invalid JSON: {}", e))?;

    let mut prompts = storage::load_prompts()?;
    let (mut added, mut updated, mut skipped) = (0usize, 0usize, 0usize);

    for p in incoming {
        match prompts.iter().position(|e| e.id == p.id) {
            Some(pos) if overwrite => { prompts[pos] = p; updated += 1; }
            Some(_) => skipped += 1,
            None => { prompts.push(p); added += 1; }
        }
    }

    storage::save_prompts(&prompts)?;

    let mut parts = vec![];
    if added > 0 { parts.push(format!("{} added", added)); }
    if updated > 0 { parts.push(format!("{} updated", updated)); }
    if skipped > 0 { parts.push(format!("{} skipped", skipped)); }
    println!("Imported: {}.", parts.join(", "));
    Ok(())
}

fn cmd_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "pbox", &mut std::io::stdout());
    Ok(())
}

fn cmd_list(category: Option<&str>) -> Result<()> {
    let prompts = storage::load_prompts()?;
    let filtered: Vec<_> = prompts
        .iter()
        .filter(|p| match category {
            Some(cat) => p.category.eq_ignore_ascii_case(cat),
            None => true,
        })
        .collect();

    if filtered.is_empty() {
        println!("No prompts found.");
        return Ok(());
    }
    for p in filtered {
        println!("[{}] {} — {}", p.category, p.id, p.title);
    }
    Ok(())
}

fn cmd_get(id: &str, copy: bool) -> Result<()> {
    let prompts = storage::load_prompts()?;
    let p = storage::find_by_id(&prompts, id)
        .ok_or_else(|| anyhow!("prompt '{}' not found", id))?;

    if copy {
        arboard::Clipboard::new()
            .and_then(|mut cb| cb.set_text(p.prompt.clone()))
            .map_err(|e| anyhow!("clipboard error: {}", e))?;
        eprintln!("Copied to clipboard: {}", p.title);
    } else {
        print!("{}", p.prompt);
    }
    Ok(())
}

fn cmd_add(id: &str, title: &str, category: &str, tags: &str, description: &str, inline: Option<&str>) -> Result<()> {
    let mut prompts = storage::load_prompts()?;

    if storage::find_by_id(&prompts, id).is_some() {
        return Err(anyhow!("prompt '{}' already exists", id));
    }

    let prompt_text = if let Some(text) = inline {
        text.trim().to_string()
    } else if should_use_editor() {
        match open_editor("")? {
            Some(text) => text,
            None => {
                println!("Aborted.");
                return Ok(());
            }
        }
    } else {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        text.trim().to_string()
    };

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

fn cmd_edit(id: &str) -> Result<()> {
    let mut prompts = storage::load_prompts()?;
    let existing = storage::find_by_id(&prompts, id)
        .ok_or_else(|| anyhow!("prompt '{}' not found", id))?;

    let initial = existing.prompt.clone();

    match open_editor(&initial)? {
        None => println!("Aborted."),
        Some(new_text) => {
            let p = prompts.iter_mut().find(|p| p.id == id).unwrap();
            p.prompt = new_text;
            p.updated_at = Utc::now().to_rfc3339();
            storage::save_prompts(&prompts)?;
            println!("Updated: {}", id);
        }
    }
    Ok(())
}

fn should_use_editor() -> bool {
    if std::env::var("PBOX_FORCE_EDITOR").is_ok() {
        return true;
    }
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

fn open_editor(initial: &str) -> Result<Option<String>> {
    crate::editor::open_editor(initial)
}

fn cmd_search(query: &str, as_json: bool) -> Result<()> {
    let prompts = storage::load_prompts()?;
    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<(i64, &Prompt)> = prompts
        .iter()
        .filter_map(|p| {
            let haystack = format!("{} {} {} {}", p.id, p.title, p.description, p.tags.join(" "));
            matcher.fuzzy_match(&haystack, query).map(|score| (score, p))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));

    if as_json {
        let matches: Vec<&Prompt> = scored.iter().map(|(_, p)| *p).collect();
        println!("{}", serde_json::to_string_pretty(&matches)?);
        return Ok(());
    }

    if scored.is_empty() {
        println!("No matches found.");
        return Ok(());
    }

    for (_, p) in &scored {
        println!("[{}] {} — {}", p.category, p.id, p.title);
    }
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
