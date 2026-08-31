**Deploy**
- [x] Deploy as homebrew package (Phase 1 — source-build)
  - [x] Setup tap repo (`iman1704/homebrew-tap` — `Formula/pomodoro-tui.rb`, `cargo install` via `std_cargo_args`)
  - [x] Setup CI/CD Github Action workflow (`.github/workflows/ci.yml`, `.github/workflows/release.yml`)

**Docs/README.md**
- [x] Change install guide (Homebrew + Cargo instructions)

**Core features**
- [x] Scrap current notification feature
- [x] Add custom notification feature (use terminal emulator native notification feature)
  - OSC 777 (`ESC]777;notify;title;body BEL`) for Ghostty/WezTerm/foot/urxvt, OSC 9 for iTerm2/Windows Terminal, OSC 99 for kitty, BEL fallback; tmux DCS passthrough (`ESC P tmux; ESC ESC]... BEL ESC \`) via `src/notify.rs`, `lib.rs:check_and_switch() -> Option<PomodoroState>` hook in `app.rs`
- [ ] Add seconds, minutes, hours to time (e.g 20s, 1m30s, 1h2m20s, etc...)
- [ ] Add session count
  - new `-session` flag
- [ ] Custom ascii art
- [x] Session name
  - new `-n --name` flag
  - change existing `-n --no-sound` flag to `-q --quiet`
  - session name is at the bottom of the timer
  - if no name is specified fallback to default layout
- [ ] Remove `esc` to exit app

**Rendering engine improvements**
- [ ] Flexible layout
  - Adapt to any terminal window size
  - Truncate or wrap to accomodate smaller window size
