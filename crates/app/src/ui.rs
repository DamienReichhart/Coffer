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

/// The widest a page's content gets: forms and lists stay readable on wide windows.
pub const CONTENT_WIDTH: f32 = 860.0;

pub fn title<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).size(20).font(fonts::SEMIBOLD)
}

pub fn heading<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).size(14).font(fonts::SEMIBOLD)
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
            icon(i, 15.0).style(move |theme: &Theme, _| iced::widget::svg::Style {
                color: Some(style(theme, button::Status::Active).text_color),
            }),
            text(label).size(13).font(fonts::MEDIUM).wrapping(text::Wrapping::None)
        ]
        .spacing(7)
        .align_y(Alignment::Center)
        .into(),
        None => text(label).size(13).font(fonts::MEDIUM).wrapping(text::Wrapping::None).into(),
    };
    button(content).padding([7, 13]).style(style).on_press_maybe(on_press)
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
    button_with(i, label, theme::ghost, msg).padding([5, 9])
}

/// A square icon-only button with a tooltip.
pub fn icon_button(i: Icon, tip: &str, msg: Option<Message>) -> El<'_> {
    let b =
        button(container(icon(i, 16.0)).center(Length::Fixed(18.0))).padding(6).style(theme::ghost).on_press_maybe(msg);
    tooltip(b, container(text(tip).size(12)).padding([4, 8]).style(theme::modal), tooltip::Position::Bottom)
        .gap(4)
        .into()
}

pub fn badge<'a>(label: impl text::IntoFragment<'a>, tone: Tone) -> El<'a> {
    container(text(label).size(11).font(fonts::MEDIUM)).padding([1, 6]).style(theme::badge(tone)).into()
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

/// The header of a page: title, a one-line description and actions on the right.
pub fn page_header<'a>(heading_text: &'a str, description: &'a str, actions: Vec<El<'a>>) -> El<'a> {
    let mut left = column![title(heading_text)].spacing(2).width(Length::Fill);
    if !description.is_empty() {
        left = left.push(muted(description).size(13));
    }
    row![left, row(actions).spacing(S).align_y(Alignment::Center)].align_y(Alignment::Center).spacing(L).into()
}

fn centered<'a>(content: impl Into<El<'a>>, padding: Padding) -> El<'a> {
    container(container(content).max_width(CONTENT_WIDTH)).padding(padding).center_x(Length::Fill).into()
}

/// A page: a header above scrolling content.
pub fn page<'a>(header: El<'a>, body: El<'a>) -> El<'a> {
    scrolling(header, body, XXL)
}

fn scrolling<'a>(header: El<'a>, body: El<'a>, bottom: f32) -> El<'a> {
    let content = column![header, body].spacing(XL);
    iced::widget::scrollable(centered(content, Padding::new(XXL).top(XL).bottom(bottom)))
        .style(theme::scroll)
        .height(Length::Fill)
        .into()
}

/// The height of a form's action bar, which the content scrolls under.
const ACTION_BAR: f32 = 54.0;

/// A form page: the header and fields scroll, the action bar stays at the bottom.
pub fn form_page<'a>(header: El<'a>, body: El<'a>, actions: El<'a>) -> El<'a> {
    let bar = column![
        rule::horizontal(1).style(theme::hairline),
        container(centered(actions, Padding::from([0.0, XXL])))
            .center_y(Length::Fixed(ACTION_BAR - 1.0))
            .width(Length::Fill)
            .style(theme::toolbar),
    ];
    // The bar is a layer above the content, so nothing scrolled under it shows through.
    iced::widget::stack![
        scrolling(header, body, ACTION_BAR + XL),
        container(bar).align_bottom(Length::Fill).width(Length::Fill),
    ]
    .into()
}

/// The action bar of a form: what still blocks it on the left, the actions on the right.
pub fn form_actions<'a>(problem: Option<String>, actions: Vec<El<'a>>) -> El<'a> {
    let left: El<'a> = match problem {
        Some(problem) => row![
            crate::icons::tinted(Icon::CircleAlert, 15.0, |t| t.warning),
            text(problem).size(13).style(theme::tone_text(Tone::Warning)),
        ]
        .spacing(S)
        .align_y(Alignment::Center)
        .into(),
        None => Space::new().into(),
    };
    row![container(left).width(Length::Fill), row(actions).spacing(S).align_y(Alignment::Center)]
        .spacing(L)
        .align_y(Alignment::Center)
        .into()
}

/// A titled group of related settings or fields.
pub fn section<'a>(heading_text: &'a str, description: Option<&'a str>, body: El<'a>) -> El<'a> {
    let mut header = column![text(heading_text).size(13).font(fonts::SEMIBOLD)].spacing(2);
    if let Some(d) = description {
        header = header.push(small(d));
    }
    column![header, container(body).padding(L).width(Length::Fill).style(theme::card)].spacing(S).into()
}

/// Rows separated by hairlines, for the inside of a [`section`].
pub fn rows(items: Vec<El<'_>>) -> El<'_> {
    let mut out = column![].spacing(M);
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            out = out.push(rule::horizontal(1).style(theme::hairline));
        }
        out = out.push(item);
    }
    out.into()
}

/// Clickable rows separated by hairlines in an outlined group, edge to edge.
pub fn list(items: Vec<El<'_>>) -> El<'_> {
    let mut out = column![];
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            out = out.push(rule::horizontal(1).style(theme::hairline));
        }
        out = out.push(item);
    }
    container(out).width(Length::Fill).clip(true).style(theme::card).into()
}

/// A form row: label and help on the left, the control on the right.
pub fn field<'a>(label: &'a str, help: Option<&'a str>, control: El<'a>) -> El<'a> {
    let mut left = column![text(label).size(13).font(fonts::MEDIUM)].spacing(2).width(Length::Fixed(260.0));
    if let Some(h) = help {
        left = left.push(small(h));
    }
    row![left, container(control).width(Length::Fill).align_right(Length::Fill)]
        .spacing(XL)
        .align_y(Alignment::Center)
        .into()
}

/// Mutually exclusive options shown side by side.
pub fn segmented<'a, T: Copy + PartialEq + 'a>(
    options: impl IntoIterator<Item = (T, El<'a>)>,
    selected: T,
    on_select: impl Fn(T) -> Message + 'a,
) -> El<'a> {
    let buttons = options.into_iter().map(|(value, label)| {
        button(label).padding([4, 10]).style(theme::segment(value == selected)).on_press(on_select(value)).into()
    });
    container(row(buttons).spacing(2)).padding(2).style(theme::segmented).into()
}

/// A labelled switch row.
pub fn switch<'a>(
    label: &'a str,
    help: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> El<'a> {
    field(label, help, toggler(value).on_toggle(on_toggle).size(18).style(theme::switch).into())
}

pub fn input<'a>(
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> text_input::TextInput<'a, Message> {
    text_input(placeholder, value).on_input(on_input).padding([7, 10]).size(13).style(theme::input)
}

pub fn select<'a, T>(options: &'a [T], selected: Option<T>, on_select: impl Fn(T) -> Message + 'a) -> El<'a>
where
    T: ToString + PartialEq + Clone + 'a,
{
    pick_list(options, selected, on_select)
        .padding([6, 10])
        .text_size(13)
        .width(Length::Fill)
        .style(theme::select)
        .into()
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
    let pick: fn(&theme::Tokens) -> iced::Color = match tone {
        Tone::Accent => |t| t.primary,
        Tone::Danger => |t| t.danger,
        Tone::Warning => |t| t.warning,
        Tone::Success => |t| t.success,
        Tone::Neutral => |t| t.subtle,
    };
    let mut content = column![crate::icons::tinted(i, 28.0, pick), vspace(XS), heading(heading_text)]
        .spacing(S)
        .align_x(Alignment::Center)
        .max_width(420);
    if !body.is_empty() {
        content = content.push(muted(body).size(13).align_x(Alignment::Center));
    }
    if !actions.is_empty() {
        content = content.push(vspace(XS)).push(row(actions).spacing(S));
    }
    container(content).center(Length::Fill).into()
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
            crate::icons::tinted(i, 16.0, pick),
            container(message).width(Length::Fill),
            row(actions).spacing(S).align_y(Alignment::Center),
        ]
        .spacing(M)
        .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .style(theme::tinted(tone))
    .into()
}

/// Two columns of "label: value" lines.
pub fn properties(items: Vec<(&str, String)>) -> El<'_> {
    column(items.into_iter().map(|(k, v)| {
        row![small(k).width(Length::Fixed(110.0)), text(v).size(13)].spacing(S).align_y(Alignment::Start).into()
    }))
    .spacing(6)
    .into()
}

/// A keyboard shortcut label.
pub fn kbd<'a>(keys: impl text::IntoFragment<'a>) -> El<'a> {
    container(text(keys).size(11).font(fonts::MEDIUM).style(theme::muted))
        .padding([1, 5])
        .style(|t: &Theme| iced::widget::container::Style {
            border: iced::Border { radius: theme::RADIUS_S.into(), ..theme::inset(t).border },
            ..theme::inset(t)
        })
        .into()
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
