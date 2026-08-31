use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};

/// Maximum length for title/body fields before truncation (keeps OSC string bounded).
const MAX_FIELD_LEN: usize = 200;

/// Strip control characters and replace field separators that would break OSC parsing.
fn sanitize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            // OSC terminators / DCS framing must never appear inside the payload
            '\x1b' | '\x07' | '\n' | '\r' => out.push(' '),
            // Semicolon separates title/body in OSC 777; normalize to avoid injection
            ';' => out.push(':'),
            _ if ch.is_control() => out.push(' '),
            _ => out.push(ch),
        }
    }
    // Truncate by char count, not bytes, to avoid cutting utf-8 mid-codepoint.
    let char_count = out.chars().count();
    if char_count > MAX_FIELD_LEN {
        out.chars().take(MAX_FIELD_LEN).collect()
    } else {
        out
    }
}

/// OSC 9 — `ESC ] 9 ; <body> BEL` — iTerm2 / Windows Terminal / ConEmu.
/// Single body field (no title). Callers should combine title+body beforehand.
fn osc9(body: &str) -> Vec<u8> {
    let body = sanitize(body);
    let mut v = Vec::with_capacity(4 + body.len());
    v.extend_from_slice(b"\x1b]9;");
    v.extend_from_slice(body.as_bytes());
    v.push(0x07); // BEL terminator — broadest compatibility (ST `\x1b\\` also works but BEL is canonical)
    v
}

/// OSC 777 — `ESC ] 777 ; notify ; <title> ; <body> BEL` — Ghostty / WezTerm / foot / urxvt.
/// Title and body separated; both sanitized.
fn osc777(title: &str, body: &str) -> Vec<u8> {
    let title = sanitize(title);
    let body = sanitize(body);
    let mut v = Vec::with_capacity(12 + title.len() + body.len());
    v.extend_from_slice(b"\x1b]777;notify;");
    v.extend_from_slice(title.as_bytes());
    v.push(b';');
    v.extend_from_slice(body.as_bytes());
    v.push(0x07);
    v
}

/// OSC 99 — Kitty desktop notification.
/// Kitty spec: `ESC ] 99 ; i=<id> : d=<0|1> : p=<title|body> ; <base64_or_plain> ST`
/// We implement the simple single-chunk plain-text form without base64, which
/// kitty accepts: `ESC ] 99 ; i=1 : d=0 ; <title> BEL` followed by body chunk.
/// For pomodoro we keep it to one notification id and reuse plain text.
/// Simpler fallback: if chunking is not needed, kitty also accepts
/// `ESC ] 99 ; i=1:d=0:p=title;body BEL` style in some builds. We use the explicit
/// two-part form for correctness.
fn osc99(title: &str, body: &str) -> Vec<u8> {
    // Kitty uses `i` (notification id), `d` (display), `p` (title). We keep a fixed
    // id 1 and use d=0 for transient. Use plain (non-base64) payload.
    // Sequence per kitty docs: OSC 99 ; i=1:d=0:p=title ; title ST  then body as second chunk
    // with p=body. Many integrators instead send a single combined line:
    // `ESC]99;i=1:d=0:p=body;title\x07`. That gets dropped in newer kitty, so we send
    // the two-chunk variant concatenated with both OSCs.
    // To keep the byte API simple, we return the concatenated bytes of both chunks.
    let title = sanitize(title);
    let body = sanitize(body);
    let mut v = Vec::new();
    // Title chunk — d=0, p= title
    v.extend_from_slice(b"\x1b]99;i=1:d=0:p=title;");
    v.extend_from_slice(title.as_bytes());
    v.push(0x07);
    // Body chunk — d=0, p=body, same id, final
    v.extend_from_slice(b"\x1b]99;i=1:d=0:p=body;");
    v.extend_from_slice(body.as_bytes());
    v.push(0x07);
    v
}

/// Terminal bell — `BEL` — urgency hint fallback.
fn bell() -> Vec<u8> {
    vec![0x07]
}

/// Wrap an OSC sequence for tmux passthrough.
///
/// When `TMUX` is set, tmux requires DCS wrapping: `ESC P tmux ; ESC ESC ] ... BEL ESC \`
/// See https://github.com/tmux/tmux/wiki/AdvancedUse#passthrough
/// The inner `ESC` must be doubled. We do that by replacing `\x1b` with `\x1b\x1b` inside.
fn tmux_wrap(seq: &[u8]) -> Vec<u8> {
    let mut wrapped = Vec::with_capacity(seq.len() + 12);
    wrapped.extend_from_slice(b"\x1bPtmux;");
    for &b in seq {
        if b == 0x1b {
            wrapped.push(0x1b);
            wrapped.push(0x1b);
        } else {
            wrapped.push(b);
        }
    }
    wrapped.extend_from_slice(b"\x1b\\");
    wrapped
}

fn is_tmux() -> bool {
    env::var("TMUX").is_ok()
}

/// Detect which OSC protocol the current terminal most likely supports.
/// Priority: KITTY_WINDOW_ID -> OSC99, iTerm/WT -> OSC9, else OSC777.
fn detect_mode() -> Mode {
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return Mode::Osc99;
    }
    if env::var("TERM_PROGRAM")
        .map(|v| v == "iTerm.app")
        .unwrap_or(false)
        || env::var("ITERM_SESSION_ID").is_ok()
        || env::var("WT_SESSION").is_ok()
    {
        return Mode::Osc9;
    }
    // Ghostty / WezTerm / foot / urxvt all prefer OSC777; it's also understood by kitty,
    // so it is the safest default.
    Mode::Osc777
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Osc9,
    Osc777,
    Osc99,
    Bell,
    Auto,
}

impl Mode {
    fn from_env_or(mode: Mode) -> Mode {
        match mode {
            Mode::Auto => detect_mode(),
            other => other,
        }
    }
}

/// Build the raw OSC byte sequence for the given mode (without tmux wrapping).
fn build_seq(mode: Mode, title: &str, body: &str) -> Vec<u8> {
    match mode {
        Mode::Osc9 => {
            // OSC9 has no title; combine as "Title: Body"
            let combined = if title.is_empty() {
                body.to_string()
            } else if body.is_empty() {
                title.to_string()
            } else {
                format!("{}: {}", title, body)
            };
            osc9(&combined)
        }
        Mode::Osc777 => osc777(title, body),
        Mode::Osc99 => osc99(title, body),
        Mode::Bell => bell(),
        Mode::Auto => unreachable!("Auto should be resolved before build_seq"),
    }
}

/// Emit bytes to the controlling terminal.
///
/// Tries `/dev/tty` first (correct when stdout is taken by ratatui's raw mode),
/// falls back to stderr. Errors are ignored — notifications are best-effort.
fn emit(seq: &[u8]) {
    let final_seq = if is_tmux() {
        tmux_wrap(seq)
    } else {
        seq.to_vec()
    };

    // Prefer /dev/tty so we bypass ratatui's stdout capture.
    if let Ok(mut f) = OpenOptions::new().write(true).open("/dev/tty") {
        let _ = f.write_all(&final_seq);
        let _ = f.flush();
        return;
    }
    // Fallback: stderr (crossterm/ratatui does not own stderr)
    let _ = io::stderr().write_all(&final_seq);
    let _ = io::stderr().flush();
}

/// Public entry: notify with auto-detection.
/// Title is shown as the notification title on terminals that support it (OSC777/OSC99);
/// body is the message. Both are sanitized and capped.
pub fn notify(title: &str, body: &str) {
    notify_with_mode(title, body, Mode::Auto)
}

/// Notify with explicit mode (useful for `--notify` flag or tests).
pub fn notify_with_mode(title: &str, body: &str, mode: Mode) {
    let resolved = Mode::from_env_or(mode);
    let seq = build_seq(resolved, title, body);
    emit(&seq);
}

// Exposed for tests.
#[allow(dead_code)]
#[cfg(test)]
pub(crate) fn build_seq_for_test(mode: Mode, title: &str, body: &str) -> Vec<u8> {
    let resolved = Mode::from_env_or(mode);
    build_seq(resolved, title, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_strips_controls_and_semicolon() {
        assert_eq!(sanitize("a;b"), "a:b");
        assert_eq!(sanitize("a\x1bb"), "a b");
        assert_eq!(sanitize("a\x07b"), "a b");
        assert_eq!(sanitize("a\nb"), "a b");
    }

    #[test]
    fn test_sanitize_truncates() {
        let long = "a".repeat(300);
        assert_eq!(sanitize(&long).chars().count(), MAX_FIELD_LEN);
    }

    #[test]
    fn test_osc9_format() {
        let seq = osc9("hello");
        assert_eq!(seq, b"\x1b]9;hello\x07");
    }

    #[test]
    fn test_osc777_format() {
        let seq = osc777("Pomodoro", "Break time");
        assert_eq!(seq, b"\x1b]777;notify;Pomodoro;Break time\x07");
    }

    #[test]
    fn test_osc99_contains_both_chunks() {
        let seq = osc99("Work", "Done");
        let s = String::from_utf8_lossy(&seq);
        assert!(s.contains("p=title") || s.contains("p=title"), "title chunk missing: {s}");
        assert!(s.contains("p=body"), "body chunk missing: {s}");
        assert!(s.contains("Work"));
        assert!(s.contains("Done"));
    }

    #[test]
    fn test_bell() {
        assert_eq!(bell(), vec![0x07]);
    }

    #[test]
    fn test_tmux_wrap_doubles_esc() {
        let inner = b"\x1b]777;notify;T;B\x07";
        let wrapped = tmux_wrap(inner);
        assert!(wrapped.starts_with(b"\x1bPtmux;"));
        assert!(wrapped.ends_with(b"\x1b\\"));
        // inner ESC 0x1b should appear doubled as 0x1b 0x1b
        assert!(wrapped.windows(2).any(|w| w == [0x1b, 0x1b]));
    }

    #[test]
    fn test_build_seq_osc9_combines_title_body() {
        let seq = build_seq(Mode::Osc9, "Pomodoro", "Break");
        assert_eq!(seq, b"\x1b]9;Pomodoro: Break\x07");
        let seq2 = build_seq(Mode::Osc9, "", "only body");
        assert_eq!(seq2, b"\x1b]9;only body\x07");
    }

    #[test]
    fn test_build_seq_osc777() {
        let seq = build_seq(Mode::Osc777, "Pomodoro", "Work done");
        assert_eq!(seq, b"\x1b]777;notify;Pomodoro;Work done\x07");
    }
}
