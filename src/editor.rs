use anyhow::{anyhow, Result};
use std::io::Write;

pub fn editor_cmd() -> String {
    std::env::var("EDITOR")
        .ok()
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| {
            let has_nano = std::process::Command::new("which")
                .arg("nano")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if has_nano { "nano".to_string() } else { "vim".to_string() }
        })
}

pub fn open_editor(initial: &str) -> Result<Option<String>> {
    let mut tmp = tempfile::NamedTempFile::new()?;
    write!(tmp, "{}", initial)?;
    tmp.flush()?;

    let editor = editor_cmd();
    let path = tmp.path().to_owned();

    let status = std::process::Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|e| anyhow!("failed to launch editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(anyhow!("editor '{}' exited with error", editor));
    }

    let content = std::fs::read_to_string(&path)?.trim().to_string();

    if content.is_empty() || content == initial.trim() {
        return Ok(None);
    }

    Ok(Some(content))
}
