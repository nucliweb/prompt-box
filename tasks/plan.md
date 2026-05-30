# Implementation Plan: prompt-box (pbox)

## Overview

`prompt-box` is a dual-mode CLI/TUI prompt manager written in Rust. It stores developer prompts as JSON and exposes them via an interactive terminal UI (TUI mode, no arguments) or a non-interactive CLI (CLI mode, with arguments). The CLI output pipes directly to AI agents. The TUI uses ratatui for rendering and fuzzy search.

## Architecture Decisions

- **Binary crate** with modules: `storage`, `cli`, `tui`. No lib/bin split needed at this scale.
- **Storage first**: All commands depend on the JSON read/write layer — build and test it in isolation before wiring commands.
- **Clipboard via `arboard`**: Shared between CLI (`get --copy`) and TUI (Enter key) — extract to a tiny helper so both modes call the same function.
- **Fuzzy matching via `fuzzy-matcher`**: Used in both CLI `search` and TUI live filter — same scorer, different rendering.
- **TUI mode detection**: `if args is empty → TUI, else → CLI`. Handled in `main.rs` before clap parses.
- **Error handling**: `anyhow` for ergonomic error propagation throughout. No custom error types needed at MVP scale.

## Dependency Graph

```
Task 1: Project scaffold (Cargo.toml, modules, anyhow)
    │
    Task 2: Storage layer (Prompt struct, JSON r/w, config dir)
        │
        ├── Task 3: CLI skeleton (clap commands, mode dispatch)
        │       │
        │       ├── Task 4: pbox list + pbox get (stdout)
        │       ├── Task 5: pbox add + pbox remove
        │       ├── Task 6: pbox search (fuzzy, JSON flag)
        │       └── Task 7: pbox get --copy (arboard)
        │
        └── Task 8: TUI foundation (ratatui + crossterm event loop)
                │
                Task 9: TUI search input + prompt list (fuzzy filter)
                    │
                    Task 10: TUI split preview pane
                        │
                        Task 11: TUI copy on Enter + clean exit

Phase 3 (polish — after Phase 2 checkpoint):
    Task 12: $EDITOR integration for pbox add / pbox edit
    Task 13: Shell autocompletion generation (clap_complete)
```

---

## Phase 1: Core CLI & Storage

### Task 1: Project scaffold

**Description:** Initialize the Cargo binary crate, declare all dependencies in `Cargo.toml`, and create the module skeleton (`main.rs`, `storage.rs`, `cli.rs`, `tui.rs`). No logic yet — just compiling stubs.

**Acceptance criteria:**
- [ ] `cargo build` succeeds with zero warnings
- [ ] `Cargo.toml` declares: `clap`, `serde`+`serde_json`, `directories`, `arboard`, `fuzzy-matcher`, `ratatui`, `crossterm`, `anyhow`
- [ ] `src/main.rs` has `mod storage; mod cli; mod tui;` and compiles

**Verification:**
- [ ] `cargo build` exits 0
- [ ] `cargo check` exits 0

**Dependencies:** None

**Files touched:**
- `Cargo.toml`
- `src/main.rs`
- `src/storage.rs`
- `src/cli.rs`
- `src/tui.rs`

**Estimated scope:** S

---

### Task 2: Storage layer

**Description:** Implement the `storage` module — the `Prompt` struct with serde derives, and functions to resolve the config path (`~/.config/prompt-box/prompts.json`), read all prompts, write all prompts, and find a prompt by id.

**Acceptance criteria:**
- [ ] `Prompt` struct matches schema: `id`, `title`, `category`, `description`, `prompt`, `tags: Vec<String>`, `created_at`, `updated_at`
- [ ] Config path resolves via `directories::ProjectDirs`
- [ ] `load_prompts()` returns `Vec<Prompt>` (empty vec if file missing, error on parse failure)
- [ ] `save_prompts(prompts)` creates the directory if absent and writes pretty-printed JSON
- [ ] `find_by_id(id)` returns `Option<&Prompt>`
- [ ] Unit tests: round-trip (write then read equals original), missing file returns empty vec

**Verification:**
- [ ] `cargo test storage` passes
- [ ] `cargo build` exits 0

**Dependencies:** Task 1

**Files touched:**
- `src/storage.rs`

**Estimated scope:** M

---

### Task 3: CLI skeleton + mode dispatch

**Description:** Wire up `clap` with all subcommands (`get`, `search`, `list`, `add`, `edit`, `remove`) as stubs that print "not implemented". Implement mode dispatch in `main.rs`: if no arguments → print "TUI mode (coming soon)", else → run CLI.

**Acceptance criteria:**
- [ ] `pbox --help` shows all subcommands
- [ ] `pbox get --help` shows `<id>` arg and `--copy` flag
- [ ] `pbox search --help` shows `<query>` and `--json` flag
- [ ] `pbox list` prints "not implemented"
- [ ] Running `pbox` with no args prints "TUI mode (coming soon)"

**Verification:**
- [ ] `cargo run -- --help` exits 0 and lists all subcommands
- [ ] `cargo run -- get foo` prints "not implemented"
- [ ] `cargo run` (no args) prints TUI placeholder

**Dependencies:** Task 1

**Files touched:**
- `src/main.rs`
- `src/cli.rs`

**Estimated scope:** S

---

### Task 4: `pbox list` + `pbox get` (stdout)

**Description:** Implement `pbox list` (tabular output: `[category] id — description`) and `pbox get <id>` (outputs raw prompt text to stdout). Both load from the storage layer.

**Acceptance criteria:**
- [ ] `pbox list` prints one line per prompt: `[category] id — description`
- [ ] `pbox list` on empty store prints "No prompts found."
- [ ] `pbox get <id>` prints only the `prompt` field to stdout (no trailing newline noise)
- [ ] `pbox get <nonexistent>` exits with code 1 and prints error to stderr
- [ ] Output is pipeable: `pbox get <id> | wc -c` works correctly

**Verification:**
- [ ] Seed `prompts.json` manually; `cargo run -- list` shows the entry
- [ ] `cargo run -- get <id>` output matches the `prompt` field exactly
- [ ] `cargo run -- get missing` exits 1

**Dependencies:** Tasks 2, 3

**Files touched:**
- `src/cli.rs`

**Estimated scope:** S

---

### Task 5: `pbox add` + `pbox remove`

**Description:** Implement `pbox add <id> --title --category --tags` (reads prompt text from stdin if not in args, writes to storage) and `pbox remove <id>` (removes by id, errors if not found).

**Acceptance criteria:**
- [ ] `pbox add <id> --title "T" --category "c" --tags "a,b"` prompts for prompt text via stdin and saves
- [ ] `add` sets `created_at` and `updated_at` to current UTC ISO-8601
- [ ] Duplicate `id` on `add` exits 1 with clear error message
- [ ] `pbox remove <id>` removes the prompt and confirms to stdout
- [ ] `pbox remove <nonexistent>` exits 1 with clear error

**Verification:**
- [ ] `echo "my prompt" | cargo run -- add test-id --title "Test" --category "test" --tags "a"` then `cargo run -- list` shows it
- [ ] `cargo run -- remove test-id` then `cargo run -- list` no longer shows it

**Dependencies:** Tasks 2, 3

**Files touched:**
- `src/cli.rs`

**Estimated scope:** M

---

### Task 6: `pbox search`

**Description:** Implement `pbox search <query>` using `fuzzy-matcher` to score prompts by query match against `id + title + description + tags`. Output a simple list by default; `--json` outputs full JSON array of matches.

**Acceptance criteria:**
- [ ] `pbox search <query>` returns prompts ranked by fuzzy score (best match first)
- [ ] Default output: one line per match: `[category] id — title`
- [ ] `pbox search <query> --json` outputs a valid JSON array of full prompt objects
- [ ] No matches prints "No matches found." (or empty JSON array with `--json`)

**Verification:**
- [ ] With seeded data, `cargo run -- search perf` returns performance-related prompts
- [ ] `cargo run -- search perf --json | python3 -m json.tool` validates as JSON

**Dependencies:** Tasks 2, 3

**Files touched:**
- `src/cli.rs`

**Estimated scope:** S

---

### Task 7: `pbox get --copy` (clipboard)

**Description:** Add `--copy` / `-c` flag to `pbox get`. When set, copy the prompt to the system clipboard via `arboard` instead of writing to stdout. Print a confirmation to stderr so it doesn't pollute pipelines.

**Acceptance criteria:**
- [ ] `pbox get <id> --copy` copies prompt to clipboard and prints "Copied to clipboard." to stderr
- [ ] `pbox get <id>` (without flag) still outputs to stdout unchanged
- [ ] Clipboard write failure prints error to stderr and exits 1

**Verification:**
- [ ] `cargo run -- get <id> --copy` then paste confirms correct text
- [ ] `cargo run -- get <id> --copy 2>/dev/null` produces no stdout output

**Dependencies:** Tasks 2, 3, 4

**Files touched:**
- `src/cli.rs`

**Estimated scope:** S

---

### Checkpoint: Phase 1 Complete

- [ ] `cargo test` passes (all storage unit tests)
- [ ] `cargo build --release` succeeds
- [ ] `pbox list`, `pbox get`, `pbox add`, `pbox remove`, `pbox search` all work end-to-end
- [ ] `pbox get <id> | cat` pipes correctly
- [ ] Human review before proceeding to Phase 2

---

## Phase 2: TUI Implementation

### Task 8: TUI foundation (event loop)

**Description:** Replace the "TUI coming soon" placeholder with a real `ratatui` + `crossterm` event loop. Renders a blank frame. Exits cleanly on `q` or `Esc`.

**Acceptance criteria:**
- [ ] `pbox` (no args) enters raw terminal mode and renders a blank ratatui frame
- [ ] `q` and `Esc` both exit cleanly, restoring the terminal
- [ ] No terminal corruption after exit (alternate screen is disabled cleanly)

**Verification:**
- [ ] `cargo run` launches without panic
- [ ] Press `q` → returns to shell prompt cleanly
- [ ] `echo $?` is 0 after clean exit

**Dependencies:** Task 3

**Files touched:**
- `src/tui.rs`
- `src/main.rs`

**Estimated scope:** M

---

### Task 9: TUI search input + prompt list

**Description:** Add a search input box at the top and a scrollable prompt list on the left. As you type, the list filters in real-time using `fuzzy-matcher`. Arrow keys navigate the list.

**Acceptance criteria:**
- [ ] Search box renders at top with cursor visible
- [ ] All prompts listed on load; list updates on each keystroke
- [ ] `↑`/`↓` navigate the highlighted selection
- [ ] Mouse scroll moves selection up/down
- [ ] Backspace removes last character; list re-filters
- [ ] Empty query shows all prompts

**Verification:**
- [ ] With seeded data, typing "perf" filters to matching prompts
- [ ] Arrow keys move selection highlight
- [ ] Clearing input restores full list

**Dependencies:** Tasks 2, 8

**Files touched:**
- `src/tui.rs`

**Estimated scope:** M

---

### Task 10: TUI split preview pane

**Description:** Add a right-side preview pane (50/50 split). When a prompt is selected in the list, the preview shows: title, category, tags, description, then the full prompt text with word-wrap.

**Acceptance criteria:**
- [ ] Layout is two columns: left list, right preview
- [ ] Preview updates instantly when selection changes
- [ ] Preview shows: title header, `Tags:` line, `Description:` line, separator, full prompt text
- [ ] Long prompt text wraps within the pane width
- [ ] Click on a prompt in the list selects it
- [ ] Status bar at bottom shows key hints: `[↑↓/scroll] Navigate | [Enter] Copy & Exit | [Esc/q] Quit`

**Verification:**
- [ ] Selecting different prompts updates preview without flicker
- [ ] Terminal resize reflows layout correctly

**Dependencies:** Task 9

**Files touched:**
- `src/tui.rs`

**Estimated scope:** M

---

### Task 11: TUI copy on Enter + clean exit

**Description:** Pressing `Enter` copies the selected prompt to the system clipboard via `arboard`, exits the TUI, and prints "Copied: <title>" to stdout.

**Acceptance criteria:**
- [ ] `Enter` with a selection copies the `prompt` field to clipboard
- [ ] TUI exits cleanly (terminal restored)
- [ ] Prints "Copied: <title>" to stdout after exit
- [ ] `Enter` with empty list (no selection) is a no-op
- [ ] Clipboard failure prints error and exits 1

**Verification:**
- [ ] `pbox` → type query → navigate → Enter → paste confirms correct prompt text
- [ ] Output "Copied: <title>" visible in terminal after exit

**Dependencies:** Tasks 7, 10

**Files touched:**
- `src/tui.rs`

**Estimated scope:** S

---

### Checkpoint: Phase 2 Complete

- [ ] `cargo test` still passes
- [ ] `cargo build --release` succeeds
- [ ] Full TUI flow works: launch → search → navigate → copy → exit
- [ ] CLI flow unchanged: `pbox get <id>` still pipes correctly
- [ ] Terminal always restored cleanly (test with `reset` not needed after exit)
- [ ] Human review before proceeding to Phase 3

---

## Phase 3: Polish & Distribution

### Task 12: `$EDITOR` integration for `pbox add` / `pbox edit`

**Description:** When `pbox add` is called without piped stdin, open `$EDITOR` (fallback: `nano`, then `vim`) with a temp file for the prompt body. `pbox edit <id>` opens the existing prompt in `$EDITOR` for full editing.

**Acceptance criteria:**
- [ ] `pbox add <id> --title "T" --category "c"` opens `$EDITOR` when stdin is a TTY
- [ ] Saving and closing the editor saves the prompt
- [ ] Aborting the editor (empty file or no change) prints "Aborted." and does not save
- [ ] `pbox edit <id>` opens a temp file pre-filled with the existing prompt text
- [ ] `pbox edit <nonexistent>` exits 1 with clear error

**Verification:**
- [ ] `EDITOR=cat pbox add test --title "T" --category "c"` (cat echoes the file, exits 0) saves a prompt
- [ ] `pbox edit <id>` opens populated temp file

**Dependencies:** Tasks 4, 5

**Files touched:**
- `src/cli.rs`

**Estimated scope:** M

---

### Task 13: Shell autocompletion

**Description:** Add a `pbox completions <shell>` subcommand using `clap_complete` to generate shell completion scripts for bash, zsh, and fish.

**Acceptance criteria:**
- [ ] `pbox completions bash` outputs a valid bash completion script to stdout
- [ ] `pbox completions zsh` outputs a valid zsh completion script to stdout
- [ ] `pbox completions fish` outputs a valid fish completion script to stdout

**Verification:**
- [ ] `pbox completions zsh > /tmp/pbox.zsh` then `source /tmp/pbox.zsh` then `pbox <TAB>` shows completions

**Dependencies:** Task 3

**Files touched:**
- `Cargo.toml` (add `clap_complete`)
- `src/cli.rs`

**Estimated scope:** S

---

### Checkpoint: Phase 3 Complete

- [ ] `cargo test` passes
- [ ] `cargo build --release` succeeds with no warnings
- [ ] All CLI commands work end-to-end including editor integration
- [ ] Shell completions work in at least zsh
- [ ] Binary size is reasonable (`strip` target binary if needed)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `arboard` clipboard fails on macOS/Linux headless | Med | Wrap in clear error; document that clipboard requires a display server |
| `ratatui` terminal corruption on panic | High | Use `color_eyre` or a panic hook to restore terminal before exit |
| `crossterm` raw mode left open on crash | High | Implement `Drop` on TUI struct or use RAII guard to restore terminal |
| Config dir doesn't exist on first run | Low | `create_dir_all` in `save_prompts` handles it |
| Large `prompts.json` slows fuzzy search | Low | All in-memory; acceptable for personal use scale |

## Decisions Made

- **`pbox add` flags**: Keep simple (id + title + category + tags only). No `--prompt` flag. Future non-interactive scripting use → GitHub Issue with `enhancement` label for community vote.
- **TUI mouse support**: Yes — scroll list and click to select. Included in Task 9/10 scope.
- **Fuzzy score visibility**: Never shown in output. Score is internal only (used for ranking). A `--verbose` flag for scripting can be a future GitHub Issue.

## Future GitHub Issues (post-MVP)

- `enhancement`: `pbox add --prompt "text"` for fully non-interactive scripting
- `enhancement`: `pbox search --verbose` to expose fuzzy match scores
- `enhancement`: TUI mouse support for scroll and click-to-select (included in MVP per decision above)
