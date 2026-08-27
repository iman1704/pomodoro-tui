# AGENTS.md

Muse Code reads this file as project rules when it runs in this directory.

## Project

- Name: pomodoro-tui (`pomodoro-tui` v0.2.0) — terminal Pomodoro timer
- Stack: Rust 2021, ratatui 0.29 + crossterm 0.29, clap 4 (derive), rodio, notify-rust, tui-big-text
- Purpose: Work/Break state machine (`src/lib.rs`) + ratatui TUI (`src/app.rs`). Works well in tmux.

## Common Commands

- Build: `cargo build`
- Run: `cargo run` or `cargo run -- -w 25 -b 5` | `cargo run -- -w 30 -b 10 -i` | `cargo run -- --no-sound`
- Test: `cargo test`

## Project Layout

- `src/main.rs`: CLI parsing (clap) + terminal init/restore, delegates to `App`
- `src/lib.rs`: Core `Pomodoro`/`Timer` logic, `check_and_switch()`, sound/notification, unit tests
- `src/app.rs`: `App` — event loop (200ms Tick), rendering, layout
- `src/ascii_images.rs`: ASCII art (`computer()` for Work, `sleeping_cat()` for Break)
- `default_sound.mp3` + `doc/` + `Cargo.toml`

## CLI Flags (`src/main.rs`)

| Flag | Default | Notes |
|------|---------|-------|
| `-w, --work <mins>` | `25` | Work duration |
| `-b, --break <mins>` | `5` | Break duration |
| `-i, --hide-image` | `false` | Hide ASCII art (expands timer to 100% width) |
| `-s, --sound <path>` | `default_sound.mp3` | Resolved via `CARGO_MANIFEST_DIR` if omitted |
| `-n, --no-sound` | `false` | Suppress sound/`say` |

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
- State rendering: `PomodoroState::Work` = large blue work timer + `computer()` art; `Break` = large green break timer + `sleeping_cat()`.
- Notifications/sound: macOS uses `osascript display notification` + `say "Thomas (French (France))"`; Linux uses `notify-rust` + `rodio` on spawned thread. Both gated by `no_sound`. No `clippy`/`fmt` gate in this repo.
