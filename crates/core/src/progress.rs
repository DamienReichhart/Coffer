//! Reading 7-Zip's live progress (`-bsp1`) from its console output.
//!
//! 7-Zip redraws its progress line in place with backspaces, e.g.
//! `\x08\x08\x08 45% 12 + photos/a.jpg`, and prints `-bb1` log lines such as
//! `- docs/readme.txt` with newlines. [`Parser`] splits the raw byte stream on
//! backspace, carriage return and newline, and reports both.

/// Something 7-Zip reported while running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    /// Overall progress in percent, with the file being processed when known.
    Progress { percent: u8, file: Option<String> },
    /// A complete line of normal output, with progress redraws removed.
    Line(String),
}

/// An incremental parser for 7-Zip's console output.
#[derive(Debug, Default)]
pub struct Parser {
    pending: Vec<u8>,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds a chunk of output and returns what it completed.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<Signal> {
        let mut signals = Vec::new();
        for &byte in chunk {
            match byte {
                b'\n' => self.flush(&mut signals, true),
                b'\r' | 0x08 => self.flush(&mut signals, false),
                _ => self.pending.push(byte),
            }
        }
        signals
    }

    /// Returns whatever is left at the end of the stream.
    pub fn finish(&mut self) -> Vec<Signal> {
        let mut signals = Vec::new();
        self.flush(&mut signals, true);
        signals
    }

    fn flush(&mut self, signals: &mut Vec<Signal>, newline: bool) {
        if self.pending.is_empty() {
            return;
        }
        let text = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending.clear();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        if let Some(signal) = parse_progress(trimmed) {
            signals.push(signal);
        } else if newline {
            // Leading spaces are kept: they align 7-Zip's tables.
            signals.push(Signal::Line(text.trim_end().to_owned()));
        }
    }
}

/// Parses `45%`, `45% 12`, `45% + file` or `45% 12 - file`.
fn parse_progress(segment: &str) -> Option<Signal> {
    let (number, rest) = segment.split_once('%')?;
    let percent: u8 = number.trim().parse().ok().filter(|p| *p <= 100)?;
    let mut rest = rest.trim_start();
    // An optional count of processed files.
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 {
        rest = rest[digits..].trim_start();
    }
    // An optional operation marker: + add, - extract, T test, U update, D delete, R rename, = copy.
    let mut chars = rest.chars();
    if let (Some(op), Some(' ')) = (chars.next(), chars.next())
        && matches!(op, '+' | '-' | 'T' | 'U' | 'D' | 'R' | '=' | 'A')
    {
        rest = &rest[2..];
    }
    let file = rest.trim();
    Some(Signal::Progress { percent, file: (!file.is_empty()).then(|| file.to_owned()) })
}

/// The file named by a `-bb1` log line such as `- docs/a.txt` or `+ a.txt`.
pub fn logged_file(line: &str) -> Option<&str> {
    let mut chars = line.chars();
    match (chars.next(), chars.next()) {
        (Some('+' | '-' | 'T' | 'U' | 'D' | 'R' | '='), Some(' ')) => Some(line[2..].trim()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(percent: u8, file: Option<&str>) -> Signal {
        Signal::Progress { percent, file: file.map(ToOwned::to_owned) }
    }

    #[test]
    fn reads_backspace_redrawn_progress() {
        let mut parser = Parser::new();
        let raw = b"  0M Scan\x08\x08\x08\x08\x08\x08\x08\x08\x08         \x08\x08\x08\x08  0%\x08\x08\x08\x08    \x08\x08\x08\x08 20% + big.bin\x08\x08\x08\x08 45% + big.bin\x08\x08 99% 1 + big.bin\x08\x08";
        let signals = parser.feed(raw);
        assert_eq!(
            signals,
            [
                progress(0, None),
                progress(20, Some("big.bin")),
                progress(45, Some("big.bin")),
                progress(99, Some("big.bin")),
            ]
        );
    }

    #[test]
    fn keeps_lines_split_across_chunks() {
        let mut parser = Parser::new();
        assert!(parser.feed(b"- docs/na\xc3").is_empty());
        let signals = parser.feed(b"\xafve.txt\nEverything is Ok\n");
        assert_eq!(signals, [Signal::Line("- docs/naïve.txt".into()), Signal::Line("Everything is Ok".into())]);
    }

    #[test]
    fn parses_test_and_extract_markers() {
        assert_eq!(parse_progress("99% 2 T src/sub/b.bin"), Some(progress(99, Some("src/sub/b.bin"))));
        assert_eq!(parse_progress("12% - a b.txt"), Some(progress(12, Some("a b.txt"))));
        assert_eq!(parse_progress("100%"), Some(progress(100, None)));
        assert_eq!(parse_progress("Scanning"), None);
        assert_eq!(parse_progress("250%"), None);
    }

    #[test]
    fn finish_flushes_the_rest() {
        let mut parser = Parser::new();
        assert!(parser.feed(b"Everything is Ok").is_empty());
        assert_eq!(parser.finish(), [Signal::Line("Everything is Ok".into())]);
    }

    #[test]
    fn log_lines_name_files() {
        assert_eq!(logged_file("- docs/a.txt"), Some("docs/a.txt"));
        assert_eq!(logged_file("+ a.txt"), Some("a.txt"));
        assert_eq!(logged_file("Everything is Ok"), None);
    }
}
