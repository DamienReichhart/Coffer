//! The frame around every screen: the sidebar, the update banner, toasts and
//! the drag-and-drop overlay.

use iced::widget::{Space, button, column, container, progress_bar, row, svg, text};
use iced::{Alignment, Element, Length, Padding};

use crate::app::{App, Message, Screen, SevenState};
use crate::fonts;
use crate::icons::{Icon, icon};
use crate::theme::{self, Tone};
use crate::ui::{self, El};
use crate::update::{self, State};

const LOGO: &[u8] = include_bytes!("../../../../assets/brand/icon.svg");

fn nav_item<'a>(app: &App, screen: Screen, i: Icon, label: String, enabled: bool, trailing: Option<El<'a>>) -> El<'a> {
    let active = app.screen == screen;
    let color = move |theme: &iced::Theme, _| {
        let t = theme::tokens(theme);
        svg::Style { color: Some(if active { t.primary } else { t.muted }) }
    };
    let mut content = row![
        icon(i, 18.0).style(color),
        text(label).font(if active { fonts::MEDIUM } else { fonts::REGULAR }).width(Length::Fill),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    if let Some(trailing) = trailing {
        content = content.push(trailing);
    }
    button(content)
        .width(Length::Fill)
        .padding([8, 10])
        .style(theme::nav(active))
        .on_press_maybe(enabled.then_some(Message::Navigate(screen)))
        .into()
}

pub fn sidebar(app: &App) -> El<'_> {
    let brand = row![
        svg(svg::Handle::from_memory(LOGO)).width(30).height(30),
        column![text(crate::APP_NAME).font(fonts::SEMIBOLD).size(15), ui::small("for 7-Zip")].spacing(0),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let running = app.jobs.running();
    let activity_badge = (running > 0).then(|| ui::badge(running.to_string(), Tone::Accent));
    let archive_label = app.browser.as_ref().map_or_else(|| "Archive".to_owned(), |b| b.name().to_owned());

    let nav = column![
        nav_item(app, Screen::Home, Icon::House, "Home".into(), true, None),
        nav_item(
            app,
            Screen::Browse,
            Icon::FileArchive,
            archive_label,
            app.browser.is_some() || app.opening.is_some(),
            None
        ),
        nav_item(app, Screen::Compress, Icon::PackagePlus, "Compress".into(), true, None),
        nav_item(app, Screen::Extract, Icon::FolderOutput, "Extract".into(), true, None),
        nav_item(app, Screen::Checksums, Icon::Hash, "Checksums".into(), true, None),
        nav_item(app, Screen::Activity, Icon::Activity, "Activity".into(), true, activity_badge),
    ]
    .spacing(2);

    let footer = column![
        nav_item(app, Screen::Settings, Icon::Settings, "Settings".into(), true, None),
        nav_item(app, Screen::About, Icon::Info, "About".into(), true, None),
        seven_status(app),
    ]
    .spacing(2);

    container(
        column![
            container(brand).padding(Padding::new(6.0).bottom(14.0)),
            nav,
            Space::new().height(Length::Fill),
            footer,
        ]
        .spacing(6),
    )
    .padding([18, 12])
    .width(Length::Fixed(220.0))
    .height(Length::Fill)
    .style(theme::sidebar)
    .into()
}

fn seven_status(app: &App) -> El<'_> {
    let (tone, label) = match &app.seven {
        SevenState::Detecting => (Tone::Neutral, "Looking for 7-Zip…".to_owned()),
        SevenState::Ready(seven) => (Tone::Success, format!("7-Zip {}", seven.version)),
        SevenState::Missing => (Tone::Danger, "7-Zip not found".to_owned()),
        SevenState::Invalid { .. } => (Tone::Warning, "7-Zip path invalid".to_owned()),
    };
    let dot = container(Space::new().width(8).height(8)).style(theme::badge(tone));
    let dot = container(dot).style(move |theme: &iced::Theme| {
        let (_, fg) = tone.colors(theme::tokens(theme));
        container::Style {
            background: Some(fg.into()),
            border: iced::Border { radius: 999.0.into(), ..Default::default() },
            ..Default::default()
        }
    });
    button(row![dot, ui::small(label)].spacing(8).align_y(Alignment::Center))
        .padding([8, 10])
        .width(Length::Fill)
        .style(theme::ghost)
        .on_press(Message::Navigate(Screen::Settings))
        .into()
}

/// The "a new version is available" bar above the content.
pub fn update_banner(app: &App) -> El<'_> {
    let skipped = app.settings.updates.skipped_version.as_deref();
    let Some(state) = app.updater.banner(skipped) else { return Space::new().into() };
    let content: El<'_> = match state {
        State::Available(update) => {
            let install_label = if update.asset.is_some() { "Update now" } else { "Download" };
            let notes = update.notes_url.clone().unwrap_or_else(|| crate::CHANGELOG.to_owned());
            ui::banner(
                Tone::Accent,
                Icon::Sparkles,
                row![
                    ui::strong(format!("{} {} is available", crate::APP_NAME, update.version)),
                    ui::muted(format!(" — you have {}.", crate::VERSION)),
                ]
                .into(),
                vec![
                    ui::ghost(None, "What's new", Some(Message::OpenUrl(notes))).into(),
                    ui::ghost(
                        None,
                        "Skip this version",
                        Some(Message::Preferences(crate::screens::preferences::Msg::SkipVersion(
                            update.version.to_string(),
                        ))),
                    )
                    .into(),
                    ui::primary(Some(Icon::Download), install_label, Some(Message::Update(update::Msg::Install)))
                        .into(),
                    ui::icon_button(Icon::X, "Later", Some(Message::Update(update::Msg::Dismiss))),
                ],
            )
        }
        State::Downloading { update, progress } => {
            let ratio = if progress.total == 0 { 0.0 } else { progress.received as f32 / progress.total as f32 };
            ui::banner(
                Tone::Accent,
                Icon::Download,
                column![
                    ui::strong(format!(
                        "Downloading {} {} — {} of {}",
                        crate::APP_NAME,
                        update.version,
                        sevenzip_core::format::bytes(progress.received),
                        sevenzip_core::format::bytes(progress.total)
                    )),
                    progress_bar(0.0..=1.0, ratio).girth(6).style(theme::progress(Tone::Accent)),
                ]
                .spacing(6)
                .into(),
                vec![ui::ghost(None, "Cancel", Some(Message::Update(update::Msg::Cancel))).into()],
            )
        }
        State::Installing(update) => ui::banner(
            Tone::Accent,
            Icon::LoaderCircle,
            ui::strong(format!(
                "Verified. Installing {} {}… the app restarts when it's done.",
                crate::APP_NAME,
                update.version
            ))
            .into(),
            vec![],
        ),
        _ => return Space::new().into(),
    };
    container(content).padding(Padding::new(12.0).bottom(0.0)).into()
}

pub fn toasts(app: &App) -> El<'_> {
    let items = app.toasts.iter().map(|toast| {
        let i = match toast.tone {
            Tone::Success => Icon::CircleCheck,
            Tone::Danger => Icon::CircleX,
            Tone::Warning => Icon::TriangleAlert,
            _ => Icon::Info,
        };
        let pick: fn(&theme::Tokens) -> iced::Color = match toast.tone {
            Tone::Success => |t| t.success,
            Tone::Danger => |t| t.danger,
            Tone::Warning => |t| t.warning,
            _ => |t| t.primary,
        };
        let mut line = row![crate::icons::tinted(i, 18.0, pick), text(toast.text.as_str()).width(Length::Fill)]
            .spacing(10)
            .align_y(Alignment::Center);
        if let Some((label, msg)) = &toast.action {
            line = line.push(ui::ghost(None, label.as_str(), Some(msg.clone())));
        }
        line = line.push(ui::icon_button(Icon::X, "Dismiss", Some(Message::DismissToast(toast.id))));
        container(line).padding([10, 12]).width(Length::Fixed(380.0)).style(theme::toast(toast.tone)).into()
    });
    container(column(items).spacing(8).align_x(Alignment::End))
        .align_right(Length::Fill)
        .align_bottom(Length::Fill)
        .padding(20)
        .into()
}

pub fn drop_overlay(app: &App) -> El<'_> {
    let hint = match app.screen {
        crate::app::Screen::Compress => "Drop to add to the new archive",
        crate::app::Screen::Checksums => "Drop to compute checksums",
        crate::app::Screen::Browse if app.browser.is_some() => "Drop to add to this archive, or an archive to open it",
        _ => "Drop an archive to open it, or files to compress them",
    };
    let card = container(
        column![ui::icon_tile(Icon::Upload, Tone::Accent, 64.0), ui::heading(hint).size(18)]
            .spacing(14)
            .align_x(Alignment::Center),
    )
    .padding(40)
    .style(theme::drop_zone(true));
    let element: Element<'_, Message> = container(card).center(Length::Fill).style(theme::backdrop).padding(40).into();
    element
}
