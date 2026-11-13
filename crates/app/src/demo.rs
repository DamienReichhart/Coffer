//! Realistic sample states for the generated screenshots and UI tests.
//!
//! Nothing here touches the file system or runs 7-Zip: every scene is built
//! from plain data, so screenshots are reproducible on any machine.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use coffer_core::listing::{ArchiveInfo, Entry};
use coffer_core::{HashAlgorithm, Listing, SevenZip, Version};

use crate::app::{App, Screen, SevenState};
use crate::jobs::{Job, Spec, Status};
use crate::screens::browse::Browser;
use crate::settings::Settings;
use crate::theme::ThemeChoice;

/// The screens the website shows, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    Welcome,
    Browse,
    Compress,
    Extract,
    Activity,
    Checksums,
    Settings,
}

impl Scene {
    pub const ALL: [Self; 7] =
        [Self::Welcome, Self::Browse, Self::Compress, Self::Extract, Self::Activity, Self::Checksums, Self::Settings];

    /// The file name (without extension) the website expects.
    pub fn name(self) -> &'static str {
        match self {
            Self::Welcome => "welcome",
            Self::Browse => "browse",
            Self::Compress => "compress",
            Self::Extract => "extract",
            Self::Activity => "activity",
            Self::Checksums => "checksums",
            Self::Settings => "settings",
        }
    }
}

fn home() -> PathBuf {
    if cfg!(windows) { PathBuf::from(r"C:\Users\alex") } else { PathBuf::from("/home/alex") }
}

fn seven() -> SevenZip {
    SevenZip {
        path: if cfg!(windows) {
            PathBuf::from(r"C:\Program Files\7-Zip\7z.exe")
        } else {
            PathBuf::from("/usr/bin/7zz")
        },
        version: Version::new(24, 9),
        flavor: coffer_core::Flavor::Official,
    }
}

fn entry(path: &str, size: u64, packed: Option<u64>, modified: &str) -> Entry {
    Entry {
        path: path.to_owned(),
        is_dir: false,
        size,
        packed,
        modified: Some(modified.to_owned()),
        crc: Some("A3F1C2D9".to_owned()),
        method: Some("LZMA2:24".to_owned()),
        encrypted: false,
        attributes: None,
    }
}

fn dir(path: &str, modified: &str) -> Entry {
    Entry { path: path.to_owned(), is_dir: true, modified: Some(modified.to_owned()), ..Entry::default() }
}

/// A plausible project archive.
pub fn listing() -> Listing {
    let entries = vec![
        dir("website-redesign", "2026-10-28 18:42:10"),
        dir("website-redesign/assets", "2026-10-28 18:40:03"),
        dir("website-redesign/assets/photos", "2026-10-27 09:12:44"),
        dir("website-redesign/docs", "2026-10-26 16:20:51"),
        dir("website-redesign/src", "2026-10-28 18:41:37"),
        entry("website-redesign/assets/photos/hero.jpg", 4_812_334, Some(4_700_112), "2026-10-27 09:12:44"),
        entry("website-redesign/assets/photos/team.jpg", 3_201_882, None, "2026-10-27 09:12:40"),
        entry("website-redesign/assets/logo.svg", 12_402, None, "2026-10-22 11:03:19"),
        entry("website-redesign/assets/fonts.woff2", 98_310, None, "2026-10-20 08:55:00"),
        entry("website-redesign/docs/brief.pdf", 1_942_118, Some(1_610_221), "2026-10-26 16:20:51"),
        entry("website-redesign/docs/meeting-notes.md", 18_220, None, "2026-10-25 10:02:33"),
        entry("website-redesign/src/index.html", 24_118, Some(5_102), "2026-10-28 18:41:37"),
        entry("website-redesign/src/styles.css", 61_904, None, "2026-10-28 18:40:12"),
        entry("website-redesign/src/app.js", 142_877, None, "2026-10-28 18:39:58"),
        entry("website-redesign/README.md", 3_412, None, "2026-10-28 18:42:10"),
        entry("website-redesign/budget-2027.xlsx", 88_120, None, "2026-10-24 14:31:09"),
        entry("website-redesign/contract-signed.pdf", 512_336, None, "2026-10-19 17:45:21"),
        entry("website-redesign/roadmap.key", 6_410_552, Some(5_902_120), "2026-10-27 20:14:02"),
    ];
    Listing {
        info: ArchiveInfo {
            path: home().join("Documents").join("website-redesign.7z"),
            kind: "7z".to_owned(),
            physical_size: Some(14_870_112),
            total_physical_size: None,
            method: Some("LZMA2:24 BCJ".to_owned()),
            solid: Some(true),
            blocks: Some(2),
            volumes: None,
            comment: None,
            encrypted: false,
            warnings: Vec::new(),
        },
        entries,
    }
}

fn job(id: u64, spec: Spec, status: Status, percent: u8, current: Option<&str>, elapsed: u64) -> Job {
    let mut job = Job::new(id, spec);
    job.status = status;
    job.percent = percent;
    job.current = current.map(ToOwned::to_owned);
    job.started = Instant::now().checked_sub(Duration::from_secs(elapsed)).unwrap_or_else(Instant::now);
    if !job.is_running() {
        job.elapsed = Some(Duration::from_secs(elapsed));
    }
    job
}

/// The app in the given scene and theme.
pub fn app(scene: Scene, dark: bool) -> App {
    let documents = home().join("Documents");
    let mut settings =
        Settings { theme: if dark { ThemeChoice::Dark } else { ThemeChoice::Light }, ..Settings::default() };
    settings.recent = vec![
        documents.join("website-redesign.7z"),
        home().join("Downloads").join("photos-2026-summer.zip"),
        home().join("Backups").join("thesis-final.tar.xz"),
        home().join("Downloads").join("dataset-v3.7z.001"),
    ];
    let mut app = App::with_settings(settings);
    app.seven = SevenState::Ready(seven());
    app.system_dark = dark;
    let archive = documents.join("website-redesign.7z");

    match scene {
        Scene::Welcome => app.screen = Screen::Home,
        Scene::Browse => {
            let mut browser = Browser::new(archive, listing(), None);
            browser.selection.insert("website-redesign/docs".to_owned());
            app.browser = Some(browser);
            app.screen = Screen::Browse;
        }
        Scene::Compress => {
            app.compress.add_inputs(vec![
                home().join("Pictures").join("Iceland 2026"),
                home().join("Pictures").join("itinerary.pdf"),
                home().join("Pictures").join("budget.ods"),
            ]);
            "correct horse battery staple".clone_into(&mut app.compress.password);
            app.compress.confirm.clone_from(&app.compress.password);
            app.compress.level = coffer_core::CompressionLevel::Maximum;
            app.screen = Screen::Compress;
        }
        Scene::Extract => {
            app.extract.set_archive(home().join("Downloads").join("photos-2026-summer.zip"), &app.settings.extract);
            app.screen = Screen::Extract;
        }
        Scene::Activity => {
            app.browser = Some(Browser::new(archive.clone(), listing(), None));
            app.jobs.list = vec![
                job(
                    4,
                    Spec::Compress {
                        archive: home().join("Pictures").join("Iceland 2026.7z"),
                        inputs: vec![home().join("Pictures").join("Iceland 2026")],
                        options: coffer_core::CompressOptions::default(),
                    },
                    Status::Running,
                    62,
                    Some("Iceland 2026/DSC_0412.NEF"),
                    48,
                ),
                job(
                    3,
                    Spec::Extract {
                        archive: home().join("Downloads").join("photos-2026-summer.zip"),
                        options: coffer_core::ExtractOptions {
                            destination: home().join("Downloads").join("photos-2026-summer"),
                            ..Default::default()
                        },
                        open_when_done: true,
                    },
                    Status::Running,
                    27,
                    Some("summer/beach/IMG_2210.HEIC"),
                    12,
                ),
                job(
                    2,
                    Spec::Test { archive: archive.clone(), password: None },
                    Status::Succeeded { warnings: false },
                    100,
                    None,
                    3,
                ),
                job(
                    1,
                    Spec::Extract {
                        archive: home().join("Backups").join("thesis-final.tar.xz"),
                        options: coffer_core::ExtractOptions {
                            destination: home().join("Documents").join("thesis-final"),
                            ..Default::default()
                        },
                        open_when_done: false,
                    },
                    Status::Succeeded { warnings: false },
                    100,
                    None,
                    21,
                ),
            ];
            app.screen = Screen::Activity;
        }
        Scene::Checksums => {
            let file = home().join("Downloads").join("ubuntu-26.04-desktop-amd64.iso");
            app.checksums.files = vec![file];
            app.checksums.algorithms = vec![HashAlgorithm::Crc32, HashAlgorithm::Sha256];
            "9f2e81c0a5e4c7b16a3a6e2b1c0b8f4d6e0e1f3a7c9d2b5e8f1a4c7d0e3b6a9f".clone_into(&mut app.checksums.compare);
            app.checksums.results = vec![coffer_core::hash::FileHashes {
                path: "ubuntu-26.04-desktop-amd64.iso".to_owned(),
                size: Some(6_114_656_256),
                digests: vec![
                    ("CRC32".to_owned(), "7D3C21A9".to_owned()),
                    (
                        "SHA256".to_owned(),
                        "9f2e81c0a5e4c7b16a3a6e2b1c0b8f4d6e0e1f3a7c9d2b5e8f1a4c7d0e3b6a9f".to_owned(),
                    ),
                ],
            }];
            app.screen = Screen::Checksums;
        }
        Scene::Settings => app.screen = Screen::Settings,
    }
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_builds_and_renders() {
        for scene in Scene::ALL {
            for dark in [false, true] {
                let app = app(scene, dark);
                let _ = app.view();
                assert_eq!(app.is_dark(), dark);
            }
        }
    }
}
