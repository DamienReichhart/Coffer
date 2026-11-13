//! Multi-volume archives: several files that together are one archive.
//!
//! 7-Zip reads every common way of splitting an archive, but only when it is
//! given the right volume, and it fails with a vague "unexpected end of
//! archive" when a volume is missing. This module recognises the naming
//! schemes from the file names alone, finds the sibling volumes on disk,
//! picks the one 7-Zip must open and lists the ones that are missing.
//!
//! | Scheme | Volumes | 7-Zip opens |
//! | --- | --- | --- |
//! | [`Scheme::Numbered`] | `backup.7z.001`, `backup.7z.002`… (7-Zip, HJSplit, `split`) | `.001` |
//! | [`Scheme::RarParts`] | `movie.part1.rar`, `movie.part2.rar`… (RAR 3 and later) | `.part1.rar` |
//! | [`Scheme::RarLegacy`] | `movie.rar`, `movie.r00`, `movie.r01`… (RAR 2) | `.rar` |
//! | [`Scheme::ZipSpanned`] | `site.z01`, `site.z02`…, `site.zip` (Info-ZIP, WinZip) | `.zip` |

use std::path::{Path, PathBuf};

/// How the volumes of a set are named.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scheme {
    /// `name.001`, `name.002`… after the full archive name.
    Numbered,
    /// `name.part1.rar`, `name.part2.rar`…
    RarParts,
    /// `name.rar`, then `name.r00`, `name.r01`…
    RarLegacy,
    /// `name.z01`, `name.z02`…, and `name.zip` last.
    ZipSpanned,
}

impl Scheme {
    /// A short description for people.
    pub fn label(self) -> &'static str {
        match self {
            Self::Numbered => "Split archive",
            Self::RarParts | Self::RarLegacy => "Multi-part RAR",
            Self::ZipSpanned => "Spanned ZIP",
        }
    }
}

/// What a single file name says about the set it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub scheme: Scheme,
    /// The name every volume of the set shares, e.g. `backup.7z` for
    /// `backup.7z.003`, `movie` for `movie.part2.rar`.
    pub base: String,
    /// The position of this volume in the set, from 1. For a spanned ZIP the
    /// `.zip` file is last, but its position is unknown from its name alone:
    /// it is reported as [`u32::MAX`].
    pub index: u32,
    /// How many digits the volume number has (`001` → 3), to name missing volumes.
    pub width: usize,
}

impl Part {
    /// The archive name to show for the whole set: `backup.7z`, `movie.rar`, `site.zip`.
    pub fn display_name(&self) -> String {
        match self.scheme {
            Scheme::Numbered => self.base.clone(),
            Scheme::RarParts | Scheme::RarLegacy => format!("{}.rar", self.base),
            Scheme::ZipSpanned => format!("{}.zip", self.base),
        }
    }

    /// The file name of volume `index` of this set.
    fn file_name(&self, index: u32, last_extension: &str) -> String {
        let width = self.width;
        match self.scheme {
            Scheme::Numbered => format!("{}.{index:0width$}", self.base),
            Scheme::RarParts => format!("{}.part{index:0width$}.rar", self.base),
            Scheme::RarLegacy if index == 1 => format!("{}.{last_extension}", self.base),
            Scheme::RarLegacy => format!("{}.r{:0width$}", self.base, index - 2),
            Scheme::ZipSpanned if index == u32::MAX => format!("{}.{last_extension}", self.base),
            Scheme::ZipSpanned => format!("{}.z{index:0width$}", self.base),
        }
    }
}

/// Reads a volume's file name. `None` for a file that is not a volume.
///
/// A lone `name.rar` or `name.zip` *could* be the first or last volume of a
/// set; it is reported with [`Part::index`] 1 or [`u32::MAX`], and [`detect`]
/// decides from its siblings.
pub fn parse_name(name: &str) -> Option<Part> {
    let (stem, extension) = name.rsplit_once('.')?;
    if stem.is_empty() {
        return None;
    }
    let lower = extension.to_ascii_lowercase();

    // name.ext.001: three digits, or more with leading zeros, so a year
    // (`report.2024`) is not mistaken for volume 2024.
    let zero_padded = lower.len() == 3 || (lower.len() > 3 && lower.starts_with('0'));
    if zero_padded && lower.bytes().all(|b| b.is_ascii_digit()) {
        let index = lower.parse().ok()?;
        return Some(Part { scheme: Scheme::Numbered, base: stem.to_owned(), index, width: lower.len() });
    }
    // name.part1.rar
    if lower == "rar" {
        if let Some((base, part)) = stem.rsplit_once('.')
            && let Some(digits) = part.get(..4).filter(|p| p.eq_ignore_ascii_case("part")).and(part.get(4..))
            && !digits.is_empty()
            && digits.bytes().all(|b| b.is_ascii_digit())
            && !base.is_empty()
        {
            let index = digits.parse().ok()?;
            return Some(Part { scheme: Scheme::RarParts, base: base.to_owned(), index, width: digits.len() });
        }
        return Some(Part { scheme: Scheme::RarLegacy, base: stem.to_owned(), index: 1, width: 2 });
    }
    if lower == "zip" {
        return Some(Part { scheme: Scheme::ZipSpanned, base: stem.to_owned(), index: u32::MAX, width: 2 });
    }
    // name.r00 (RAR 2) and name.z01 (spanned ZIP)
    let mut chars = lower.chars();
    let letter = chars.next()?;
    let digits = chars.as_str();
    if digits.len() >= 2 && digits.bytes().all(|b| b.is_ascii_digit()) {
        let number: u32 = digits.parse().ok()?;
        return match letter {
            'r' => {
                Some(Part { scheme: Scheme::RarLegacy, base: stem.to_owned(), index: number + 2, width: digits.len() })
            }
            'z' if number > 0 => {
                Some(Part { scheme: Scheme::ZipSpanned, base: stem.to_owned(), index: number, width: digits.len() })
            }
            _ => None,
        };
    }
    None
}

/// The volumes of a multi-volume archive found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeSet {
    pub scheme: Scheme,
    /// The volume 7-Zip must be given to read the whole set.
    pub primary: PathBuf,
    /// The archive's name, without the volume numbering.
    pub name: String,
    /// Every volume found, in order.
    pub volumes: Vec<PathBuf>,
    /// The file names of the volumes that should be there and are not.
    /// Volumes after the last one found can't be known from the names.
    pub missing: Vec<String>,
    /// The size of all the volumes found, in bytes.
    pub total_size: u64,
}

impl VolumeSet {
    /// Whether every volume between the first and the last one found is there.
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }
}

/// Finds the set `path` belongs to, by looking at the files next to it.
///
/// `None` when `path` is not part of a multi-volume archive: a plain
/// `photos.zip` or `photos.rar` with no sibling volumes, or any other file.
pub fn detect(path: &Path) -> Option<VolumeSet> {
    let file_name = path.file_name()?.to_str()?;
    let part = parse_name(file_name)?;
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));

    let mut found: Vec<(u32, PathBuf, String)> = Vec::new();
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(sibling) = parse_name(name) else { continue };
        if sibling.scheme != part.scheme || sibling.base != part.base {
            continue;
        }
        if entry.file_type().is_ok_and(|t| t.is_file() || t.is_symlink()) {
            found.push((sibling.index, entry.path(), name.to_owned()));
        }
    }
    // The file itself may be named with a different case than read_dir reports
    // on case-insensitive file systems; it is always part of its own set.
    if !found.iter().any(|(index, _, _)| *index == part.index) && path.is_file() {
        found.push((part.index, path.to_path_buf(), file_name.to_owned()));
    }
    found.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.2.cmp(&b.2)));
    found.dedup_by_key(|(index, _, _)| *index);

    // A `.rar` without any `.r00`, or a `.zip` without any `.z01`, is an
    // ordinary archive.
    let has_last_zip = found.iter().any(|(index, _, _)| *index == u32::MAX);
    let only = |index: u32| found.iter().all(|(i, _, _)| *i == index);
    match part.scheme {
        Scheme::RarLegacy if only(1) => return None,
        Scheme::ZipSpanned if only(u32::MAX) => return None,
        _ => {}
    }

    let extension = |default: &str| {
        found
            .iter()
            .find(|(index, _, _)| *index == if part.scheme == Scheme::ZipSpanned { u32::MAX } else { 1 })
            .and_then(|(_, _, name)| name.rsplit_once('.').map(|(_, ext)| ext.to_owned()))
            .unwrap_or_else(|| default.to_owned())
    };
    let last_extension = extension(if part.scheme == Scheme::ZipSpanned { "zip" } else { "rar" });

    let highest = found.iter().map(|(index, _, _)| *index).filter(|i| *i != u32::MAX).max().unwrap_or(0);
    let mut missing: Vec<String> = (1..=highest)
        .filter(|index| !found.iter().any(|(i, _, _)| i == index))
        .map(|index| part.file_name(index, &last_extension))
        .collect();
    if part.scheme == Scheme::ZipSpanned && !has_last_zip {
        missing.push(part.file_name(u32::MAX, &last_extension));
    }

    let primary_index = if part.scheme == Scheme::ZipSpanned { u32::MAX } else { 1 };
    let primary = found
        .iter()
        .find(|(index, _, _)| *index == primary_index)
        .map_or_else(|| dir.join(part.file_name(primary_index, &last_extension)), |(_, path, _)| path.clone());
    let total_size = found.iter().filter_map(|(_, path, _)| std::fs::metadata(path).ok()).map(|m| m.len()).sum();

    // In order: `.rar` has index 1 and a spanned `.zip` u32::MAX, so both sort right.
    let volumes: Vec<PathBuf> = found.into_iter().map(|(_, path, _)| path).collect();
    // One complete volume is just an archive with an unusual name.
    if volumes.len() == 1 && missing.is_empty() {
        return None;
    }

    Some(VolumeSet { scheme: part.scheme, primary, name: part.display_name(), volumes, missing, total_size })
}

/// The file to give 7-Zip for `path`: the primary volume when `path` is one
/// volume of a set, `path` itself otherwise.
pub fn primary(path: &Path) -> PathBuf {
    detect(path).map_or_else(|| path.to_path_buf(), |set| set.primary)
}

/// Replaces the volumes of each set in `paths` by the set's primary volume,
/// once, keeping the order of first appearance. Other paths are kept as they are.
///
/// Dropping the eleven files of a split archive on the window gives one archive.
pub fn collapse(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::with_capacity(paths.len());
    for path in paths {
        let resolved = if path.is_file() { primary(path) } else { path.clone() };
        if !out.contains(&resolved) {
            out.push(resolved);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(name: &str) -> (Scheme, String, u32) {
        let p = parse_name(name).unwrap_or_else(|| panic!("{name} is a volume"));
        (p.scheme, p.base, p.index)
    }

    #[test]
    fn reads_volume_names() {
        assert_eq!(part("backup.7z.001"), (Scheme::Numbered, "backup.7z".into(), 1));
        assert_eq!(part("backup.ZIP.011"), (Scheme::Numbered, "backup.ZIP".into(), 11));
        assert_eq!(part("disk.0004"), (Scheme::Numbered, "disk".into(), 4));
        assert_eq!(part("movie.part1.rar"), (Scheme::RarParts, "movie".into(), 1));
        assert_eq!(part("movie.Part07.RAR"), (Scheme::RarParts, "movie".into(), 7));
        assert_eq!(part("movie.rar"), (Scheme::RarLegacy, "movie".into(), 1));
        assert_eq!(part("movie.r00"), (Scheme::RarLegacy, "movie".into(), 2));
        assert_eq!(part("movie.r12"), (Scheme::RarLegacy, "movie".into(), 14));
        assert_eq!(part("site.z01"), (Scheme::ZipSpanned, "site".into(), 1));
        assert_eq!(part("site.zip"), (Scheme::ZipSpanned, "site".into(), u32::MAX));
    }

    #[test]
    fn ignores_other_names() {
        for name in ["notes.txt", "a.7z", "photo.r", "x.z00", "x.zz1", ".001", "x.01", "report.2024", "README"] {
            assert_eq!(parse_name(name), None, "{name}");
        }
        // Not a numbered part: an ordinary RAR whose name contains a dot.
        assert_eq!(part("x.part.rar"), (Scheme::RarLegacy, "x.part".into(), 1));
    }

    #[test]
    fn names_the_whole_set() {
        let name = |n: &str| parse_name(n).map(|p| p.display_name());
        assert_eq!(name("backup.7z.003").as_deref(), Some("backup.7z"));
        assert_eq!(name("movie.part2.rar").as_deref(), Some("movie.rar"));
        assert_eq!(name("movie.r04").as_deref(), Some("movie.rar"));
        assert_eq!(name("site.z02").as_deref(), Some("site.zip"));
    }

    fn touch(dir: &Path, names: &[&str]) {
        for name in names {
            std::fs::write(dir.join(name), b"0123456789").expect("write");
        }
    }

    fn file_names(paths: &[PathBuf]) -> Vec<String> {
        paths.iter().map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn finds_a_numbered_set_from_any_volume() {
        let dir = tempfile::tempdir().expect("tempdir");
        let names: Vec<String> = (1..=11).map(|i| format!("big.zip.{i:03}")).collect();
        touch(dir.path(), &names.iter().map(String::as_str).collect::<Vec<_>>());
        touch(dir.path(), &["big.zip", "other.7z.001", "notes.txt"]);

        let set = detect(&dir.path().join("big.zip.007")).expect("a set");
        assert_eq!(set.scheme, Scheme::Numbered);
        assert_eq!(set.name, "big.zip");
        assert_eq!(set.primary, dir.path().join("big.zip.001"));
        assert_eq!(file_names(&set.volumes), names);
        assert!(set.is_complete());
        assert_eq!(set.total_size, 110);
    }

    #[test]
    fn reports_missing_volumes() {
        let dir = tempfile::tempdir().expect("tempdir");
        touch(dir.path(), &["a.7z.002", "a.7z.003", "a.7z.005"]);
        let set = detect(&dir.path().join("a.7z.003")).expect("a set");
        assert_eq!(set.missing, ["a.7z.001", "a.7z.004"]);
        assert_eq!(set.primary, dir.path().join("a.7z.001"));
        assert!(!set.is_complete());
    }

    #[test]
    fn rar_sets() {
        let dir = tempfile::tempdir().expect("tempdir");
        touch(dir.path(), &["new.part01.rar", "new.part02.rar", "new.part03.rar"]);
        touch(dir.path(), &["old.rar", "old.r00", "old.r01", "old.r03"]);
        touch(dir.path(), &["single.rar", "lone.7z.001", "orphan.r00"]);

        let new = detect(&dir.path().join("new.part03.rar")).expect("new style");
        assert_eq!(new.primary, dir.path().join("new.part01.rar"));
        assert_eq!(new.volumes.len(), 3);
        assert_eq!(new.name, "new.rar");

        let old = detect(&dir.path().join("old.r01")).expect("old style");
        assert_eq!(old.primary, dir.path().join("old.rar"));
        assert_eq!(file_names(&old.volumes), ["old.rar", "old.r00", "old.r01", "old.r03"]);
        assert_eq!(old.missing, ["old.r02"]);

        assert_eq!(detect(&dir.path().join("single.rar")), None);
        assert_eq!(primary(&dir.path().join("single.rar")), dir.path().join("single.rar"));
        assert_eq!(detect(&dir.path().join("lone.7z.001")), None);
        assert_eq!(detect(&dir.path().join("orphan.r00")).map(|s| s.missing), Some(vec!["orphan.rar".to_owned()]));
    }

    #[test]
    fn spanned_zip() {
        let dir = tempfile::tempdir().expect("tempdir");
        touch(dir.path(), &["site.z01", "site.z02", "site.zip", "plain.zip"]);
        let set = detect(&dir.path().join("site.z01")).expect("a set");
        assert_eq!(set.primary, dir.path().join("site.zip"));
        assert_eq!(file_names(&set.volumes), ["site.z01", "site.z02", "site.zip"]);
        assert!(set.is_complete());
        assert_eq!(detect(&dir.path().join("site.zip")).map(|s| s.volumes.len()), Some(3));
        assert_eq!(detect(&dir.path().join("plain.zip")), None);

        std::fs::remove_file(dir.path().join("site.zip")).expect("remove");
        let set = detect(&dir.path().join("site.z02")).expect("a set");
        assert_eq!(set.missing, ["site.zip"]);
    }

    #[test]
    fn collapses_dropped_volumes() {
        let dir = tempfile::tempdir().expect("tempdir");
        touch(dir.path(), &["a.7z.001", "a.7z.002", "a.7z.003", "notes.txt"]);
        let dropped: Vec<PathBuf> =
            ["a.7z.002", "notes.txt", "a.7z.001", "a.7z.003"].iter().map(|n| dir.path().join(n)).collect();
        assert_eq!(collapse(&dropped), [dir.path().join("a.7z.001"), dir.path().join("notes.txt")]);
    }
}
