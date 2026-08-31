# AGENTS.md

Muse Code reads this file as project rules when it runs in this directory.

## Project

- Name: pomodoro-tui (`pomodoro-tui` v0.2.0) — terminal Pomodoro timer
- Stack: Rust 2021, ratatui 0.29 + crossterm 0.29, clap 4 (derive), tui-big-text
- Purpose: Work/Break state machine (`src/lib.rs`) + ratatui TUI (`src/app.rs`). Works well in tmux.
- Features: Work/Break auto-switch with color-coded BigText timers (blue Work / green Break), customizable durations, ASCII art per state, session name (`-n/--name` BigText yellow Sextant below timers), cross-platform Linux/macOS.
- Fork: Fork of [xamcost/pomodoro](https://github.com/xamcost/pomodoro) maintained by [iman1704](https://github.com/iman1704/pomodoro-tui).

## Common Commands

- Build: `cargo build` / `cargo build --release`
- Run: `cargo run` (defaults 25/5) | `cargo run -- -w 30 -b 10 -i` | `cargo run -- -n "Deep Work"`
- Binary: `pomodoro-tui` | `pomodoro-tui -w 30 -b 10` | `pomodoro-tui -i` | `pomodoro-tui -n "Study Session" -w 50 -b 10`
- Test: `cargo test`
- Install (Homebrew): `brew tap iman1704/tap && brew install pomodoro-tui` | `brew install iman1704/tap/pomodoro-tui`
- Release: bump `Cargo.toml` version + `git tag vX.Y.Z && git push origin vX.Y.Z` (triggers `.github/workflows/release.yml`)

## Project Layout

- `src/main.rs`: CLI parsing (clap `Args`) + terminal init/restore, delegates to `App`. Includes `parse_session_name` validator (max 25 chars), passes `work`, `break_time`, `hide_image`, `name` to `App::new`.
- `src/lib.rs`: Core `Pomodoro`/`Timer` logic, `check_and_switch()` (pure state machine, no notifications), unit tests.
- `src/app.rs`: `App` — event loop (200ms Tick), rendering, layout. Holds `session_name: Option<String>`. Layout branches on `session_name` (adds 1-row padding + 3-row Sextant BigText area); helpers `get_session_name_widget()` (yellow Sextant, centered), `truncate_with_ellipsis_for_big_text()` (cell-width `available_width / 4` for Sextant) and `truncate_with_ellipsis()`.
- `src/ascii_images.rs`: ASCII art (`computer()` for Work, `sleeping_cat()` for Break)
- `.github/workflows/ci.yml` + `.github/workflows/release.yml`: CI and tag-triggered Release workflows
- `doc/pomo_tmux.png` + `Cargo.toml` + `TODO.md`

## CLI Flags (`src/main.rs` — `pomodoro-tui --help`)

| Flag | Default | Description |
|------|---------|-------------|
| `-w, --work <mins>` | `25` | Work duration in minutes |
| `-b, --break <mins>` | `5` | Break duration in minutes |
| `-i, --hide-image` | `false` | Hide ASCII art (timer expands to 100% width) |
| `-n, --name <NAME>` | — | Session name (max 25 chars, validated by `parse_session_name`) |
| `-h, --help` | — | Print help |
| `-V, --version` | — | Print version |

Examples: `pomodoro-tui -n "Deep Work"` / `pomodoro-tui --name "Study Session" -w 50 -b 10` / `cargo run -- -w 30 -b 10 -i`

## Keybindings (`src/app.rs:handle_key_event`)

| Key | Action |
|-----|--------|
| `s` | Start / Pause current timer |
| `r` | Reset both timers to Work state |
| `q` / `Esc` | Quit |

Footer shows `Start <S> Reset <R> Quit <Q/Esc>`, title `Pomodoro`.

## Notes for Agents

- Separation: Keep timer/state logic in `lib.rs`, rendering/input in `app.rs`. Don't duplicate timer logic in `app.rs`.
- Event loop: `handle_inputs()` spawns thread polling crossterm with 200ms tick; `App::run()` matches `Event::Key` / `Event::Tick -> check_and_switch()`.
- Terminal: `ratatui::init()` / `ratatui::restore()` must always pair (see `main.rs`).
- State rendering: `PomodoroState::Work` = large blue work timer + `computer()` art; `Break` = large green break timer + `sleeping_cat()`. Timers use `tui-big-text` `PixelSize::Full` (active) vs `Quadrant` (inactive).
- Session name: `App::new(work, break, hide_image, Option<String>)` stores `session_name`. In `draw()`/`get_layout()`, if `Some`, right column reserves `Length(1)` padding + `Length(3)` for name; `get_session_name_widget()` renders yellow `PixelSize::Sextant` BigText (3 rows), centered, truncated via `truncate_with_ellipsis_for_big_text()` (4 cells/char for Sextant, `max_chars = available_width / 4`). Fallback to 4-row layout when `None`.
- CLI validation: `parse_session_name` rejects `>25` chars (`value_parser`). Update both README and AGENTS.md if limits change.
- Docs sync: Keep this file in sync with `README.md` (Features, CLI Flags, Commands, Acknowledgements). `README.md` is the user-facing source of truth; reflect any flag/layout/behavior changes here.
- Distribution: Homebrew tap `iman1704/homebrew-tap` — `Formula/pomodoro-tui.rb` builds from source (`depends_on "rust" => :build`, `cargo install` + `std_cargo_args`, no signing/notarization). New releases: bump `Cargo.toml`, push `vX.Y.Z` tag, update formula `url`/`sha256` in tap (manual `brew bump-formula-pr` or SHA edit for now).
