# prompt-box (pbox) — Task List

## Phase 1: Core CLI & Storage

- [x] T1: Project scaffold (Cargo.toml + module stubs, `cargo build` clean)
- [x] T2: Storage layer (`Prompt` struct, JSON r/w, config dir, unit tests)
- [x] T3: CLI skeleton + mode dispatch (clap stubs, TUI placeholder)
- [x] T4: `pbox list` + `pbox get` (stdout, pipeable)
- [x] T5: `pbox add` + `pbox remove`
- [x] T6: `pbox search` (fuzzy ranking, `--json` flag)
- [x] T7: `pbox get --copy` (arboard clipboard)

**[ ] CHECKPOINT 1** — cargo test passes, all CLI commands work end-to-end

## Phase 2: TUI

- [x] T8: TUI foundation (ratatui + crossterm loop, `q`/`Esc` exits cleanly)
- [ ] T9: TUI search input + prompt list (real-time fuzzy filter, arrow nav)
- [ ] T10: TUI split preview pane (50/50 layout, metadata + prompt text, status bar)
- [ ] T11: TUI copy on Enter + clean exit (arboard, print "Copied: <title>")

**[ ] CHECKPOINT 2** — full TUI flow works, terminal always restored cleanly

## Phase 3: Polish

- [ ] T12: `$EDITOR` integration for `pbox add` / `pbox edit`
- [ ] T13: Shell autocompletion (`pbox completions bash|zsh|fish`)

**[ ] CHECKPOINT 3** — release build clean, completions work in zsh
