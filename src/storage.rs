#![allow(dead_code)]

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub prompt: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub fn load_prompts() -> Result<Vec<Prompt>> {
    load_from(&config_path()?)
}

pub fn save_prompts(prompts: &[Prompt]) -> Result<()> {
    save_to(&config_path()?, prompts)
}

pub fn find_by_id<'a>(prompts: &'a [Prompt], id: &str) -> Option<&'a Prompt> {
    prompts.iter().find(|p| p.id == id)
}

fn config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("PBOX_CONFIG_FILE") {
        return Ok(PathBuf::from(path));
    }
    let dirs = ProjectDirs::from("", "", "prompt-box")
        .ok_or_else(|| anyhow::anyhow!("could not determine config directory"))?;
    Ok(dirs.config_dir().join("prompts.json"))
}

pub(crate) fn load_from(path: &Path) -> Result<Vec<Prompt>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))
}

pub(crate) fn save_to(path: &Path, prompts: &[Prompt]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(prompts)?;
    std::fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_prompt() -> Prompt {
        Prompt {
            id: "test-id".to_string(),
            title: "Test Prompt".to_string(),
            category: "testing".to_string(),
            description: "A test prompt".to_string(),
            prompt: "Do the test thing.".to_string(),
            tags: vec!["test".to_string(), "sample".to_string()],
            created_at: "2026-05-30T20:00:00Z".to_string(),
            updated_at: "2026-05-30T20:00:00Z".to_string(),
        }
    }

    #[test]
    fn round_trip_write_then_read() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("prompts.json");
        let prompts = vec![sample_prompt()];
        save_to(&path, &prompts).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded, prompts);
    }

    #[test]
    fn missing_file_returns_empty_vec() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        let result = load_from(&path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn find_by_id_returns_correct_prompt() {
        let prompts = vec![sample_prompt()];
        let found = find_by_id(&prompts, "test-id");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Prompt");
    }

    #[test]
    fn find_by_id_returns_none_for_missing() {
        let prompts = vec![sample_prompt()];
        let found = find_by_id(&prompts, "does-not-exist");
        assert!(found.is_none());
    }

    #[test]
    fn save_creates_parent_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("dir").join("prompts.json");
        let prompts = vec![sample_prompt()];
        save_to(&path, &prompts).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn invalid_json_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("prompts.json");
        std::fs::write(&path, "not valid json").unwrap();
        let result = load_from(&path);
        assert!(result.is_err());
    }
}
