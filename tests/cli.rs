use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_prompts(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("prompts.json");
    fs::write(
        &path,
        r#"[
  {
    "id": "webperf",
    "title": "Web Performance Assistant",
    "category": "performance",
    "description": "Helps optimize Core Web Vitals",
    "prompt": "Act as a web performance expert.",
    "tags": ["performance", "frontend"],
    "created_at": "2026-05-30T20:00:00Z",
    "updated_at": "2026-05-30T20:00:00Z"
  }
]"#,
    )
    .unwrap();
    path
}

fn setup_multiple_prompts(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("prompts.json");
    fs::write(
        &path,
        r#"[
  {
    "id": "webperf",
    "title": "Web Performance Assistant",
    "category": "performance",
    "description": "Helps optimize Core Web Vitals",
    "prompt": "Act as a web performance expert.",
    "tags": ["performance", "frontend"],
    "created_at": "2026-05-30T20:00:00Z",
    "updated_at": "2026-05-30T20:00:00Z"
  },
  {
    "id": "css-grid",
    "title": "CSS Grid Helper",
    "category": "css",
    "description": "Helps with CSS Grid layouts",
    "prompt": "Act as a CSS expert.",
    "tags": ["css", "layout"],
    "created_at": "2026-05-30T20:00:00Z",
    "updated_at": "2026-05-30T20:00:00Z"
  },
  {
    "id": "rust-cli",
    "title": "Rust CLI Boilerplate",
    "category": "rust",
    "description": "Boilerplate for Rust CLI apps",
    "prompt": "Act as a Rust expert.",
    "tags": ["rust", "cli"],
    "created_at": "2026-05-30T20:00:00Z",
    "updated_at": "2026-05-30T20:00:00Z"
  }
]"#,
    )
    .unwrap();
    path
}

// ── T3: CLI skeleton ─────────────────────────────────────────────────────────

#[test]
fn help_shows_all_subcommands() {
    Command::cargo_bin("pbox")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("edit"))
        .stdout(predicate::str::contains("remove"));
}

#[test]
fn get_help_shows_id_arg_and_copy_flag() {
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--copy"));
}

#[test]
fn search_help_shows_query_and_json_flag() {
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
}

// Real TUI requires a TTY and a running terminal — can't test as subprocess.
// Verify manually: run `pbox` with no args, press q to exit.
#[test]
#[ignore]
fn no_args_launches_tui() {
    Command::cargo_bin("pbox").unwrap().assert().success();
}

// ── T5: add + remove ─────────────────────────────────────────────────────────

#[test]
fn add_creates_prompt_readable_via_list() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("prompts.json");
    Command::cargo_bin("pbox")
        .unwrap()
        .args([
            "add", "my-prompt",
            "--title", "My Prompt",
            "--category", "test",
            "--tags", "a,b",
        ])
        .env("PBOX_CONFIG_FILE", &config)
        .write_stdin("This is my prompt text.")
        .assert()
        .success();
    Command::cargo_bin("pbox")
        .unwrap()
        .arg("list")
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stdout(predicate::str::contains("[test] my-prompt"));
}

#[test]
fn add_prompt_text_roundtrips_via_get() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("prompts.json");
    Command::cargo_bin("pbox")
        .unwrap()
        .args([
            "add", "rt-prompt",
            "--title", "RT",
            "--category", "test",
            "--tags", "",
        ])
        .env("PBOX_CONFIG_FILE", &config)
        .write_stdin("roundtrip text")
        .assert()
        .success();
    let output = Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "rt-prompt"])
        .env("PBOX_CONFIG_FILE", &config)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "roundtrip text");
}

#[test]
fn add_duplicate_id_exits_1() {
    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["add", "webperf", "--title", "Dup", "--category", "test", "--tags", ""])
        .env("PBOX_CONFIG_FILE", &config)
        .write_stdin("duplicate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("webperf"));
}

#[test]
fn remove_deletes_prompt() {
    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["remove", "webperf"])
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));
    Command::cargo_bin("pbox")
        .unwrap()
        .arg("list")
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stdout(predicate::str::contains("No prompts found"));
}

#[test]
fn remove_nonexistent_exits_1() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("empty.json");
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["remove", "nonexistent"])
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

// ── T4: list + get ───────────────────────────────────────────────────────────

#[test]
fn list_shows_category_id_and_description() {
    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .arg("list")
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stdout(predicate::str::contains("[performance] webperf"))
        .stdout(predicate::str::contains("Helps optimize Core Web Vitals"));
}

#[test]
fn list_empty_store_prints_no_prompts_found() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("empty.json");
    Command::cargo_bin("pbox")
        .unwrap()
        .arg("list")
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stdout(predicate::str::contains("No prompts found"));
}

#[test]
fn get_outputs_exact_prompt_text() {
    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    let output = Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "webperf"])
        .env("PBOX_CONFIG_FILE", &config)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "Act as a web performance expert.");
}

#[test]
fn get_nonexistent_exits_1_with_stderr_message() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("empty.json");
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "nonexistent"])
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

// ── T6: search ───────────────────────────────────────────────────────────────

#[test]
fn search_returns_matching_prompts() {
    let dir = TempDir::new().unwrap();
    let config = setup_multiple_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["search", "perf"])
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stdout(predicate::str::contains("webperf"))
        .stdout(predicate::str::contains("Web Performance Assistant"));
}

#[test]
fn search_ranks_best_match_first() {
    let dir = TempDir::new().unwrap();
    let config = setup_multiple_prompts(&dir);
    let output = Command::cargo_bin("pbox")
        .unwrap()
        .args(["search", "rust"])
        .env("PBOX_CONFIG_FILE", &config)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rust-cli"), "expected rust-cli in output");
    // rust-cli must rank before any other result
    let rust_pos = stdout.find("rust-cli").unwrap();
    let css_pos = stdout.find("css-grid").unwrap_or(usize::MAX);
    assert!(rust_pos < css_pos, "rust-cli should rank before css-grid");
}

#[test]
fn search_json_outputs_valid_json_array() {
    let dir = TempDir::new().unwrap();
    let config = setup_multiple_prompts(&dir);
    let output = Command::cargo_bin("pbox")
        .unwrap()
        .args(["search", "perf", "--json"])
        .env("PBOX_CONFIG_FILE", &config)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("output should be valid JSON");
    assert!(parsed.is_array());
    assert!(!parsed.as_array().unwrap().is_empty());
}

#[test]
fn search_no_matches_prints_message() {
    let dir = TempDir::new().unwrap();
    let config = setup_multiple_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["search", "xyzzy_no_match_ever"])
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn search_json_no_matches_outputs_empty_array() {
    let dir = TempDir::new().unwrap();
    let config = setup_multiple_prompts(&dir);
    let output = Command::cargo_bin("pbox")
        .unwrap()
        .args(["search", "xyzzy_no_match_ever", "--json"])
        .env("PBOX_CONFIG_FILE", &config)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("output should be valid JSON");
    assert_eq!(parsed, serde_json::json!([]));
}

// ── T7: get --copy ────────────────────────────────────────────────────────────

#[test]
fn get_without_copy_still_outputs_to_stdout() {
    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "webperf"])
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stdout(predicate::str::contains("Act as a web performance expert."));
}

#[test]
fn get_copy_produces_no_stdout() {
    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    let output = Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "webperf", "--copy"])
        .env("PBOX_CONFIG_FILE", &config)
        .output()
        .unwrap();
    // Whether clipboard succeeds or fails, nothing must go to stdout
    assert!(
        output.stdout.is_empty(),
        "stdout should be empty when --copy is used, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

// Clipboard write requires a running pasteboard server (interactive session).
// Run manually: pbox get <id> --copy && pbpaste
#[test]
#[ignore]
fn get_copy_prints_confirmation_to_stderr() {
    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "webperf", "--copy"])
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .success()
        .stderr(predicate::str::contains("Copied"));
}

// ── T12: $EDITOR integration ──────────────────────────────────────────────────

#[test]
fn edit_nonexistent_exits_1_with_error() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("empty.json");
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["edit", "nonexistent"])
        .env("PBOX_CONFIG_FILE", &config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

#[test]
fn edit_with_editor_true_aborts() {
    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["edit", "webperf"])
        .env("PBOX_CONFIG_FILE", &config)
        .env("EDITOR", "true")
        .assert()
        .success()
        .stdout(predicate::str::contains("Aborted."));
}

#[test]
fn edit_with_script_saves_changes() {
    let script_dir = TempDir::new().unwrap();
    let script_path = script_dir.path().join("fake_editor.sh");
    fs::write(&script_path, "#!/bin/sh\nprintf 'Updated prompt text' > \"$1\"\n").unwrap();
    std::process::Command::new("chmod")
        .args(["+x", script_path.to_str().unwrap()])
        .status()
        .unwrap();

    let dir = TempDir::new().unwrap();
    let config = setup_prompts(&dir);
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["edit", "webperf"])
        .env("PBOX_CONFIG_FILE", &config)
        .env("EDITOR", script_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated: webperf"));

    let output = Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "webperf"])
        .env("PBOX_CONFIG_FILE", &config)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "Updated prompt text"
    );
}

#[test]
fn add_with_force_editor_aborts_on_empty() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("prompts.json");
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["add", "test-editor", "--title", "T", "--category", "c"])
        .env("PBOX_CONFIG_FILE", &config)
        .env("PBOX_FORCE_EDITOR", "1")
        .env("EDITOR", "true")
        .assert()
        .success()
        .stdout(predicate::str::contains("Aborted."));
}

#[test]
fn add_with_force_editor_saves_prompt() {
    let script_dir = TempDir::new().unwrap();
    let script_path = script_dir.path().join("fake_editor.sh");
    fs::write(&script_path, "#!/bin/sh\nprintf 'My new prompt' > \"$1\"\n").unwrap();
    std::process::Command::new("chmod")
        .args(["+x", script_path.to_str().unwrap()])
        .status()
        .unwrap();

    let dir = TempDir::new().unwrap();
    let config = dir.path().join("prompts.json");
    Command::cargo_bin("pbox")
        .unwrap()
        .args(["add", "test-editor", "--title", "T", "--category", "c"])
        .env("PBOX_CONFIG_FILE", &config)
        .env("PBOX_FORCE_EDITOR", "1")
        .env("EDITOR", script_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Added: test-editor"));

    let output = Command::cargo_bin("pbox")
        .unwrap()
        .args(["get", "test-editor"])
        .env("PBOX_CONFIG_FILE", &config)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "My new prompt"
    );
}
