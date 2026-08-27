# Pomodoro

A simple Pomodoro timer built in Rust. It uses the excellent [ratatui library](https://ratatui.rs/) to render a terminal UI.


## Features

- **Work / Break state machine** — automatic switching between Work and Break phases with large, color-coded timers (blue for Work, green for Break).
- **Terminal UI (TUI)** — built with [ratatui](https://ratatui.rs/) + [crossterm](https://github.com/crossterm-rs/crossterm. Works great inside `tmux`.
- **Customizable durations** — configure work and break lengths via CLI flags.
- **ASCII art** — custom ascii art that adapts to session types.
- **Desktop notifications & sound** — native notifications on Linux (`notify-rust`) and macOS (`osascript`); audio playback via `rodio` (Linux) / `say` with `Thomas (French (France))` voice (macOS).
- **Session name** — specify custom session names.
- **Cross-platform** — Linux and macOS supported.

## Installation

- [ ] TODO: update installation instructions

## How to Run

```bash
pomodoro-tui
```

By default, the timer is set to 25 minutes for work sessions and 5 minutes for breaks. You can change these values using the `-w/--work` and `-b/--break` flags. You can also remove the ASCII art next to the timers using the `-i/--hide-image` flag. For instance, if you want to set the work timer to 30 minutes and the break timer to 10 minutes, and hide the ASCII art, you can run:

```bash
pomodoro-tui -w 30 -b 10 -i
```

## Common Commands & Flags

### Cargo Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Build the project (debug) |
| `cargo build --release` | Build optimized binary |
| `cargo run` | Run with defaults (25 min work / 5 min break) |
| `cargo run -- -w 30 -b 10 -i` | Run with 30 min work, 10 min break, no ASCII art |
| `cargo run -- --quiet` | Run with notifications muted |
| `cargo run -- -n "Deep Work"` | Run with session name below timers |
| `cargo test` | Run unit tests |

### CLI Flags (`pomodoro-tui --help`)

| Flag | Default | Description |
|------|---------|-------------|
| `-w, --work <mins>` | `25` | Work duration in minutes |
| `-b, --break <mins>` | `5` | Break duration in minutes |
| `-i, --hide-image` | `false` | Hide ASCII art (timer expands to 100% width) |
| `-s, --sound <path>` | `default_sound.mp3` | Custom sound file path  |
| `-n, --name <NAME>` | — | Session name (max 25 chars)  |
| `-q, --quiet` | `false` | Suppress sound and `say` voice (notifications still shown) |
| `-h, --help` | — | Print help |
| `-V, --version` | — | Print version |

Examples:

```bash
# Default 25/5
pomodoro-tui

# Custom durations
pomodoro-tui -w 30 -b 10

# Hide ASCII art, custom sound
pomodoro-tui -i -s /path/to/sound.mp3

# Session name (BigText below timers, e.g. visible as yellow 3-row text)
pomodoro-tui -n "Deep Work"
pomodoro-tui --name "Study Session" -w 50 -b 10

# Silent mode (no sound/voice)
pomodoro-tui --quiet
pomodoro-tui -q

# Via cargo
cargo run -- -w 25 -b 5
cargo run -- -w 30 -b 10 -i --quiet
cargo run -- -n "Deep Work" --quiet
```

### Keybindings

| Key | Action |
|-----|--------|
| `s` | Start / Pause current timer |
| `r` | Reset both timers to Work state |
| `q` / `Esc` | Quit |

## About Notifications

On Linux and macOS, the app will send a desktop notification when the work or break time is over.

- **Linux**: uses `notify-rust` for notifications and `rodio` for sound playback on a background thread.
- **macOS**: uses `osascript display notification` and the native `say` command with the `Thomas (French (France))` voice.

Both are gated by `--quiet` / `-q` to run silently.

Thanks to @Cythonic1 for adding Linux notification support!

## Acknowledgements

> **Fork notice:** This project is a fork of the original [xamcost/pomodoro](https://github.com/xamcost/pomodoro) (published as `pomodoro-tui` on crates.io) originally created by [Maxime Costalonga](https://github.com/xamcost). All core Pomodoro logic, TUI design, and initial implementation credit goes to the upstream project and its contributors.

This fork is maintained by [iman1704](https://github.com/iman1704/pomodoro-tui) — Cargo metadata and maintenance updates have been applied while keeping the original functionality intact.

Original acknowledgements from upstream:

> This small project to learn Rust has been inspired by my partner, who likes and encourages me to use the Pomodoro technique, even if she doesn't always enjoy breaks when it's time...

I also want to thank the authors of the ASCII art used in the app, which are combinations of works by _jgs_ and _Felix Lee_ you can find [here](https://www.asciiart.eu/computers/computers) and [here](https://www.asciiart.eu/animals/cats).

Additional thanks to all upstream contributors, especially @Cythonic1 for Linux support.
