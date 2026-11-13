//! Extracting an archive (or a selection from it) to a folder.

use std::path::PathBuf;

use coffer_core::{ExtractOptions, OverwriteMode};
use iced::widget::{column, row, text};
use iced::{Alignment, Length, Task};

use crate::app::{App, Message, Screen};
use crate::icons::Icon;
use crate::jobs::{Spec, plural};
use crate::screens::browse::default_destination;
use crate::settings::ExtractDefaults;
use crate::theme::Tone;
use crate::ui::{self, El};

#[derive(Debug, Clone)]
pub enum Msg {
    Archive(String),
    BrowseArchive,
    Destination(String),
    BrowseDestination,
    IntoFolder(bool),
    Overwrite(OverwriteMode),
    KeepPaths(bool),
    Password(String),
    ShowPassword(bool),
    OpenWhenDone(bool),
    ClearSelection,
    Start,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Form {
    pub archive: String,
    pub destination: String,
    pub destination_touched: bool,
    pub into_folder: bool,
    pub overwrite: OverwriteMode,
    pub keep_paths: bool,
    pub password: String,
    pub show_password: bool,
    pub open_when_done: bool,
    /// Entries chosen in the archive browser; empty extracts everything.
    pub only: Vec<String>,
}

impl Form {
    pub fn from_defaults(defaults: &ExtractDefaults) -> Self {
        Self {
            archive: String::new(),
            destination: String::new(),
            destination_touched: false,
            into_folder: defaults.into_folder,
            overwrite: defaults.overwrite,
            keep_paths: defaults.keep_paths,
            password: String::new(),
            show_password: false,
            open_when_done: defaults.open_when_done,
            only: Vec::new(),
        }
    }

    pub fn set_archive(&mut self, archive: PathBuf, defaults: &ExtractDefaults) {
        *self = Self { archive: archive.display().to_string(), ..Self::from_defaults(defaults) };
        self.suggest();
    }

    fn suggest(&mut self) {
        if self.destination_touched || self.archive.trim().is_empty() {
            return;
        }
        self.destination =
            default_destination(&PathBuf::from(self.archive.trim()), self.into_folder).display().to_string();
    }

    pub fn problem(&self) -> Option<&'static str> {
        if self.archive.trim().is_empty() {
            return Some("Choose the archive to extract.");
        }
        if !PathBuf::from(self.archive.trim()).is_file() {
            return Some("The archive doesn't exist.");
        }
        if self.destination.trim().is_empty() {
            return Some("Choose where to extract.");
        }
        None
    }
}

impl App {
    pub(crate) fn update_extract(&mut self, msg: Msg) -> Task<Message> {
        let form = &mut self.extract;
        match msg {
            Msg::Archive(value) => {
                form.archive = value;
                form.only.clear();
                form.suggest();
            }
            Msg::BrowseArchive => {
                return Task::perform(
                    rfd::AsyncFileDialog::new()
                        .set_title("Choose an archive to extract")
                        .add_filter("Archives", crate::app::ARCHIVE_EXTENSIONS)
                        .add_filter("All files", &["*"])
                        .pick_file(),
                    |file| {
                        file.map_or(Message::Nothing, |f| {
                            Message::Extract(Msg::Archive(f.path().display().to_string()))
                        })
                    },
                );
            }
            Msg::Destination(value) => {
                form.destination = value;
                form.destination_touched = true;
            }
            Msg::BrowseDestination => {
                return Task::perform(rfd::AsyncFileDialog::new().set_title("Extract to").pick_folder(), |dir| {
                    dir.map_or(Message::Nothing, |d| Message::Extract(Msg::Destination(d.path().display().to_string())))
                });
            }
            Msg::IntoFolder(on) => {
                form.into_folder = on;
                form.destination_touched = false;
                form.suggest();
            }
            Msg::Overwrite(mode) => form.overwrite = mode,
            Msg::KeepPaths(on) => form.keep_paths = on,
            Msg::Password(p) => form.password = p,
            Msg::ShowPassword(on) => form.show_password = on,
            Msg::OpenWhenDone(on) => form.open_when_done = on,
            Msg::ClearSelection => form.only.clear(),
            Msg::Start => {
                if form.problem().is_some() {
                    return Task::none();
                }
                let spec = Spec::Extract {
                    archive: PathBuf::from(form.archive.trim()),
                    options: ExtractOptions {
                        destination: PathBuf::from(form.destination.trim()),
                        overwrite: form.overwrite,
                        keep_paths: form.keep_paths,
                        password: (!form.password.is_empty()).then(|| form.password.clone()),
                        only: form.only.clone(),
                    },
                    open_when_done: form.open_when_done,
                };
                form.password.clear();
                self.screen = Screen::Activity;
                return self.start(spec);
            }
        }
        Task::none()
    }
}

pub fn view(app: &App) -> El<'_> {
    let form = &app.extract;
    let header = ui::page_header("Extract", "Unpack an archive into a folder.", vec![]);
    let mut body = column![].spacing(ui::L);

    if !form.only.is_empty() {
        body = body.push(ui::banner(
            Tone::Accent,
            Icon::ListFilter,
            text(format!("Only the {} you selected will be extracted.", plural(form.only.len(), "item"))).into(),
            vec![ui::ghost(None, "Extract everything instead", Some(Message::Extract(Msg::ClearSelection))).into()],
        ));
    }

    body = body.push(ui::section(
        "Archive",
        None,
        column![
            ui::field(
                "Archive",
                Some("7z, ZIP, RAR, TAR, ISO and every format your 7-Zip reads."),
                ui::path_input(
                    "Choose an archive",
                    &form.archive,
                    |v| Message::Extract(Msg::Archive(v)),
                    Message::Extract(Msg::BrowseArchive)
                ),
            ),
            ui::field(
                "Password",
                Some("Only for encrypted archives."),
                column![
                    ui::input("No password", &form.password, |v| Message::Extract(Msg::Password(v)))
                        .secure(!form.show_password),
                    iced::widget::checkbox(form.show_password)
                        .label("Show the password")
                        .on_toggle(|v| Message::Extract(Msg::ShowPassword(v)))
                        .size(16),
                ]
                .spacing(8)
                .into(),
            ),
        ]
        .spacing(16)
        .into(),
    ));

    body = body.push(ui::section(
        "Destination",
        None,
        column![
            ui::field(
                "Extract to",
                None,
                ui::path_input(
                    "Choose a folder",
                    &form.destination,
                    |v| Message::Extract(Msg::Destination(v)),
                    Message::Extract(Msg::BrowseDestination)
                ),
            ),
            ui::switch(
                "Create a folder named after the archive",
                Some("Keeps loose files from spilling into the destination."),
                form.into_folder,
                |v| Message::Extract(Msg::IntoFolder(v))
            ),
            ui::switch(
                "Keep the folder structure",
                Some("Off puts every file directly in the destination."),
                form.keep_paths,
                |v| Message::Extract(Msg::KeepPaths(v))
            ),
            ui::field(
                "When a file already exists",
                None,
                ui::select(&OverwriteMode::ALL, Some(form.overwrite), |m| Message::Extract(Msg::Overwrite(m)))
            ),
            ui::switch("Open the folder when done", None, form.open_when_done, |v| Message::Extract(
                Msg::OpenWhenDone(v)
            )),
        ]
        .spacing(16)
        .into(),
    ));

    let start = ui::primary(
        Some(Icon::FolderOutput),
        "Extract",
        form.problem().is_none().then_some(Message::Extract(Msg::Start)),
    );
    let footer: El<'_> = match form.problem() {
        Some(problem) if !form.archive.is_empty() => {
            row![ui::banner(Tone::Warning, Icon::CircleAlert, text(problem).into(), vec![]), start]
                .spacing(12)
                .align_y(Alignment::Center)
                .into()
        }
        _ => row![ui::fill(), start].into(),
    };
    body = body.push(footer);
    let _ = Length::Fill;
    ui::page(header, body.into())
}
