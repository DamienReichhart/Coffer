//! The Settings screen.

use std::path::PathBuf;

use iced::widget::{button, column, row, text};
use iced::{Alignment, Task};
use sevenzip_core::{ArchiveFormat, CompressionLevel, OverwriteMode};

use crate::app::{App, Message, SevenState};
use crate::icons::Icon;
use crate::settings::{Settings, paths};
use crate::theme::{self, ThemeChoice, Tone};
use crate::ui::{self, El};
use crate::update::{self, State};

#[derive(Debug, Clone)]
pub enum Msg {
    Theme(ThemeChoice),
    SevenZipInput(String),
    SevenZipPath(String),
    BrowseSevenZip,
    ClearSevenZip,
    Format(ArchiveFormat),
    Level(CompressionLevel),
    Solid(bool),
    EncryptNames(bool),
    ZipAes(bool),
    Overwrite(OverwriteMode),
    KeepPaths(bool),
    IntoFolder(bool),
    OpenWhenDone(bool),
    ConfirmDelete(bool),
    AutoUpdate(bool),
    SkipVersion(String),
    ClearRecent,
    OpenConfigFolder,
    OpenLogFolder,
    Reset,
}

impl App {
    pub(crate) fn update_preferences(&mut self, msg: Msg) -> Task<Message> {
        let mut relocate = false;
        let s = &mut self.settings;
        match msg {
            Msg::Theme(choice) => s.theme = choice,
            Msg::SevenZipInput(text) => {
                self.seven_path_input = text;
                return Task::none();
            }
            Msg::SevenZipPath(path) => {
                self.seven_path_input.clone_from(&path);
                s.seven_zip_path = (!path.trim().is_empty()).then(|| PathBuf::from(path.trim()));
                relocate = true;
            }
            Msg::BrowseSevenZip => {
                return Task::perform(
                    rfd::AsyncFileDialog::new().set_title("Choose the 7-Zip executable (7z, 7zz or 7za)").pick_file(),
                    |file| {
                        file.map_or(Message::Nothing, |f| {
                            Message::Preferences(Msg::SevenZipPath(f.path().display().to_string()))
                        })
                    },
                );
            }
            Msg::ClearSevenZip => {
                self.seven_path_input.clear();
                s.seven_zip_path = None;
                relocate = true;
            }
            Msg::Format(format) => s.compress.format = format,
            Msg::Level(level) => s.compress.level = level,
            Msg::Solid(on) => s.compress.solid = on,
            Msg::EncryptNames(on) => s.compress.encrypt_names = on,
            Msg::ZipAes(on) => s.compress.zip_aes = on,
            Msg::Overwrite(mode) => s.extract.overwrite = mode,
            Msg::KeepPaths(on) => s.extract.keep_paths = on,
            Msg::IntoFolder(on) => s.extract.into_folder = on,
            Msg::OpenWhenDone(on) => s.extract.open_when_done = on,
            Msg::ConfirmDelete(on) => s.confirm_delete = on,
            Msg::AutoUpdate(on) => s.updates.check_automatically = on,
            Msg::SkipVersion(version) => {
                s.updates.skipped_version = Some(version);
                self.updater.dismissed = true;
            }
            Msg::ClearRecent => s.recent.clear(),
            Msg::OpenConfigFolder => {
                if let Some(dir) = paths::config_dir() {
                    let _ = std::fs::create_dir_all(&dir);
                    return Task::done(Message::Reveal(dir));
                }
            }
            Msg::OpenLogFolder => {
                if let Some(dir) = paths::log_dir() {
                    return Task::done(Message::Reveal(dir));
                }
            }
            Msg::Reset => {
                let recent = std::mem::take(&mut s.recent);
                *s = Settings { recent, ..Settings::default() };
                relocate = true;
            }
        }
        self.save_settings();
        if relocate { self.update(Message::Relocate) } else { Task::none() }
    }
}

pub fn view(app: &App) -> El<'_> {
    let s = &app.settings;
    let header = ui::page_header(
        "Settings",
        "Saved automatically.",
        vec![ui::ghost(Some(Icon::RotateCcw), "Restore defaults", Some(Message::Preferences(Msg::Reset))).into()],
    );

    let themes = row(ThemeChoice::ALL.iter().map(|choice| {
        let (i, label) = match choice {
            ThemeChoice::System => (Icon::Monitor, "System"),
            ThemeChoice::Light => (Icon::Sun, "Light"),
            ThemeChoice::Dark => (Icon::Moon, "Dark"),
        };
        button(row![crate::icons::icon(i, 15.0), text(label).size(13)].spacing(6).align_y(Alignment::Center))
            .padding([7, 12])
            .style(theme::chip(s.theme == *choice))
            .on_press(Message::Preferences(Msg::Theme(*choice)))
            .into()
    }))
    .spacing(6);

    let appearance = ui::section("Appearance", None, ui::field("Theme", None, themes.into()));

    let status: El<'_> = match &app.seven {
        SevenState::Ready(seven) => ui::banner(
            Tone::Success,
            Icon::CircleCheck,
            column![
                ui::strong(format!("7-Zip {} is ready", seven.version)),
                ui::small(seven.path.display().to_string())
            ]
            .spacing(2)
            .into(),
            vec![ui::ghost(Some(Icon::RefreshCw), "Detect again", Some(Message::Relocate)).into()],
        ),
        SevenState::Detecting => {
            ui::banner(Tone::Neutral, Icon::LoaderCircle, ui::strong("Looking for 7-Zip…").into(), vec![])
        }
        SevenState::Missing => crate::screens::home::missing(None),
        SevenState::Invalid { path, error } => {
            crate::screens::home::missing(Some((path.display().to_string(), error.clone())))
        }
    };
    let seven = ui::section(
        "7-Zip",
        Some("7zip-GUI uses the 7-Zip installed on this computer. It is never bundled."),
        column![
            status,
            ui::field(
                "Executable",
                Some("Leave empty to find it automatically (7zz, 7z or 7za)."),
                row![
                    ui::input("Automatic", &app.seven_path_input, |v| Message::Preferences(Msg::SevenZipInput(v)))
                        .on_submit(Message::Preferences(Msg::SevenZipPath(app.seven_path_input.clone())))
                        .width(iced::Length::Fill),
                    ui::secondary(Some(Icon::FolderOpen), "Browse…", Some(Message::Preferences(Msg::BrowseSevenZip))),
                    ui::ghost(
                        None,
                        "Clear",
                        s.seven_zip_path.is_some().then_some(Message::Preferences(Msg::ClearSevenZip))
                    ),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into(),
            ),
        ]
        .spacing(16)
        .into(),
    );

    let compress = ui::section(
        "New archives",
        Some("The starting values of the Compress screen."),
        column![
            ui::field(
                "Format",
                None,
                ui::select(&ArchiveFormat::ALL, Some(s.compress.format), |f| Message::Preferences(Msg::Format(f)))
            ),
            ui::field(
                "Compression level",
                None,
                ui::select(&CompressionLevel::ALL, Some(s.compress.level), |l| Message::Preferences(Msg::Level(l)))
            ),
            ui::switch("Solid 7z archives", None, s.compress.solid, |v| Message::Preferences(Msg::Solid(v))),
            ui::switch("Encrypt file names in 7z", None, s.compress.encrypt_names, |v| Message::Preferences(
                Msg::EncryptNames(v)
            )),
            ui::switch("AES-256 for encrypted ZIP", None, s.compress.zip_aes, |v| Message::Preferences(Msg::ZipAes(v))),
        ]
        .spacing(16)
        .into(),
    );

    let extract = ui::section(
        "Extraction",
        None,
        column![
            ui::field(
                "When a file already exists",
                None,
                ui::select(&OverwriteMode::ALL, Some(s.extract.overwrite), |m| Message::Preferences(Msg::Overwrite(m)))
            ),
            ui::switch("Create a folder named after the archive", None, s.extract.into_folder, |v| {
                Message::Preferences(Msg::IntoFolder(v))
            }),
            ui::switch("Keep the folder structure", None, s.extract.keep_paths, |v| Message::Preferences(
                Msg::KeepPaths(v)
            )),
            ui::switch("Open the folder when done", None, s.extract.open_when_done, |v| Message::Preferences(
                Msg::OpenWhenDone(v)
            )),
            ui::switch("Ask before deleting from an archive", None, s.confirm_delete, |v| Message::Preferences(
                Msg::ConfirmDelete(v)
            )),
        ]
        .spacing(16)
        .into(),
    );

    let update_status = match &app.updater.state {
        State::Checking { .. } => "Checking…".to_owned(),
        State::UpToDate => format!("{} is up to date.", crate::VERSION),
        State::Available(u) | State::Installing(u) | State::Downloading { update: u, .. } => {
            format!("Version {} is available.", u.version)
        }
        State::Failed(e) => format!("The last check failed: {e}"),
        State::HandedOver => "The installer is open.".to_owned(),
        State::Idle => format!("You have version {}.", crate::VERSION),
    };
    let signing = if sevenzip_updater::PUBLIC_KEY.is_some() {
        "Updates are downloaded from the website and installed only if their signature matches the 7zip-GUI release key."
    } else {
        "This build has no release key, so it points you to the website instead of installing updates itself."
    };
    let updates = ui::section(
        "Updates",
        Some(signing),
        column![
            ui::switch(
                "Check for updates automatically",
                Some("At startup and every six hours. Only the version file on the website is fetched; nothing about you is sent."),
                s.updates.check_automatically,
                |v| Message::Preferences(Msg::AutoUpdate(v)),
            ),
            row![
                ui::muted(update_status).width(iced::Length::Fill),
                ui::secondary(Some(Icon::RefreshCw), "Check now", Some(Message::Update(update::Msg::Check { manual: true }))),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(16)
        .into(),
    );

    let advanced = ui::section(
        "Data",
        None,
        column![
            row![
                ui::secondary(
                    Some(Icon::FolderOpen),
                    "Settings folder",
                    Some(Message::Preferences(Msg::OpenConfigFolder))
                ),
                ui::secondary(Some(Icon::Terminal), "Logs", Some(Message::Preferences(Msg::OpenLogFolder))),
                ui::secondary(
                    Some(Icon::Clock),
                    "Clear recent archives",
                    (!s.recent.is_empty()).then_some(Message::Preferences(Msg::ClearRecent))
                ),
            ]
            .spacing(8),
            ui::small(paths::settings_file().map(|p| p.display().to_string()).unwrap_or_default()),
        ]
        .spacing(10)
        .into(),
    );

    ui::page(header, column![appearance, seven, compress, extract, updates, advanced].spacing(ui::L).into())
}
