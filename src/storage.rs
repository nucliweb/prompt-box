#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    todo!("implement in T2")
}

pub fn save_prompts(_prompts: &[Prompt]) -> Result<()> {
    todo!("implement in T2")
}
