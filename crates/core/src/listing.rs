//! Parsing `7z l -slt` and browsing the result as a folder tree.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{Error, Result};

/// Properties of the archive itself (the block before `----------`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArchiveInfo {
    pub path: PathBuf,
    /// 7-Zip's name of the format: `7z`, `zip`, `tar`, `Rar5`…
    pub kind: String,
    pub physical_size: Option<u64>,
    /// The size of every volume together, for a multi-volume archive.
    pub total_physical_size: Option<u64>,
    pub method: Option<String>,
    pub solid: Option<bool>,
    pub blocks: Option<u64>,
    pub volumes: Option<u64>,
    pub comment: Option<String>,
    /// Whether some entry is encrypted.
    pub encrypted: bool,
    /// Warnings 7-Zip printed while opening (e.g. trailing data).
    pub warnings: Vec<String>,
}

/// One item of an archive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entry {
    /// The path inside the archive, always with `/` separators.
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    /// The packed size. Solid archives only report it on the first file of a block.
    pub packed: Option<u64>,
    /// `YYYY-MM-DD HH:MM:SS`, as 7-Zip reports it.
    pub modified: Option<String>,
    pub crc: Option<String>,
    pub method: Option<String>,
    pub encrypted: bool,
    pub attributes: Option<String>,
}

impl Entry {
    /// The last component of the path.
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }
}

/// The parsed output of `7z l -slt`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Listing {
    pub info: ArchiveInfo,
    pub entries: Vec<Entry>,
}

impl Listing {
    /// Parses the technical listing (`-slt`).
    pub fn parse(output: &str) -> Result<Self> {
        let mut info = ArchiveInfo::default();
        let mut entries = Vec::new();
        let mut in_entries = false;
        let mut in_header = false;
        let mut current: Option<Entry> = None;

        for raw in output.lines() {
            let line = raw.trim_end_matches('\r');
            if line.starts_with("----------") {
                in_entries = true;
                continue;
            }
            if line == "--" {
                in_header = true;
                continue;
            }
            if !in_entries {
                if line.starts_with("WARNING") || line.starts_with("Warnings:") {
                    info.warnings.push(line.trim().to_owned());
                }
                if !in_header {
                    continue;
                }
                if let Some((key, value)) = split(line) {
                    apply_info(&mut info, key, value);
                }
                continue;
            }
            if line.is_empty() {
                if let Some(entry) = current.take() {
                    entries.push(entry);
                }
                continue;
            }
            let Some((key, value)) = split(line) else { continue };
            if key == "Path" {
                if let Some(entry) = current.take() {
                    entries.push(entry);
                }
                current = Some(Entry { path: normalize(value), ..Entry::default() });
                continue;
            }
            if let Some(entry) = current.as_mut() {
                apply_entry(entry, key, value);
            }
        }
        if let Some(entry) = current.take() {
            entries.push(entry);
        }
        if !in_entries && info.kind.is_empty() {
            return Err(Error::Parse("no archive listing in the output".into()));
        }
        info.encrypted = entries.iter().any(|e| e.encrypted);
        Ok(Self { info, entries })
    }

    /// Sum of the uncompressed sizes of every file.
    pub fn total_size(&self) -> u64 {
        self.entries.iter().filter(|e| !e.is_dir).map(|e| e.size).sum()
    }

    /// The size of the archive on disk: every volume for a multi-volume archive.
    pub fn archive_size(&self) -> Option<u64> {
        self.info.total_physical_size.or(self.info.physical_size)
    }

    pub fn file_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.is_dir).count()
    }
}

fn split(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(" = ").or_else(|| {
        // `CRC = ` with an empty value has no trailing space after trimming.
        line.strip_suffix(" =").map(|key| (key, ""))
    })?;
    Some((key.trim(), value))
}

fn flag(value: &str) -> Option<bool> {
    match value.trim() {
        "+" => Some(true),
        "-" => Some(false),
        _ => None,
    }
}

fn number(value: &str) -> Option<u64> {
    value.trim().parse().ok()
}

fn text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn normalize(path: &str) -> String {
    path.replace('\\', "/").trim_matches('/').to_owned()
}

fn apply_info(info: &mut ArchiveInfo, key: &str, value: &str) {
    match key {
        "Path" if info.path.as_os_str().is_empty() => info.path = PathBuf::from(value),
        "Type" => value.trim().clone_into(&mut info.kind),
        "Physical Size" => info.physical_size = number(value),
        "Total Physical Size" => info.total_physical_size = number(value),
        // An error 7-Zip worked around while opening, e.g. `Missing volume : a.z05`.
        "ERROR" | "WARNING" => {
            if let Some(message) = text(value).filter(|m| !info.warnings.contains(m)) {
                info.warnings.push(message);
            }
        }
        "Method" => info.method = text(value),
        "Solid" => info.solid = flag(value),
        "Blocks" => info.blocks = number(value),
        "Volumes" => info.volumes = number(value),
        "Comment" => info.comment = text(value),
        _ => {}
    }
}

fn apply_entry(entry: &mut Entry, key: &str, value: &str) {
    match key {
        "Folder" => entry.is_dir = flag(value).unwrap_or(entry.is_dir),
        "Size" => entry.size = number(value).unwrap_or(0),
        "Packed Size" => entry.packed = number(value),
        "Modified" => entry.modified = text(value).map(|t| t.chars().take(19).collect()),
        "CRC" => entry.crc = text(value),
        "Method" => entry.method = text(value),
        "Encrypted" => entry.encrypted = flag(value).unwrap_or(false),
        "Attributes" => {
            // 7z archives have no `Folder` line: a leading `D` marks a directory.
            if value.trim_start().starts_with('D') {
                entry.is_dir = true;
            }
            entry.attributes = text(value);
        }
        _ => {}
    }
}

/// A file or folder shown while browsing an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    /// For a folder, the total size of everything inside it.
    pub size: u64,
    pub packed: Option<u64>,
    pub modified: Option<String>,
    pub method: Option<String>,
    pub crc: Option<String>,
    pub encrypted: bool,
    /// For a folder, how many files it holds (recursively).
    pub files: u64,
}

/// The entries of a listing arranged as folders, including folders that the
/// archive only implies (zip files often store `a/b.txt` without `a/`).
#[derive(Debug, Clone, Default)]
pub struct Tree {
    nodes: Vec<Node>,
    by_path: HashMap<String, usize>,
    children: HashMap<String, Vec<usize>>,
}

impl Tree {
    pub fn new(entries: &[Entry]) -> Self {
        let mut tree = Self::default();
        tree.children.insert(String::new(), Vec::new());
        for entry in entries.iter().filter(|e| !e.path.is_empty()) {
            tree.insert(entry);
        }
        // Folder totals.
        let files: Vec<(String, u64, Option<u64>)> =
            tree.nodes.iter().filter(|n| !n.is_dir).map(|n| (n.path.clone(), n.size, n.packed)).collect();
        for (path, size, packed) in files {
            let mut dir = parent(&path);
            while let Some(d) = dir {
                if let Some(&i) = tree.by_path.get(d) {
                    let node = &mut tree.nodes[i];
                    node.size += size;
                    node.files += 1;
                    if let Some(p) = packed {
                        node.packed = Some(node.packed.unwrap_or(0) + p);
                    }
                }
                dir = parent(d);
            }
        }
        tree
    }

    fn insert(&mut self, entry: &Entry) {
        if let Some(dir) = parent(&entry.path) {
            self.ensure_dir(dir);
        }
        let node = Node {
            path: entry.path.clone(),
            name: entry.name().to_owned(),
            is_dir: entry.is_dir,
            size: if entry.is_dir { 0 } else { entry.size },
            packed: if entry.is_dir { None } else { entry.packed },
            modified: entry.modified.clone(),
            method: entry.method.clone(),
            crc: entry.crc.clone(),
            encrypted: entry.encrypted,
            files: 0,
        };
        if let Some(&i) = self.by_path.get(&entry.path) {
            // An explicit folder entry after its implied one: keep the details.
            let existing = &mut self.nodes[i];
            existing.modified = node.modified;
            existing.encrypted |= node.encrypted;
            return;
        }
        let index = self.nodes.len();
        let parent_key = parent(&entry.path).unwrap_or("").to_owned();
        self.nodes.push(node);
        self.by_path.insert(entry.path.clone(), index);
        self.children.entry(parent_key).or_default().push(index);
        if entry.is_dir {
            self.children.entry(entry.path.clone()).or_default();
        }
    }

    fn ensure_dir(&mut self, dir: &str) {
        if self.by_path.contains_key(dir) {
            return;
        }
        if let Some(up) = parent(dir) {
            self.ensure_dir(up);
        }
        let index = self.nodes.len();
        self.nodes.push(Node {
            path: dir.to_owned(),
            name: dir.rsplit('/').next().unwrap_or(dir).to_owned(),
            is_dir: true,
            size: 0,
            packed: None,
            modified: None,
            method: None,
            crc: None,
            encrypted: false,
            files: 0,
        });
        self.by_path.insert(dir.to_owned(), index);
        self.children.entry(parent(dir).unwrap_or("").to_owned()).or_default().push(index);
        self.children.entry(dir.to_owned()).or_default();
    }

    /// The items directly inside `dir` (`""` is the root), folders first,
    /// then by name.
    pub fn children(&self, dir: &str) -> Vec<&Node> {
        let mut items: Vec<&Node> =
            self.children.get(dir).map(|ids| ids.iter().map(|&i| &self.nodes[i]).collect()).unwrap_or_default();
        items.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| natural_cmp(&a.name, &b.name)));
        items
    }

    pub fn get(&self, path: &str) -> Option<&Node> {
        self.by_path.get(path).map(|&i| &self.nodes[i])
    }

    pub fn is_dir(&self, path: &str) -> bool {
        path.is_empty() || self.get(path).is_some_and(|n| n.is_dir)
    }

    /// Every node whose name contains `query` (case-insensitive), anywhere.
    pub fn search(&self, query: &str) -> Vec<&Node> {
        let query = query.to_lowercase();
        let mut found: Vec<&Node> = self.nodes.iter().filter(|n| n.name.to_lowercase().contains(&query)).collect();
        found.sort_by(|a, b| natural_cmp(&a.path, &b.path));
        found
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// The folder holding `path`, `None` at the root.
pub fn parent(path: &str) -> Option<&str> {
    path.rfind('/').map(|i| &path[..i])
}

/// Compares names the way people expect: `file2` before `file10`, ignoring case.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut a = a.chars().peekable();
    let mut b = b.chars().peekable();
    loop {
        match (a.peek(), b.peek()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars<'_>>| {
                    let mut digits = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        digits.push(c);
                        it.next();
                    }
                    digits
                };
                let (da, db) = (take(&mut a), take(&mut b));
                let (ta, tb) = (da.trim_start_matches('0'), db.trim_start_matches('0'));
                let order = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
                if order != Ordering::Equal {
                    return order;
                }
            }
            (Some(x), Some(y)) => {
                let order = x.to_lowercase().cmp(y.to_lowercase());
                if order != Ordering::Equal {
                    return order;
                }
                a.next();
                b.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEVEN_Z: &str = "\n7-Zip 23.01 (x64) : Copyright (c) 1999-2023 Igor Pavlov : 2023-06-20\n\nScanning the drive for archives:\n1 file, 300257 bytes (294 KiB)\n\nListing archive: arc.7z\n\n--\nPath = arc.7z\nType = 7z\nPhysical Size = 300257\nHeaders Size = 223\nMethod = LZMA2:384k\nSolid = +\nBlocks = 1\n\n----------\nPath = src\nSize = 0\nPacked Size = 0\nModified = 2026-11-12 20:15:36.4032595\nAttributes = D drwxr-xr-x\nCRC = \nEncrypted = -\nMethod = \nBlock = \n\nPath = src/a.txt\nSize = 6\nPacked Size = 300034\nModified = 2026-11-12 20:15:36.4032595\nAttributes = A -rw-r--r--\nCRC = 363A3020\nEncrypted = -\nMethod = LZMA2:384k\nBlock = 0\n\nPath = src/sub/b.bin\nSize = 300000\nPacked Size = \nModified = 2026-11-12 20:15:36.4076740\nAttributes = A -rw-r--r--\nCRC = 092B3AD0\nEncrypted = -\nMethod = LZMA2:384k\nBlock = 0\n\nPath = src/sub/naïve file.txt\nSize = 6\nPacked Size = \nModified = 2026-11-12 20:15:36.4076740\nAttributes = A -rw-r--r--\nCRC = C0F7BFD3\nEncrypted = -\nMethod = LZMA2:384k\nBlock = 0\n\n";

    const ZIP: &str = "--\nPath = C:\\x\\enc.zip\nType = zip\nPhysical Size = 300744\n\n----------\nPath = src\\a.txt\nFolder = -\nSize = 6\nPacked Size = 18\nModified = 2026-11-12 20:15:36\nAttributes =  -rw-r--r--\nEncrypted = +\nComment = \nCRC = 363A3020\nMethod = ZipCrypto Store\n";

    #[test]
    fn parses_a_7z_listing() {
        let listing = Listing::parse(SEVEN_Z).expect("parse");
        assert_eq!(listing.info.kind, "7z");
        assert_eq!(listing.info.physical_size, Some(300_257));
        assert_eq!(listing.info.solid, Some(true));
        assert_eq!(listing.info.method.as_deref(), Some("LZMA2:384k"));
        assert_eq!(listing.entries.len(), 4);
        assert!(listing.entries[0].is_dir);
        let a = &listing.entries[1];
        assert_eq!(a.path, "src/a.txt");
        assert_eq!(a.size, 6);
        assert_eq!(a.packed, Some(300_034));
        assert_eq!(a.modified.as_deref(), Some("2026-11-12 20:15:36"));
        assert_eq!(a.crc.as_deref(), Some("363A3020"));
        assert_eq!(listing.entries[3].name(), "naïve file.txt");
        assert_eq!(listing.total_size(), 300_012);
        assert_eq!(listing.file_count(), 3);
        assert!(!listing.info.encrypted);
    }

    #[test]
    fn reads_the_size_of_every_volume() {
        let out = "Listing archive: span.zip\n\n--\nPath = span.zip\nType = zip\n\
                   ERROR = Missing volume : span.z05\nPhysical Size = 236139\nTotal Physical Size = 2693739\n\
                   Multivolume = +\nVolume Index = 9\nVolumes = 10\n\n----------\nPath = a.txt\nFolder = -\n\
                   Size = 3\n\n";
        let listing = Listing::parse(out).expect("parse");
        assert_eq!(listing.info.volumes, Some(10));
        assert_eq!(listing.archive_size(), Some(2_693_739));
        assert_eq!(listing.info.warnings, ["Missing volume : span.z05"]);
    }

    #[test]
    fn parses_windows_paths_and_encryption() {
        let listing = Listing::parse(ZIP).expect("parse");
        assert_eq!(listing.entries[0].path, "src/a.txt");
        assert!(listing.entries[0].encrypted);
        assert!(listing.info.encrypted);
        assert_eq!(listing.info.path, PathBuf::from("C:\\x\\enc.zip"));
    }

    #[test]
    fn rejects_output_without_a_listing() {
        assert!(Listing::parse("ERROR: something").is_err());
    }

    #[test]
    fn builds_a_tree_with_implied_folders() {
        let listing = Listing::parse(ZIP).expect("parse");
        let tree = Tree::new(&listing.entries);
        let root = tree.children("");
        assert_eq!(root.len(), 1);
        assert_eq!(root[0].name, "src");
        assert!(root[0].is_dir);
        assert_eq!(root[0].files, 1);
        assert_eq!(tree.children("src")[0].name, "a.txt");
    }

    #[test]
    fn folder_totals_and_ordering() {
        let listing = Listing::parse(SEVEN_Z).expect("parse");
        let tree = Tree::new(&listing.entries);
        let src = tree.children("");
        assert_eq!(src.len(), 1);
        assert_eq!(src[0].size, 300_012);
        assert_eq!(src[0].files, 3);
        let names: Vec<&str> = tree.children("src").iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, ["sub", "a.txt"]);
        assert_eq!(tree.search("NAÏVE").len(), 1);
        assert!(tree.is_dir("src/sub"));
        assert!(!tree.is_dir("src/a.txt"));
    }

    #[test]
    fn natural_ordering() {
        let mut names = vec!["file10", "File2", "file1", "a", "file02b"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["a", "file1", "File2", "file02b", "file10"]);
    }
}
