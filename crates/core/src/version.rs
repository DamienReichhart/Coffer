//! 7-Zip version numbers, read from the banner every 7-Zip build prints.

use std::fmt;

/// A 7-Zip version such as `24.08` or p7zip's `16.02`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Reads the version from 7-Zip's banner, e.g.
    /// `7-Zip 23.01 (x64) : Copyright (c) 1999-2023 Igor Pavlov : 2023-06-20`,
    /// `7-Zip (z) 24.08 (arm64) : ...` or `p7zip Version 16.02 (locale=...)`.
    pub fn from_banner(banner: &str) -> Option<Self> {
        for line in banner.lines() {
            let line = line.trim();
            let rest = if let Some(rest) = line.strip_prefix("7-Zip") {
                rest
            } else if let Some(rest) = line.strip_prefix("p7zip Version") {
                rest
            } else {
                continue;
            };
            if let Some(version) = rest.split_whitespace().find_map(Self::parse) {
                return Some(version);
            }
        }
        None
    }

    fn parse(token: &str) -> Option<Self> {
        let (major, minor) = token.split_once('.')?;
        let minor: String = minor.chars().take_while(char::is_ascii_digit).collect();
        Some(Self::new(major.parse().ok()?, minor.parse().ok()?))
    }

    /// Whether this version supports a feature introduced in `other`.
    pub fn at_least(self, major: u32, minor: u32) -> bool {
        self >= Self::new(major, minor)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:02}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_official_banners() {
        let banner = "\n7-Zip 23.01 (x64) : Copyright (c) 1999-2023 Igor Pavlov : 2023-06-20\n 64-bit locale=C.UTF-8";
        assert_eq!(Version::from_banner(banner), Some(Version::new(23, 1)));
        let banner = "7-Zip (z) 24.08 (arm64) : Copyright (c) 1999-2024 Igor Pavlov : 2024-08-11";
        assert_eq!(Version::from_banner(banner), Some(Version::new(24, 8)));
        let banner = "7-Zip (a) 19.00 (x64) : Copyright (c) 1999-2018 Igor Pavlov : 2019-02-21";
        assert_eq!(Version::from_banner(banner), Some(Version::new(19, 0)));
    }

    #[test]
    fn reads_p7zip_banners() {
        let banner = "p7zip Version 16.02 (locale=utf8,Utf16=on,HugeFiles=on,64 bits,8 CPUs)";
        assert_eq!(Version::from_banner(banner), Some(Version::new(16, 2)));
    }

    #[test]
    fn rejects_other_output() {
        assert_eq!(Version::from_banner("bash: 7z: command not found"), None);
        assert_eq!(Version::from_banner(""), None);
    }

    #[test]
    fn displays_like_7zip() {
        assert_eq!(Version::new(24, 8).to_string(), "24.08");
        assert!(Version::new(24, 8).at_least(23, 1));
        assert!(!Version::new(16, 2).at_least(21, 0));
    }
}
