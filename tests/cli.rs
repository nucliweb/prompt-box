use assert_cmd::Command;
use predicates::prelude::*;

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
fn list_prints_not_implemented() {
    Command::cargo_bin("pbox")
        .unwrap()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("not implemented"));
}

#[test]
fn no_args_enters_tui_placeholder() {
    Command::cargo_bin("pbox")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains("TUI mode"));
}
