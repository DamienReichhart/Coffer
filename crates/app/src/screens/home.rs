//! The start screen: a drop zone, the main actions, recent archives, and
//! what to do when 7-Zip is missing.

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Length};

use crate::app::{App, Message, Screen, SevenState};
use crate::fonts;
use crate::icons::{Icon, icon};
use crate::theme::{self, Tone};
use crate::ui::{self, El};

/// One of the main actions: an icon, what it does, and its shortcut.
fn action<'a>(i: Icon, title: &'a str, body: &'a str, shortcut: String, msg: Message) -> El<'a> {
    button(
        row![
            crate::icons::tinted(i, 18.0, |t| t.primary),
            column![text(title).size(14).font(fonts::MEDIUM), ui::small(body)].spacing(1).width(Length::Fill),
            ui::kbd(shortcut),
        ]
        .spacing(14)
        .align_y(Alignment::Center),
    )
    .padding([11, 14])
    .width(Length::Fill)
    .style(theme::list_row(false))
    .on_press(msg)
    .into()
}

pub fn view(app: &App) -> El<'_> {
    let modifier = ui::shortcut_label();
    let header = ui::page_header("Home", "Open, create and extract archives with the 7-Zip on this computer.", vec![]);

    let mut body = column![].spacing(ui::XL);

    match &app.seven {
        SevenState::Missing => body = body.push(missing(None)),
        SevenState::Invalid { path, error } => {
            body = body.push(missing(Some((path.display().to_string(), error.clone()))));
        }
        _ => {}
    }

    let actions = ui::list(vec![
        action(
            Icon::FolderOpen,
            "Open an archive…",
            "7z, ZIP, RAR, TAR, ISO and every other format 7-Zip reads, split archives included.",
            format!("{modifier} O"),
            Message::OpenDialog,
        ),
        action(
            Icon::PackagePlus,
            "New archive",
            "Compress files and folders into a 7z, ZIP or TAR, with a password or in volumes.",
            format!("{modifier} N"),
            Message::Navigate(Screen::Compress),
        ),
        action(
            Icon::FolderOutput,
            "Extract…",
            "Unpack an archive into a folder of your choice.",
            format!("{modifier} E"),
            Message::Navigate(Screen::Extract),
        ),
    ]);
    body = body.push(actions);
    body = body.push(recent(app));
    body = body.push(
        row![
            icon(Icon::Upload, 14.0),
            ui::small("Drop an archive anywhere on this window to open it, or other files to compress them.")
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );
    ui::page(header, body.into())
}

fn recent(app: &App) -> El<'_> {
    let heading = row![
        text("Recent").size(13).font(fonts::SEMIBOLD).width(Length::Fill),
        ui::ghost(
            None,
            "Clear",
            (!app.settings.recent.is_empty())
                .then_some(Message::Preferences(crate::screens::preferences::Msg::ClearRecent))
        ),
    ]
    .align_y(Alignment::Center);
    if app.settings.recent.is_empty() {
        let empty = container(ui::muted("Archives you open will be listed here.").size(13))
            .padding([14, 14])
            .width(Length::Fill)
            .style(theme::card);
        return column![heading, empty].spacing(ui::S).into();
    }
    let rows = app
        .settings
        .recent
        .iter()
        .map(|path| {
            let name =
                path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
            let folder = path.parent().map(|p| p.display().to_string()).unwrap_or_default();
            button(
                row![
                    icon(Icon::FileArchive, 16.0),
                    text(name).size(13).font(fonts::MEDIUM).wrapping(text::Wrapping::None),
                    container(ui::small(folder).wrapping(text::Wrapping::None)).width(Length::Fill).clip(true),
                    icon(Icon::ChevronRight, 14.0),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding([9, 14])
            .width(Length::Fill)
            .style(theme::list_row(false))
            .on_press(Message::Open(path.clone(), None))
            .into()
        })
        .collect();
    column![heading, ui::list(rows)].spacing(ui::S).into()
}

/// Explains how to install 7-Zip, with copyable commands.
pub fn missing<'a>(invalid: Option<(String, String)>) -> El<'a> {
    let (title, body) = match &invalid {
        Some((path, _)) => (
            "The configured 7-Zip doesn't work".to_owned(),
            format!("{path} is not a working 7-Zip executable. Choose another one in Settings, or clear it to search again."),
        ),
        None => (
            "7-Zip is not installed".to_owned(),
            "Coffer drives the 7-Zip you install; it doesn't include one. Install it with one of these, then press Check again.".to_owned(),
        ),
    };
    let methods = coffer_core::install::methods().iter().map(|m| {
        let right: El<'a> = match m.command {
            Some(command) => row![
                container(ui::mono(command).size(12)).padding([5, 10]).style(theme::inset).width(Length::Fill),
                ui::icon_button(Icon::Copy, "Copy", Some(Message::Copy(command.to_owned()))),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into(),
            None => ui::secondary(
                Some(Icon::ExternalLink),
                "7-zip.org",
                Some(Message::OpenUrl(coffer_core::install::DOWNLOAD_PAGE.into())),
            )
            .into(),
        };
        row![ui::small(m.label).width(Length::Fixed(190.0)), right].spacing(12).align_y(Alignment::Center).into()
    });
    container(
        column![
            row![
                crate::icons::tinted(Icon::TriangleAlert, 18.0, |t| t.warning),
                column![ui::heading(title), ui::muted(body).size(13)].spacing(2).width(Length::Fill),
                ui::secondary(Some(Icon::RefreshCw), "Check again", Some(Message::Relocate)),
                ui::ghost(None, "Settings", Some(Message::Navigate(Screen::Settings))),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            column(methods).spacing(6),
        ]
        .spacing(14),
    )
    .padding(16)
    .style(theme::tinted(Tone::Warning))
    .into()
}
