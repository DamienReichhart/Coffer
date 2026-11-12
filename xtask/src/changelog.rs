//! Reading CHANGELOG.md (Keep a Changelog format).

/// One `## [x.y.z] - YYYY-MM-DD` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub version: String,
    pub date: Option<String>,
    pub body: String,
}

/// Every versioned section, newest first as written.
pub fn sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current: Option<Section> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            let rest = rest.trim();
            let (head, date) = match rest.split_once(" - ") {
                Some((head, date)) => (head.trim(), Some(date.trim().to_owned())),
                None => (rest, None),
            };
            let version = head.trim_start_matches('[').trim_end_matches(']').trim().to_owned();
            current = Some(Section { version, date, body: String::new() });
            continue;
        }
        // Link reference definitions at the end belong to no section.
        if line.starts_with('[') && line.contains("]: ") {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            continue;
        }
        if let Some(section) = current.as_mut() {
            section.body.push_str(line);
            section.body.push('\n');
        }
    }
    if let Some(section) = current.take() {
        sections.push(section);
    }
    for section in &mut sections {
        section.body = section.body.trim().to_owned();
    }
    sections
}

pub fn section(text: &str, version: &str) -> Option<Section> {
    let version = version.trim_start_matches('v');
    sections(text).into_iter().find(|s| s.version == version)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "# Changelog\n\n## [Unreleased]\n\n## [1.1.0] - 2026-11-20\n\n### Added\n\n- Checksums.\n\n## [1.0.0] - 2026-11-12\n\n- First release.\n\n[1.1.0]: https://x/compare/v1.0.0...v1.1.0\n[1.0.0]: https://x/releases/tag/v1.0.0\n";

    #[test]
    fn finds_sections() {
        let all = sections(TEXT);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].version, "Unreleased");
        assert!(all[0].body.is_empty());
        let s = section(TEXT, "v1.1.0").expect("1.1.0");
        assert_eq!(s.date.as_deref(), Some("2026-11-20"));
        assert_eq!(s.body, "### Added\n\n- Checksums.");
        assert_eq!(section(TEXT, "1.0.0").expect("1.0.0").body, "- First release.");
        assert!(section(TEXT, "2.0.0").is_none());
    }
}
