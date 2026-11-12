//! Parsing the table printed by `7z h`.

use crate::error::{Error, Result};

/// The checksums of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHashes {
    pub path: String,
    pub size: Option<u64>,
    /// `(algorithm name as 7-Zip prints it, hex digest)` in column order.
    pub digests: Vec<(String, String)>,
}

/// Parses the output of `7z h -scrc<A> -scrc<B> files…`.
///
/// The table looks like:
///
/// ```text
/// CRC32    SHA256                                                                    Size  Name
/// -------- ---------------------------------------------------------------- -------------  ------------
/// 363A3020 5891b5b5…                                                                    6  src/a.txt
/// -------- ---------------------------------------------------------------- -------------  ------------
/// ```
///
/// The dashed line gives each column's position, which keeps names with
/// spaces intact. Folders (empty digest columns) are skipped.
pub fn parse(output: &str) -> Result<Vec<FileHashes>> {
    let lines: Vec<&str> = output.lines().map(|l| l.trim_end_matches('\r')).collect();
    let rule =
        lines.iter().position(|l| l.starts_with("--------")).ok_or_else(|| Error::Parse("no checksum table".into()))?;
    let header = lines.get(rule.wrapping_sub(1)).ok_or_else(|| Error::Parse("no checksum header".into()))?;
    let columns = column_spans(lines[rule]);
    if columns.len() < 2 {
        return Err(Error::Parse("unexpected checksum table".into()));
    }
    let header: Vec<char> = header.chars().collect();
    let names: Vec<String> = columns.iter().map(|&(start, end)| slice(&header, start, end)).collect();
    let algorithms = columns.len() - 2;

    let mut files = Vec::new();
    for line in &lines[rule + 1..] {
        if line.starts_with("--------") || line.trim().is_empty() {
            break;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut digests = Vec::with_capacity(algorithms);
        for (i, &(start, end)) in columns.iter().take(algorithms).enumerate() {
            let value = slice(&chars, start, end);
            if !value.is_empty() {
                digests.push((names[i].clone(), value));
            }
        }
        let (size_start, size_end) = columns[algorithms];
        let size = slice(&chars, size_start, size_end).parse().ok();
        let name_start = columns[algorithms + 1].0;
        let path: String = chars.iter().skip(name_start).collect::<String>().trim().to_owned();
        if digests.is_empty() || path.is_empty() {
            continue; // a folder
        }
        files.push(FileHashes { path: path.replace('\\', "/"), size, digests });
    }
    Ok(files)
}

fn column_spans(rule: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (i, c) in rule.chars().enumerate() {
        match (c, start) {
            ('-', None) => start = Some(i),
            (' ', Some(s)) => {
                spans.push((s, i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        spans.push((s, usize::MAX));
    }
    spans
}

fn slice(chars: &[char], start: usize, end: usize) -> String {
    let end = end.min(chars.len());
    if start >= end {
        return String::new();
    }
    chars[start..end].iter().collect::<String>().trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUTPUT: &str = "\n7-Zip 23.01 (x64)\n\nScanning\n2 files, 12 bytes (1 KiB)\n\nCRC32    SHA256                                                                    Size  Name\n-------- ---------------------------------------------------------------- -------------  ------------\n363A3020 5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03             6  src/a.txt\n                                                                                       0  src/sub\nC0F7BFD3 63d0d0a196ea2557ecdfd6092dd528ddb2c9367a1742942b0b75e3ba3972d1eb             6  src/sub/naïve file.txt\n-------- ---------------------------------------------------------------- -------------  ------------\nF731EFF3-00000000 bb61-00000000            12  \n\nEverything is Ok\n";

    #[test]
    fn parses_the_table() {
        let files = parse(OUTPUT).expect("parse");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/a.txt");
        assert_eq!(files[0].size, Some(6));
        assert_eq!(files[0].digests[0], ("CRC32".to_owned(), "363A3020".to_owned()));
        assert_eq!(files[0].digests[1].0, "SHA256");
        assert!(files[0].digests[1].1.starts_with("5891b5b5"));
        assert_eq!(files[1].path, "src/sub/naïve file.txt");
    }

    #[test]
    fn rejects_other_output() {
        assert!(parse("ERROR: nope").is_err());
    }
}
