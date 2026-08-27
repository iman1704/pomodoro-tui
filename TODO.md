
**Docs/README.md**
- [ ] Change install guide

**Core features**
- [ ] Add custom notification feature (integrate terminal-notifier)
- [ ] Add seconds, minutes, hours to time (e.g 20s, 1m30s, 1h2m20s, etc...)
- [ ] Add session count
  - new `-session` flag
- [ ] Remove voice notification feature, keep only sound.
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
