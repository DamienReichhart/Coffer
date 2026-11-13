//! Typed options for every operation, so a front-end never builds switches by hand.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::version::Version;

/// The archive formats 7-Zip can create.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveFormat {
    #[default]
    SevenZ,
    Zip,
    Tar,
    Gzip,
    Bzip2,
    Xz,
    Wim,
}

impl ArchiveFormat {
    pub const ALL: [Self; 7] = [Self::SevenZ, Self::Zip, Self::Tar, Self::Gzip, Self::Bzip2, Self::Xz, Self::Wim];

    /// The value of 7-Zip's `-t` switch.
    pub fn switch(self) -> &'static str {
        match self {
            Self::SevenZ => "7z",
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::Gzip => "gzip",
            Self::Bzip2 => "bzip2",
            Self::Xz => "xz",
            Self::Wim => "wim",
        }
    }

    /// The usual file extension, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            Self::SevenZ => "7z",
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::Gzip => "gz",
            Self::Bzip2 => "bz2",
            Self::Xz => "xz",
            Self::Wim => "wim",
        }
    }

    /// Stream formats compress exactly one file and hold no folder structure.
    pub fn is_single_file(self) -> bool {
        matches!(self, Self::Gzip | Self::Bzip2 | Self::Xz)
    }

    pub fn supports_encryption(self) -> bool {
        matches!(self, Self::SevenZ | Self::Zip)
    }

    /// Only 7z can hide the file names behind the password.
    pub fn supports_header_encryption(self) -> bool {
        self == Self::SevenZ
    }

    pub fn supports_solid(self) -> bool {
        self == Self::SevenZ
    }

    pub fn supports_compression(self) -> bool {
        !matches!(self, Self::Tar | Self::Wim)
    }

    pub fn supports_multithreading(self) -> bool {
        matches!(self, Self::SevenZ | Self::Zip | Self::Bzip2 | Self::Xz)
    }

    /// The methods this format accepts, the default first.
    pub fn methods(self) -> &'static [CompressionMethod] {
        use CompressionMethod as M;
        match self {
            Self::SevenZ => &[M::Lzma2, M::Lzma, M::Ppmd, M::Bzip2, M::Deflate, M::Copy],
            Self::Zip => &[M::Deflate, M::Deflate64, M::Bzip2, M::Lzma, M::Ppmd, M::Copy],
            Self::Gzip => &[M::Deflate],
            Self::Bzip2 => &[M::Bzip2],
            Self::Xz => &[M::Lzma2],
            Self::Tar | Self::Wim => &[],
        }
    }

    /// Guesses the format from a file name, for "save as" fields.
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        Self::ALL.into_iter().find(|f| f.extension() == ext)
    }
}

impl fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SevenZ => "7z",
            Self::Zip => "ZIP",
            Self::Tar => "TAR",
            Self::Gzip => "GZip",
            Self::Bzip2 => "BZip2",
            Self::Xz => "XZ",
            Self::Wim => "WIM",
        })
    }
}

/// The compression levels 7-Zip's GUI offers, mapped to `-mx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionLevel {
    Store,
    Fastest,
    Fast,
    #[default]
    Normal,
    Maximum,
    Ultra,
}

impl CompressionLevel {
    pub const ALL: [Self; 6] = [Self::Store, Self::Fastest, Self::Fast, Self::Normal, Self::Maximum, Self::Ultra];

    pub fn value(self) -> u8 {
        match self {
            Self::Store => 0,
            Self::Fastest => 1,
            Self::Fast => 3,
            Self::Normal => 5,
            Self::Maximum => 7,
            Self::Ultra => 9,
        }
    }
}

impl fmt::Display for CompressionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Store => "Store (no compression)",
            Self::Fastest => "Fastest",
            Self::Fast => "Fast",
            Self::Normal => "Normal",
            Self::Maximum => "Maximum",
            Self::Ultra => "Ultra",
        })
    }
}

/// Compression methods, mapped to `-m0=` (7z) or `-mm=` (zip).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionMethod {
    Lzma2,
    Lzma,
    Ppmd,
    Bzip2,
    Deflate,
    Deflate64,
    Copy,
}

impl CompressionMethod {
    pub fn switch(self) -> &'static str {
        match self {
            Self::Lzma2 => "LZMA2",
            Self::Lzma => "LZMA",
            Self::Ppmd => "PPMd",
            Self::Bzip2 => "BZip2",
            Self::Deflate => "Deflate",
            Self::Deflate64 => "Deflate64",
            Self::Copy => "Copy",
        }
    }
}

impl fmt::Display for CompressionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.switch())
    }
}

/// Volume sizes for split archives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplitSize {
    /// A size in bytes.
    Bytes(u64),
}

impl SplitSize {
    pub const PRESETS: [(&'static str, u64); 5] = [
        ("10 MB", 10_000_000),
        ("100 MB", 100_000_000),
        ("700 MB (CD)", 700 * 1024 * 1024),
        ("4 GB (FAT32)", 4_092 * 1024 * 1024),
        ("4.7 GB (DVD)", 4_480 * 1024 * 1024),
    ];

    pub fn bytes(self) -> u64 {
        match self {
            Self::Bytes(n) => n,
        }
    }

    /// Parses `500k`, `100m`, `4g`, `1.5g` or a plain byte count.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim().to_ascii_lowercase();
        let input = input.trim_end_matches('b').trim_end_matches('i');
        let (number, multiplier) = match input.chars().last()? {
            'k' => (&input[..input.len() - 1], 1024_f64),
            'm' => (&input[..input.len() - 1], 1024_f64 * 1024.0),
            'g' => (&input[..input.len() - 1], 1024_f64 * 1024.0 * 1024.0),
            _ => (input, 1.0),
        };
        let value: f64 = number.trim().parse().ok()?;
        let bytes = (value * multiplier).round();
        (bytes >= 1.0 && bytes.is_finite()).then_some(Self::Bytes(bytes as u64))
    }
}

/// What to do when an extracted file already exists (`-ao`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverwriteMode {
    /// Replace existing files (`-aoa`).
    Overwrite,
    /// Keep existing files (`-aos`).
    #[default]
    Skip,
    /// Rename the extracted file (`-aou`).
    RenameNew,
    /// Rename the existing file (`-aot`).
    RenameExisting,
}

impl OverwriteMode {
    pub const ALL: [Self; 4] = [Self::Overwrite, Self::Skip, Self::RenameNew, Self::RenameExisting];

    pub fn switch(self) -> &'static str {
        match self {
            Self::Overwrite => "-aoa",
            Self::Skip => "-aos",
            Self::RenameNew => "-aou",
            Self::RenameExisting => "-aot",
        }
    }
}

impl fmt::Display for OverwriteMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Overwrite => "Overwrite existing files",
            Self::Skip => "Skip existing files",
            Self::RenameNew => "Rename extracted files",
            Self::RenameExisting => "Rename existing files",
        })
    }
}

/// Options for creating or updating an archive.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompressOptions {
    pub format: ArchiveFormat,
    pub level: CompressionLevel,
    /// `None` keeps the format's default method.
    pub method: Option<CompressionMethod>,
    /// Dictionary size in MiB, `None` for 7-Zip's default for the level.
    pub dictionary_mib: Option<u32>,
    /// Solid compression (7z only). 7-Zip's default is on.
    pub solid: bool,
    /// Worker threads, `None` lets 7-Zip decide.
    pub threads: Option<u32>,
    pub password: Option<String>,
    /// Encrypt file names too (7z only).
    pub encrypt_names: bool,
    /// Use AES-256 for zip instead of the weak legacy `ZipCrypto`.
    pub zip_aes: bool,
    pub split: Option<SplitSize>,
    /// Delete the source files once they are safely in the archive.
    pub delete_after: bool,
    /// Wildcards to leave out, applied recursively (`-xr!`).
    pub excludes: Vec<String>,
}

impl CompressOptions {
    /// Checks the options against the format and the inputs, returning a
    /// message for the user when they cannot work.
    pub fn validate(&self, inputs: &[PathBuf]) -> Result<(), String> {
        if inputs.is_empty() {
            return Err("Add at least one file or folder.".into());
        }
        if self.format.is_single_file() && (inputs.len() > 1 || inputs.iter().any(|p| p.is_dir())) {
            return Err(format!(
                "{} compresses a single file. Choose 7z, ZIP or TAR for several files or folders.",
                self.format
            ));
        }
        if let Some(method) = self.method
            && !self.format.methods().contains(&method)
        {
            return Err(format!("{} archives can't use {method}.", self.format));
        }
        if self.password.as_deref().is_some_and(|p| !p.is_empty()) && !self.format.supports_encryption() {
            return Err(format!("{} archives can't be encrypted.", self.format));
        }
        if self.split.is_some_and(|s| s.bytes() < 64 * 1024) {
            return Err("Volumes must be at least 64 KiB.".into());
        }
        Ok(())
    }
}

/// Options for extracting an archive.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtractOptions {
    pub destination: PathBuf,
    pub overwrite: OverwriteMode,
    /// Recreate the folders stored in the archive (`x`) or put every file
    /// directly in the destination (`e`).
    pub keep_paths: bool,
    pub password: Option<String>,
    /// Paths inside the archive to extract; empty extracts everything.
    pub only: Vec<String>,
}

/// Checksum algorithms of `7z h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Crc32,
    Crc64,
    Sha1,
    Sha256,
    Blake2sp,
    Xxh64,
    Md5,
    Sha512,
}

impl HashAlgorithm {
    pub const ALL: [Self; 8] =
        [Self::Crc32, Self::Crc64, Self::Md5, Self::Sha1, Self::Sha256, Self::Sha512, Self::Blake2sp, Self::Xxh64];

    pub fn switch(self) -> &'static str {
        match self {
            Self::Crc32 => "CRC32",
            Self::Crc64 => "CRC64",
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Blake2sp => "BLAKE2sp",
            Self::Xxh64 => "XXH64",
            Self::Md5 => "MD5",
            Self::Sha512 => "SHA512",
        }
    }

    /// Whether the given 7-Zip version can compute this hash.
    pub fn supported_by(self, version: Version) -> bool {
        match self {
            Self::Crc32 | Self::Crc64 | Self::Sha1 | Self::Sha256 | Self::Blake2sp => true,
            Self::Xxh64 | Self::Md5 | Self::Sha512 => version.at_least(24, 0),
        }
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Sha1 => "SHA-1",
            Self::Sha256 => "SHA-256",
            Self::Sha512 => "SHA-512",
            other => other.switch(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_round_trip_through_extensions() {
        for format in ArchiveFormat::ALL {
            let path = PathBuf::from(format!("file.{}", format.extension()));
            assert_eq!(ArchiveFormat::from_path(&path), Some(format));
        }
        assert_eq!(ArchiveFormat::from_path(std::path::Path::new("a.txt")), None);
    }

    #[test]
    fn every_format_lists_its_default_method_first() {
        assert_eq!(ArchiveFormat::SevenZ.methods()[0], CompressionMethod::Lzma2);
        assert_eq!(ArchiveFormat::Zip.methods()[0], CompressionMethod::Deflate);
        assert!(ArchiveFormat::Tar.methods().is_empty());
    }

    #[test]
    fn split_sizes_parse() {
        assert_eq!(SplitSize::parse("100m"), Some(SplitSize::Bytes(100 * 1024 * 1024)));
        assert_eq!(SplitSize::parse("1.5G"), Some(SplitSize::Bytes(1_610_612_736)));
        assert_eq!(SplitSize::parse("500 KiB"), Some(SplitSize::Bytes(512_000)));
        assert_eq!(SplitSize::parse("4096"), Some(SplitSize::Bytes(4096)));
        assert_eq!(SplitSize::parse("nope"), None);
        assert_eq!(SplitSize::parse("0"), None);
    }

    #[test]
    fn validation_catches_impossible_combinations() {
        let files = vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")];
        let gzip = CompressOptions { format: ArchiveFormat::Gzip, ..Default::default() };
        assert!(gzip.validate(&files).is_err());
        assert!(gzip.validate(&files[..1]).is_ok());

        let tar = CompressOptions { format: ArchiveFormat::Tar, password: Some("x".into()), ..Default::default() };
        assert!(tar.validate(&files).is_err());

        let zip = CompressOptions {
            format: ArchiveFormat::Zip,
            method: Some(CompressionMethod::Lzma2),
            ..Default::default()
        };
        assert!(zip.validate(&files).is_err());
        assert!(CompressOptions::default().validate(&[]).is_err());
    }

    #[test]
    fn hash_support_depends_on_the_version() {
        assert!(HashAlgorithm::Sha256.supported_by(Version::new(16, 2)));
        assert!(!HashAlgorithm::Md5.supported_by(Version::new(23, 1)));
        assert!(HashAlgorithm::Md5.supported_by(Version::new(24, 8)));
    }
}
