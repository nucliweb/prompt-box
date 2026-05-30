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

#[test]
fn no_args_enters_tui_placeholder() {
    Command::cargo_bin("pbox")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains("TUI mode"));
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
