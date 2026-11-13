//! `cargo xtask release-plan`: every check that must pass before anything is
//! built, so a release fails in seconds rather than after an hour of builds.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::{capture, changelog, workspace_version};

/// The crates whose version must equal the release version.
const CRATES: [&str; 3] = ["coffer-core", "coffer-updater", "coffer"];

pub fn plan(root: &Path, notes: Option<&Path>, check_git: bool) -> Result<()> {
    let version = workspace_version(root)?;
    let tag = format!("v{version}");
    eprintln!("Planning the release of {tag}");

    lockfile_matches(root, &version)?;

    let changelog_text = std::fs::read_to_string(root.join("CHANGELOG.md")).context("CHANGELOG.md is missing")?;
    let section = changelog::section(&changelog_text, &version.to_string()).with_context(|| {
        format!("CHANGELOG.md has no `## [{version}] - YYYY-MM-DD` section: describe the release before bumping")
    })?;
    ensure!(!section.body.is_empty(), "the CHANGELOG.md section of {version} is empty");
    let date = section.date.clone().with_context(|| format!("the CHANGELOG.md section of {version} has no date"))?;
    ensure!(is_iso_date(&date), "the CHANGELOG.md date of {version} is not YYYY-MM-DD: {date}");

    if check_git {
        let tags = capture(Command::new("git").args(["tag", "--list", "v*"]))?;
        let remote = capture(Command::new("git").args(["ls-remote", "--tags", "origin", &tag])).unwrap_or_default();
        if tags.lines().any(|t| t.trim() == tag) || !remote.trim().is_empty() {
            bail!(
                "{tag} is already released. Bump the version in Cargo.toml (and add its CHANGELOG.md section) to release again; nothing was built."
            );
        }
        let newest = tags
            .lines()
            .filter_map(|t| t.trim().strip_prefix('v'))
            .filter_map(|v| v.parse::<semver::Version>().ok())
            .max();
        if let Some(newest) = newest {
            ensure!(version > newest, "{version} is not newer than the latest release {newest}");
        }
    }

    let prerelease = !version.pre.is_empty();
    let mut body = section.body.clone();
    let _ = write!(
        body,
        "\n\n---\n\n**Install:** see the [download page](https://damienreichhart.github.io/Coffer/download/). \
         Coffer needs a 7-Zip you install yourself ([how](https://damienreichhart.github.io/Coffer/docs/installing-7zip/)). \
         Every file is listed in `SHA256SUMS.txt`, signed with minisign (`.minisig`) and has build provenance (`gh attestation verify`)."
    );
    if let Some(path) = notes {
        std::fs::write(path, &body)?;
    }

    let outputs = format!("version={version}\ntag={tag}\nprerelease={prerelease}\ndate={date}\n");
    print!("{outputs}");
    if let Ok(file) = std::env::var("GITHUB_OUTPUT") {
        use std::io::Write as _;
        std::fs::OpenOptions::new().append(true).open(file)?.write_all(outputs.as_bytes())?;
    }
    eprintln!("{tag} is ready to build.");
    Ok(())
}

/// The lockfile must already carry the new version, so `--locked` builds pass.
fn lockfile_matches(root: &Path, version: &semver::Version) -> Result<()> {
    let lock: toml::Table = std::fs::read_to_string(root.join("Cargo.lock"))?.parse()?;
    let packages = lock.get("package").and_then(|p| p.as_array()).context("Cargo.lock has no packages")?;
    for name in CRATES {
        let found = packages
            .iter()
            .find(|p| p.get("name").and_then(|n| n.as_str()) == Some(name))
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .with_context(|| format!("{name} is not in Cargo.lock"))?;
        ensure!(
            found == version.to_string(),
            "Cargo.lock has {name} {found}, not {version}: run `cargo update --workspace` and commit Cargo.lock"
        );
    }
    Ok(())
}

fn is_iso_date(date: &str) -> bool {
    let parts: Vec<&str> = date.split('-').collect();
    parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert!(is_iso_date("2026-11-12"));
        assert!(!is_iso_date("12/11/2026"));
        assert!(!is_iso_date("2026-1-2"));
    }
}
