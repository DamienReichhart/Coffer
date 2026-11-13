//! The start screen: a drop zone, the main actions, recent archives, and
//! what to do when 7-Zip is missing.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Length};

use crate::app::{App, Message, Screen, SevenState};
use crate::icons::{Icon, icon};
use crate::theme::{self, Tone};
use crate::ui::{self, El};

fn action_card<'a>(i: Icon, title: &'a str, body: &'a str, shortcut: String, msg: Message) -> El<'a> {
    button(
        column![
            ui::icon_tile(i, Tone::Accent, 44.0),
            Space::new().height(4),
            ui::heading(title),
            ui::muted(body).size(13),
            Space::new().height(Length::Fill),
            row![Space::new().width(Length::Fill), ui::kbd(shortcut)],
        ]
        .spacing(6)
        .height(Length::Fixed(176.0)),
    )
    .padding(20)
    .width(Length::Fill)
    .style(theme::card_button)
    .on_press(msg)
    .into()
}

pub fn view(app: &App) -> El<'_> {
    let modifier = ui::shortcut_label();
    let header = ui::page_header(
        "Welcome",
        "Open, create and extract archives with the 7-Zip installed on this computer.",
        vec![],
    );

    let mut body = column![].spacing(ui::XL);

    match &app.seven {
        SevenState::Missing => body = body.push(missing(None)),
        SevenState::Invalid { path, error } => {
            body = body.push(missing(Some((path.display().to_string(), error.clone()))));
        }
        _ => {}
    }

    let drop = container(
        column![
            ui::icon_tile(Icon::Upload, Tone::Accent, 52.0),
            ui::heading("Drop files here").size(17),
            ui::muted("An archive opens it; anything else starts a new archive."),
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
    .padding(32)
    .center_x(Length::Fill)
    .style(theme::drop_zone(false));
    body = body.push(drop);

    body = body.push(
        row![
            action_card(
                Icon::FolderOpen,
                "Open an archive",
                "Browse 7z, ZIP, RAR, TAR, ISO and 50 more formats.",
                format!("{modifier} O"),
                Message::OpenDialog
            ),
            action_card(
                Icon::PackagePlus,
                "Compress",
                "Make a 7z, ZIP or TAR with encryption and split volumes.",
                format!("{modifier} N"),
                Message::Navigate(Screen::Compress)
            ),
            action_card(
                Icon::FolderOutput,
                "Extract",
                "Unpack an archive where you want it, keeping its folders.",
                format!("{modifier} E"),
                Message::Navigate(Screen::Extract)
            ),
        ]
        .spacing(ui::L),
    );

    body = body.push(recent(app));
    ui::page(header, body.into())
}

fn recent(app: &App) -> El<'_> {
    if app.settings.recent.is_empty() {
        return ui::section("Recent archives", None, ui::muted("Archives you open appear here.").into());
    }
    let rows = app.settings.recent.iter().map(|path| {
        let name = path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
        let folder = path.parent().map(|p| p.display().to_string()).unwrap_or_default();
        button(
            row![
                icon(Icon::FileArchive, 18.0),
                column![ui::strong(name), ui::small(folder)].spacing(1).width(Length::Fill),
                icon(Icon::ChevronRight, 16.0),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding([8, 10])
        .width(Length::Fill)
        .style(theme::list_row(false))
        .on_press(Message::Open(path.clone(), None))
        .into()
    });
    ui::section("Recent archives", None, column(rows).spacing(2).into())
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
                container(ui::mono(command)).padding([6, 10]).style(theme::inset).width(Length::Fill),
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
                ui::icon_tile(Icon::TriangleAlert, Tone::Warning, 40.0),
                column![ui::heading(title), ui::muted(body).size(13)].spacing(2).width(Length::Fill),
                ui::secondary(Some(Icon::RefreshCw), "Check again", Some(Message::Relocate)),
                ui::ghost(None, "Settings", Some(Message::Navigate(Screen::Settings))),
            ]
            .spacing(14)
            .align_y(Alignment::Center),
            column(methods).spacing(8),
            text("").size(1),
        ]
        .spacing(16),
    )
    .padding(20)
    .style(theme::tinted(Tone::Warning))
    .into()
}
