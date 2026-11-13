//! Reusable building blocks, so every screen looks and behaves the same.

use iced::widget::{Space, button, column, container, pick_list, row, rule, text, text_input, toggler, tooltip};
use iced::{Alignment, Element, Length, Padding, Theme};

use crate::app::Message;
use crate::fonts;
use crate::icons::{Icon, icon};
use crate::theme::{self, Tone};

pub type El<'a> = Element<'a, Message>;

/// Spacing scale.
pub const XS: f32 = 4.0;
pub const S: f32 = 8.0;
pub const M: f32 = 12.0;
pub const L: f32 = 16.0;
pub const XL: f32 = 24.0;
pub const XXL: f32 = 32.0;

pub fn title<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).size(22).font(fonts::SEMIBOLD)
}

pub fn heading<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).size(15).font(fonts::SEMIBOLD)
}

pub fn strong<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).font(fonts::MEDIUM)
}

pub fn muted<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).style(theme::muted)
}

pub fn small<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).size(12).style(theme::muted)
}

pub fn mono<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).font(fonts::MONO).size(13)
}

/// A button with an optional leading icon.
pub fn button_with<'a>(
    leading: Option<Icon>,
    label: impl text::IntoFragment<'a>,
    style: fn(&Theme, button::Status) -> button::Style,
    on_press: Option<Message>,
) -> button::Button<'a, Message> {
    let content: El<'a> = match leading {
        Some(i) => row![
            icon(i, 16.0).style(move |theme: &Theme, _| iced::widget::svg::Style {
                color: Some(style(theme, button::Status::Active).text_color),
            }),
            text(label).font(fonts::MEDIUM).wrapping(text::Wrapping::None)
        ]
        .spacing(S)
        .align_y(Alignment::Center)
        .into(),
        None => text(label).font(fonts::MEDIUM).wrapping(text::Wrapping::None).into(),
    };
    button(content).padding([9, 16]).style(style).on_press_maybe(on_press)
}

pub fn primary<'a>(
    i: Option<Icon>,
    label: impl text::IntoFragment<'a>,
    msg: Option<Message>,
) -> button::Button<'a, Message> {
    button_with(i, label, theme::primary, msg)
}

pub fn secondary<'a>(
    i: Option<Icon>,
    label: impl text::IntoFragment<'a>,
    msg: Option<Message>,
) -> button::Button<'a, Message> {
    button_with(i, label, theme::secondary, msg)
}

pub fn danger<'a>(
    i: Option<Icon>,
    label: impl text::IntoFragment<'a>,
    msg: Option<Message>,
) -> button::Button<'a, Message> {
    button_with(i, label, theme::danger, msg)
}

pub fn ghost<'a>(
    i: Option<Icon>,
    label: impl text::IntoFragment<'a>,
    msg: Option<Message>,
) -> button::Button<'a, Message> {
    button_with(i, label, theme::ghost, msg).padding([6, 10])
}

/// A square icon-only button with a tooltip.
pub fn icon_button(i: Icon, tip: &str, msg: Option<Message>) -> El<'_> {
    let b =
        button(container(icon(i, 17.0)).center(Length::Fixed(20.0))).padding(7).style(theme::ghost).on_press_maybe(msg);
    tooltip(b, container(text(tip).size(12)).padding([4, 8]).style(theme::modal), tooltip::Position::Bottom)
        .gap(4)
        .into()
}

pub fn badge<'a>(label: impl text::IntoFragment<'a>, tone: Tone) -> El<'a> {
    container(text(label).size(11).font(fonts::MEDIUM)).padding([2, 8]).style(theme::badge(tone)).into()
}

/// A rounded tile holding an icon, used by cards and empty states.
pub fn icon_tile<'a>(i: Icon, tone: Tone, size: f32) -> El<'a> {
    let pick: fn(&theme::Tokens) -> iced::Color = match tone {
        Tone::Success => |t| t.success,
        Tone::Warning => |t| t.warning,
        Tone::Danger => |t| t.danger,
        Tone::Neutral => |t| t.muted,
        Tone::Accent => |t| t.primary,
    };
    container(crate::icons::tinted(i, size * 0.5, pick))
        .center(Length::Fixed(size))
        .style(theme::icon_tile(tone))
        .into()
}

/// The header of a page: title, description and actions on the right.
pub fn page_header<'a>(heading_text: &'a str, description: &'a str, actions: Vec<El<'a>>) -> El<'a> {
    row![
        column![title(heading_text), muted(description)].spacing(XS).width(Length::Fill),
        row(actions).spacing(S).align_y(Alignment::Center),
    ]
    .align_y(Alignment::Center)
    .spacing(L)
    .into()
}

/// A page: a header above scrolling content, with comfortable margins.
pub fn page<'a>(header: El<'a>, body: El<'a>) -> El<'a> {
    let content = column![header, body].spacing(XL).max_width(1100);
    iced::widget::scrollable(container(content).padding(Padding::new(XXL).top(28.0)).center_x(Length::Fill))
        .style(theme::scroll)
        .height(Length::Fill)
        .into()
}

/// A titled card grouping related settings or fields.
pub fn section<'a>(heading_text: &'a str, description: Option<&'a str>, body: El<'a>) -> El<'a> {
    let mut header = column![heading(heading_text)].spacing(2);
    if let Some(d) = description {
        header = header.push(small(d));
    }
    container(column![header, rule::horizontal(1).style(rule::weak), body].spacing(L))
        .padding(XL)
        .width(Length::Fill)
        .style(theme::card)
        .into()
}

/// A form row: label and help on the left, the control on the right.
pub fn field<'a>(label: &'a str, help: Option<&'a str>, control: El<'a>) -> El<'a> {
    let mut left = column![strong(label)].spacing(2).width(Length::FillPortion(2));
    if let Some(h) = help {
        left = left.push(small(h));
    }
    row![left, container(control).width(Length::FillPortion(3)).align_right(Length::FillPortion(3))]
        .spacing(L)
        .align_y(Alignment::Center)
        .into()
}

/// A labelled switch row.
pub fn switch<'a>(
    label: &'a str,
    help: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> El<'a> {
    field(label, help, toggler(value).on_toggle(on_toggle).size(20).style(theme::switch).into())
}

pub fn input<'a>(
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> text_input::TextInput<'a, Message> {
    text_input(placeholder, value).on_input(on_input).padding([9, 12]).style(theme::input)
}

pub fn select<'a, T>(options: &'a [T], selected: Option<T>, on_select: impl Fn(T) -> Message + 'a) -> El<'a>
where
    T: ToString + PartialEq + Clone + 'a,
{
    pick_list(options, selected, on_select).padding([8, 12]).width(Length::Fill).style(theme::select).into()
}

/// A path field with a "Browse…" button.
pub fn path_input<'a>(
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_browse: Message,
) -> El<'a> {
    row![
        input(placeholder, value, on_input).width(Length::Fill),
        secondary(Some(Icon::FolderOpen), "Browse…", Some(on_browse)),
    ]
    .spacing(S)
    .into()
}

/// A centred message with an icon, for empty and error states.
pub fn empty_state<'a>(i: Icon, tone: Tone, heading_text: &'a str, body: &'a str, actions: Vec<El<'a>>) -> El<'a> {
    container(
        column![
            icon_tile(i, tone, 56.0),
            Space::new().height(XS),
            heading(heading_text).size(17),
            muted(body).align_x(Alignment::Center),
            Space::new().height(XS),
            row(actions).spacing(S),
        ]
        .spacing(S)
        .align_x(Alignment::Center)
        .max_width(460),
    )
    .center(Length::Fill)
    .into()
}

/// A tinted banner with an icon, a message and actions.
pub fn banner<'a>(tone: Tone, i: Icon, message: El<'a>, actions: Vec<El<'a>>) -> El<'a> {
    let pick: fn(&theme::Tokens) -> iced::Color = match tone {
        Tone::Success => |t| t.success,
        Tone::Warning => |t| t.warning,
        Tone::Danger => |t| t.danger,
        Tone::Neutral => |t| t.muted,
        Tone::Accent => |t| t.primary,
    };
    container(
        row![
            crate::icons::tinted(i, 18.0, pick),
            container(message).width(Length::Fill),
            row(actions).spacing(S).align_y(Alignment::Center),
        ]
        .spacing(M)
        .align_y(Alignment::Center),
    )
    .padding([10, 14])
    .style(theme::tinted(tone))
    .into()
}

/// Two columns of "label: value" lines.
pub fn properties(items: Vec<(&str, String)>) -> El<'_> {
    column(items.into_iter().map(|(k, v)| {
        row![small(k).width(Length::Fixed(132.0)), text(v).size(13)].spacing(S).align_y(Alignment::Center).into()
    }))
    .spacing(6)
    .into()
}

/// A keyboard shortcut label.
pub fn kbd<'a>(keys: impl text::IntoFragment<'a>) -> El<'a> {
    container(text(keys).size(11).font(fonts::MEDIUM).style(theme::muted)).padding([1, 6]).style(theme::inset).into()
}

pub fn shortcut_label() -> &'static str {
    if cfg!(target_os = "macos") { "⌘" } else { "Ctrl" }
}

pub fn vspace(height: f32) -> Space {
    Space::new().height(height)
}

pub fn hspace(width: f32) -> Space {
    Space::new().width(width)
}

pub fn fill() -> Space {
    Space::new().width(Length::Fill)
}
