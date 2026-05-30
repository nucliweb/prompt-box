# prompt-box

A lightweight CLI/TUI prompt manager for developers and AI agents. Store, search, and retrieve prompts from any terminal.

```
$ pbox
```

![TUI screenshot placeholder](https://github.com/nucliweb/prompt-box/assets/tui-screenshot.png)

## Features

- **Interactive TUI** — fuzzy search, split preview pane, mouse support
- **Full CLI** — scriptable subcommands for every operation
- **Editor integration** — opens `$EDITOR` to write prompts (falls back to nano/vim)
- **Clipboard support** — copy any prompt directly to the system clipboard
- **Import / Export** — JSON round-trip for backups and sharing
- **Shell completions** — Bash, Zsh, Fish, PowerShell via `clap_complete`
- **Zero config** — prompts stored in the OS config directory automatically

## Installation

### From crates.io

```bash
cargo install prompt-box
```

The binary is installed as `pbox`.

### From source

```bash
git clone https://github.com/nucliweb/prompt-box
cd prompt-box
cargo install --path .
```

## Quick start

```bash
# Add a prompt (opens $EDITOR)
pbox add code-review --title "Code Review" --category dev

# Add inline without opening an editor
pbox add greet --title "Greeting" --category misc --prompt "Hello, world!"

# List all prompts
pbox list

# Get a prompt (prints to stdout)
pbox get code-review

# Copy a prompt to the clipboard
pbox get code-review --copy

# Search with fuzzy matching
pbox search "review"

# Open the interactive TUI
pbox
```

## CLI reference

### `pbox add <id>`

Adds a new prompt. The prompt body is read from:

1. `--prompt <text>` — inline, skips stdin and editor (good for scripts)
2. `$EDITOR` — when stdin is a terminal
3. stdin — when piped

```bash
pbox add <id> --title <title> --category <category> [--tags tag1,tag2] [--description <text>] [--prompt <text>]
```

### `pbox get <id>`

Prints the prompt body to stdout, or copies it to the clipboard with `--copy`.

```bash
pbox get <id> [--copy]
```

### `pbox list`

Lists all prompts. Filter by category with `--category`.

```bash
pbox list [--category <category>]
```

### `pbox search <query>`

Fuzzy-searches across id, title, description, and tags. Output as JSON with `--json`.

```bash
pbox search <query> [--json]
```

### `pbox edit <id>`

Opens the prompt body in `$EDITOR`. Saving an unchanged or empty file aborts the edit.

```bash
pbox edit <id>
```

### `pbox remove <id>`

Removes a prompt permanently.

```bash
pbox remove <id>
```

### `pbox duplicate <id> <new-id>`

Creates a copy of an existing prompt under a new id.

```bash
pbox duplicate <id> <new-id>
```

### `pbox export`

Exports all prompts as pretty-printed JSON to stdout, or to a file with `--output`.

```bash
pbox export [--output <file>]
```

### `pbox import [file]`

Imports prompts from a JSON file or stdin. Skips prompts with duplicate ids by default; use `--overwrite` to replace them.

```bash
pbox import [file] [--overwrite]
```

### `pbox completions <shell>`

Prints a shell completion script. Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

```bash
# Zsh example
pbox completions zsh > ~/.zfunc/_pbox
```

## TUI

Run `pbox` with no arguments to open the interactive interface.

| Key | Action |
|-----|--------|
| Any character | Filter prompts (fuzzy search) |
| `Backspace` | Delete last character from filter |
| `↑` / `↓` | Move selection |
| Mouse scroll | Move selection |
| Left click | Select item |
| `Enter` | Copy selected prompt to clipboard and exit |
| `Ctrl+E` | Edit selected prompt in `$EDITOR` |
| `Ctrl+D` | Delete selected prompt (asks for confirmation) |
| `q` / `Esc` | Quit |

The status bar shows how many prompts match the current filter (`filtered/total`).

## Prompt schema

Prompts are stored as JSON. Each entry has the following fields:

```json
{
  "id": "code-review",
  "title": "Code Review",
  "category": "dev",
  "description": "General-purpose code review prompt",
  "prompt": "Review this code for correctness, readability, and edge cases.",
  "tags": ["review", "quality"],
  "created_at": "2026-05-31T10:00:00Z",
  "updated_at": "2026-05-31T10:00:00Z"
}
```

## Storage location

Prompts are stored in the OS config directory:

| Platform | Path |
|----------|------|
| Linux | `$XDG_CONFIG_HOME/prompt-box/prompts.json` |
| macOS | `~/Library/Application Support/prompt-box/prompts.json` |
| Windows | `%APPDATA%\prompt-box\prompts.json` |

## Environment variables

| Variable | Description |
|----------|-------------|
| `PBOX_CONFIG_FILE` | Override the default storage path |
| `EDITOR` | Editor used by `pbox add` and `pbox edit` (falls back to nano or vim) |
| `PBOX_FORCE_EDITOR` | Always open `$EDITOR`, even when stdin is not a terminal |

## License

MIT — see [LICENSE](LICENSE).
