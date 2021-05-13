//! Provides utilities for manipulating character values

// This is technically configurable on Unix, but exposing that information
// from the low-level terminal interface and storing it in Reader is a pain.
// Does anyone even care?

/// Character value generated by the Escape key
pub const ESCAPE: char = '\x1b';

/// Character value generated by the Backspace key
///
/// On Unix systems, this is equivalent to `RUBOUT`
#[cfg(unix)]
pub const DELETE: char = RUBOUT;

/// Character value generated by the Backspace key
///
/// On Windows systems, this character is Ctrl-H
#[cfg(windows)]
pub const DELETE: char = '\x08';

/// Character value generated by the Backspace key on some systems
pub const RUBOUT: char = '\x7f';

/// Returns a character name as a key sequence, e.g. `Control-x` or `Meta-x`.
///
/// Returns `None` if the name is invalid.
pub fn parse_char_name(name: &str) -> Option<String> {
    let name_lc = name.to_lowercase();

    let is_ctrl = contains_any(&name_lc, &["c-", "ctrl-", "control-"]);
    let is_meta = contains_any(&name_lc, &["m-", "meta-"]);

    let name = match name_lc.rfind('-') {
        Some(pos) => &name_lc[pos + 1..],
        None => &name_lc[..],
    };

    let ch = match name {
        "del" | "rubout" => DELETE,
        "esc" | "escape" => ESCAPE,
        "lfd" | "newline" => '\n',
        "ret" | "return" => '\r',
        "spc" | "space" => ' ',
        "tab" => '\t',
        s if !s.is_empty() => s.chars().next().unwrap(),
        _ => return None,
    };

    let ch = match (is_ctrl, is_meta) {
        (true, true) => meta(ctrl(ch)),
        (true, false) => ctrl(ch).to_string(),
        (false, true) => meta(ch),
        (false, false) => ch.to_string(),
    };

    Some(ch)
}

/// Returns a character sequence escaped for user-facing display.
///
/// Escape is formatted as `\e`.
/// Control key combinations are prefixed with `\C-`.
pub fn escape_sequence(s: &str) -> String {
    let mut res = String::with_capacity(s.len());

    for ch in s.chars() {
        match ch {
            ESCAPE => res.push_str(r"\e"),
            RUBOUT => res.push_str(r"\C-?"),
            '\\' => res.push_str(r"\\"),
            '\'' => res.push_str(r"\'"),
            '"' => res.push_str(r#"\""#),
            ch if is_ctrl(ch) => {
                res.push_str(r"\C-");
                res.push(unctrl_lower(ch));
            }
            ch => res.push(ch),
        }
    }

    res
}

/// Returns a meta sequence for the given character.
pub fn meta(ch: char) -> String {
    let mut s = String::with_capacity(ch.len_utf8() + 1);
    s.push(ESCAPE);
    s.push(ch);
    s
}

fn contains_any(s: &str, strs: &[&str]) -> bool {
    strs.iter().any(|a| s.contains(a))
}

const CTRL_BIT: u8 = 0x40;
const CTRL_MASK: u8 = 0x1f;

/// Returns whether the given character is a control character.
pub fn is_ctrl(c: char) -> bool {
    const CTRL_MAX: u32 = 0x1f;

    c != '\0' && c as u32 <= CTRL_MAX
}

/// Returns a control character for the given character.
pub fn ctrl(c: char) -> char {
    ((c as u8) & CTRL_MASK) as char
}

/// Returns the printable character corresponding to the given control
/// character.
pub fn unctrl(c: char) -> char {
    ((c as u8) | CTRL_BIT) as char
}

/// Returns the lowercase character corresponding to the given control
/// character.
pub fn unctrl_lower(c: char) -> char {
    unctrl(c).to_ascii_lowercase()
}

#[cfg(test)]
mod test {
    use super::{ctrl, escape_sequence, parse_char_name, unctrl, unctrl_lower};

    #[test]
    fn test_ctrl() {
        assert_eq!(ctrl('A'), '\x01');
        assert_eq!(ctrl('I'), '\t');
        assert_eq!(ctrl('J'), '\n');
        assert_eq!(ctrl('M'), '\r');

        assert_eq!(unctrl('\x01'), 'A');
        assert_eq!(unctrl('\t'), 'I');
        assert_eq!(unctrl('\n'), 'J');
        assert_eq!(unctrl('\r'), 'M');
    }

    #[test]
    fn test_unctrl() {
        assert_eq!(unctrl('\x1d'), ']');
        assert_eq!(unctrl_lower('\x1d'), ']');
    }

    #[test]
    fn test_escape() {
        assert_eq!(escape_sequence("\x1b\x7f"), r"\e\C-?");
    }

    #[test]
    fn test_parse_char() {
        assert_eq!(parse_char_name("Escape"), Some("\x1b".to_owned()));
        assert_eq!(parse_char_name("Control-u"), Some("\x15".to_owned()));
        assert_eq!(parse_char_name("Meta-tab"), Some("\x1b\t".to_owned()));
    }
}
