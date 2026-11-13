//! Everything Coffer needs to drive a 7-Zip command-line executable.
//!
//! 7-Zip itself is never bundled: [`locate`] finds the copy the user
//! installed, [`command`] turns typed options into a command line, [`runner`]
//! runs it with live progress, and [`listing`], [`progress`] and [`hash`] turn
//! 7-Zip's console output back into data.
//!
//! Nothing in this crate depends on a UI toolkit, so it can back a CLI, a
//! file-manager extension or tests just as well as the desktop app.

pub mod command;
pub mod error;
pub mod format;
pub mod hash;
pub mod install;
pub mod listing;
pub mod locate;
pub mod options;
pub mod progress;
pub mod runner;
pub mod version;
pub mod volumes;

pub use error::{Error, Result};
pub use listing::{ArchiveInfo, Entry, Listing};
pub use locate::{Flavor, SevenZip};
pub use options::{
    ArchiveFormat, CompressOptions, CompressionLevel, CompressionMethod, ExtractOptions, HashAlgorithm, OverwriteMode,
    SplitSize,
};
pub use runner::{Event, Outcome};
pub use version::Version;
