//! The visual language: colour tokens shared with the website, the two iced
//! themes built from them, and the style functions every screen uses.

use iced::border::Radius;
use iced::widget::{button, container, pick_list, progress_bar, scrollable, text_input, toggler};
use iced::{Background, Border, Color, Shadow, Theme, Vector, color};
use serde::{Deserialize, Serialize};

/// The user's theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];
}

impl std::fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::System => "Match the system",
            Self::Light => "Light",
            Self::Dark => "Dark",
        })
    }
}

/// Every colour the interface uses. The website's CSS custom properties use
/// the same values (`website/src/styles/theme.css`).
#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub bg: Color,
    pub surface: Color,
    pub raised: Color,
    pub sidebar: Color,
    pub border: Color,
    pub border_strong: Color,
    pub text: Color,
    pub muted: Color,
    pub subtle: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub on_primary: Color,
    pub primary_soft: Color,
    pub success: Color,
    pub success_soft: Color,
    pub warning: Color,
    pub warning_soft: Color,
    pub danger: Color,
    pub danger_soft: Color,
    pub shadow: Color,
    pub backdrop: Color,
}

pub const DARK: Tokens = Tokens {
    bg: color!(0x121316),
    surface: color!(0x1a1b20),
    raised: color!(0x202127),
    sidebar: color!(0x16171b),
    border: color!(0x2c2e36),
    border_strong: color!(0x3a3d48),
    text: color!(0xecedef),
    muted: color!(0x9a9da8),
    subtle: color!(0x6c707c),
    primary: color!(0x7c8cff),
    primary_hover: color!(0x96a2ff),
    on_primary: color!(0x0b0d1a),
    primary_soft: Color { r: 0.486, g: 0.549, b: 1.0, a: 0.14 },
    success: color!(0x3ccf8e),
    success_soft: Color { r: 0.235, g: 0.812, b: 0.557, a: 0.14 },
    warning: color!(0xf0b445),
    warning_soft: Color { r: 0.941, g: 0.706, b: 0.271, a: 0.14 },
    danger: color!(0xf26d6a),
    danger_soft: Color { r: 0.949, g: 0.427, b: 0.416, a: 0.14 },
    shadow: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.45 },
    backdrop: Color { r: 0.02, g: 0.02, b: 0.03, a: 0.62 },
};

pub const LIGHT: Tokens = Tokens {
    bg: color!(0xf7f7f5),
    surface: color!(0xffffff),
    raised: color!(0xefefec),
    sidebar: color!(0xf0f0ed),
    border: color!(0xe2e2de),
    border_strong: color!(0xcfcfca),
    text: color!(0x17181c),
    muted: color!(0x5f6270),
    subtle: color!(0x8b8e99),
    primary: color!(0x4555e8),
    primary_hover: color!(0x3443d0),
    on_primary: color!(0xffffff),
    primary_soft: Color { r: 0.271, g: 0.333, b: 0.910, a: 0.10 },
    success: color!(0x138a55),
    success_soft: Color { r: 0.075, g: 0.541, b: 0.333, a: 0.10 },
    warning: color!(0xb26b00),
    warning_soft: Color { r: 0.698, g: 0.420, b: 0.0, a: 0.10 },
    danger: color!(0xd03a35),
    danger_soft: Color { r: 0.816, g: 0.227, b: 0.208, a: 0.10 },
    shadow: Color { r: 0.07, g: 0.08, b: 0.12, a: 0.14 },
    backdrop: Color { r: 0.07, g: 0.08, b: 0.10, a: 0.38 },
};

/// Corner radii: small, so controls read as tools rather than cards.
pub const RADIUS_S: f32 = 4.0;
pub const RADIUS: f32 = 6.0;
pub const RADIUS_L: f32 = 8.0;

/// The iced theme for a light or dark appearance.
pub fn theme(dark: bool) -> Theme {
    let t = if dark { DARK } else { LIGHT };
    Theme::custom(
        if dark { "Coffer Dark" } else { "Coffer Light" }.to_owned(),
        iced::theme::Palette {
            background: t.bg,
            text: t.text,
            primary: t.primary,
            success: t.success,
            warning: t.warning,
            danger: t.danger,
        },
    )
}

/// The tokens of the active theme.
pub fn tokens(theme: &Theme) -> &'static Tokens {
    if theme.extended_palette().is_dark { &DARK } else { &LIGHT }
}

fn border(color: Color, width: f32, radius: f32) -> Border {
    Border { color, width, radius: Radius::from(radius) }
}

fn shadow(t: &Tokens, y: f32, blur: f32) -> Shadow {
    Shadow { color: t.shadow, offset: Vector::new(0.0, y), blur_radius: blur }
}

/// Semantic tones for banners, badges and toasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

impl Tone {
    pub fn colors(self, t: &Tokens) -> (Color, Color) {
        match self {
            Self::Neutral => (t.raised, t.muted),
            Self::Accent => (t.primary_soft, t.primary),
            Self::Success => (t.success_soft, t.success),
            Self::Warning => (t.warning_soft, t.warning),
            Self::Danger => (t.danger_soft, t.danger),
        }
    }
}

// Containers -----------------------------------------------------------------

pub fn app(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style { background: Some(t.bg.into()), text_color: Some(t.text), ..Default::default() }
}

pub fn sidebar(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style {
        background: Some(t.sidebar.into()),
        border: Border { color: t.border, width: 1.0, radius: 0.0.into() },
        ..Default::default()
    }
}

/// A group of related rows: flat, outlined, no shadow.
pub fn card(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style {
        background: Some(t.surface.into()),
        border: border(t.border, 1.0, RADIUS_L),
        ..Default::default()
    }
}

/// A side panel next to the main content (the archive details).
pub fn side_panel(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style { background: Some(t.sidebar.into()), ..Default::default() }
}

/// A hairline between rows and regions.
pub fn hairline(theme: &Theme) -> iced::widget::rule::Style {
    iced::widget::rule::Style {
        color: tokens(theme).border,
        radius: 0.0.into(),
        fill_mode: iced::widget::rule::FillMode::Full,
        snap: true,
    }
}

pub fn panel(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style { background: Some(t.surface.into()), border: border(t.border, 1.0, RADIUS), ..Default::default() }
}

pub fn inset(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style { background: Some(t.raised.into()), border: border(t.border, 1.0, RADIUS), ..Default::default() }
}

pub fn toolbar(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style { background: Some(t.bg.into()), ..Default::default() }
}

pub fn table_header(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style { text_color: Some(t.muted), ..Default::default() }
}

/// The track of a segmented control.
pub fn segmented(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style { background: Some(t.raised.into()), border: border(t.border, 1.0, RADIUS), ..Default::default() }
}

pub fn drop_zone(active: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let t = tokens(theme);
        container::Style {
            background: Some(if active { t.primary_soft } else { t.surface }.into()),
            border: Border {
                color: if active { t.primary } else { t.border_strong },
                width: if active { 2.0 } else { 1.0 },
                radius: RADIUS_L.into(),
            },
            ..Default::default()
        }
    }
}

pub fn tinted(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let t = tokens(theme);
        let (bg, fg) = tone.colors(t);
        container::Style {
            background: Some(bg.into()),
            text_color: Some(fg),
            border: border(Color { a: 0.35, ..fg }, 1.0, RADIUS),
            ..Default::default()
        }
    }
}

pub fn badge(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let t = tokens(theme);
        let (bg, fg) = tone.colors(t);
        container::Style {
            background: Some(bg.into()),
            text_color: Some(fg),
            border: border(Color::TRANSPARENT, 0.0, 999.0),
            ..Default::default()
        }
    }
}

pub fn icon_tile(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let t = tokens(theme);
        let (_, fg) = tone.colors(t);
        container::Style {
            background: Some(t.raised.into()),
            text_color: Some(fg),
            border: border(t.border, 1.0, RADIUS),
            ..Default::default()
        }
    }
}

pub fn backdrop(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style { background: Some(t.backdrop.into()), ..Default::default() }
}

pub fn modal(theme: &Theme) -> container::Style {
    let t = tokens(theme);
    container::Style {
        background: Some(t.surface.into()),
        border: border(t.border_strong, 1.0, RADIUS_L),
        shadow: shadow(t, 12.0, 32.0),
        text_color: Some(t.text),
        ..Default::default()
    }
}

pub fn toast(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let t = tokens(theme);
        let (_, fg) = tone.colors(t);
        container::Style {
            background: Some(t.surface.into()),
            text_color: Some(t.text),
            border: Border { color: Color { a: 0.55, ..fg }, width: 1.0, radius: RADIUS_L.into() },
            shadow: shadow(t, 6.0, 20.0),
            ..Default::default()
        }
    }
}

/// A table row: only the selection is tinted, so the eye follows the text.
pub fn row(selected: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let t = tokens(theme);
        let background = selected.then(|| t.primary_soft.into());
        container::Style { background, border: border(Color::TRANSPARENT, 0.0, RADIUS_S), ..Default::default() }
    }
}

// Text colours ----------------------------------------------------------------

pub fn muted(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style { color: Some(tokens(theme).muted) }
}

pub fn subtle(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style { color: Some(tokens(theme).subtle) }
}

pub fn accent(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style { color: Some(tokens(theme).primary) }
}

pub fn tone_text(tone: Tone) -> impl Fn(&Theme) -> iced::widget::text::Style {
    move |theme| iced::widget::text::Style { color: Some(tone.colors(tokens(theme)).1) }
}

// Buttons ---------------------------------------------------------------------

fn button_style(bg: Option<Color>, fg: Color, line: Color, radius: f32) -> button::Style {
    button::Style {
        background: bg.map(Background::from),
        text_color: fg,
        border: border(line, if line == Color::TRANSPARENT { 0.0 } else { 1.0 }, radius),
        shadow: Shadow::default(),
        snap: true,
    }
}

#[allow(clippy::large_types_passed_by_value)] // iced styles are values
fn disabled(style: button::Style) -> button::Style {
    button::Style {
        background: style.background.map(|b| b.scale_alpha(0.45)),
        text_color: style.text_color.scale_alpha(0.45),
        border: Border { color: style.border.color.scale_alpha(0.45), ..style.border },
        ..style
    }
}

pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let t = tokens(theme);
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => t.primary_hover,
        _ => t.primary,
    };
    let style = button_style(Some(bg), t.on_primary, Color::TRANSPARENT, RADIUS);
    if status == button::Status::Disabled { disabled(style) } else { style }
}

pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let t = tokens(theme);
    let (bg, line) = match status {
        button::Status::Hovered | button::Status::Pressed => (t.raised, t.border_strong),
        _ => (t.surface, t.border),
    };
    let style = button_style(Some(bg), t.text, line, RADIUS);
    if status == button::Status::Disabled { disabled(style) } else { style }
}

pub fn danger(theme: &Theme, status: button::Status) -> button::Style {
    let t = tokens(theme);
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => t.danger,
        _ => t.danger_soft,
    };
    let fg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::WHITE,
        _ => t.danger,
    };
    let style = button_style(Some(bg), fg, Color::TRANSPARENT, RADIUS);
    if status == button::Status::Disabled { disabled(style) } else { style }
}

pub fn ghost(theme: &Theme, status: button::Status) -> button::Style {
    let t = tokens(theme);
    let style = match status {
        button::Status::Hovered | button::Status::Pressed => {
            button_style(Some(t.raised), t.text, Color::TRANSPARENT, RADIUS_S)
        }
        _ => button_style(None, t.muted, Color::TRANSPARENT, RADIUS_S),
    };
    if status == button::Status::Disabled { disabled(style) } else { style }
}

pub fn link(theme: &Theme, status: button::Status) -> button::Style {
    let t = tokens(theme);
    let fg = match status {
        button::Status::Hovered | button::Status::Pressed => t.primary_hover,
        _ => t.primary,
    };
    button_style(None, fg, Color::TRANSPARENT, 0.0)
}

pub fn nav(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let t = tokens(theme);
        // A wash of the text colour reads on both the light and the dark sidebar.
        let wash = |a: f32| Color { a, ..t.text };
        let style = if active {
            button_style(Some(wash(0.08)), t.text, Color::TRANSPARENT, RADIUS)
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => {
                    button_style(Some(wash(0.045)), t.text, Color::TRANSPARENT, RADIUS)
                }
                _ => button_style(None, t.muted, Color::TRANSPARENT, RADIUS),
            }
        };
        if status == button::Status::Disabled { disabled(style) } else { style }
    }
}

/// A toggle that can be on alongside others (the checksum algorithms).
pub fn chip(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let t = tokens(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let style = if active {
            button_style(Some(t.primary_soft), t.primary, Color { a: 0.45, ..t.primary }, RADIUS)
        } else if hovered {
            button_style(Some(t.raised), t.text, t.border_strong, RADIUS)
        } else {
            button_style(Some(t.surface), t.muted, t.border, RADIUS)
        };
        if status == button::Status::Disabled { disabled(style) } else { style }
    }
}

/// One option of a segmented control: the chosen one is raised out of the track.
pub fn segment(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let t = tokens(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let mut style = if active {
            button_style(Some(t.surface), t.text, t.border_strong, RADIUS_S)
        } else if hovered {
            button_style(None, t.text, Color::TRANSPARENT, RADIUS_S)
        } else {
            button_style(None, t.muted, Color::TRANSPARENT, RADIUS_S)
        };
        if active {
            style.shadow = shadow(t, 1.0, 2.0);
        }
        if status == button::Status::Disabled { disabled(style) } else { style }
    }
}

pub fn list_row(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let t = tokens(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if selected {
            Some(t.primary_soft)
        } else if hovered {
            Some(t.raised)
        } else {
            None
        };
        button_style(bg, t.text, Color::TRANSPARENT, 0.0)
    }
}

// Inputs ----------------------------------------------------------------------

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let t = tokens(theme);
    let line = match status {
        text_input::Status::Focused { .. } => t.primary,
        text_input::Status::Hovered => t.border_strong,
        _ => t.border,
    };
    text_input::Style {
        background: if matches!(status, text_input::Status::Disabled) { t.raised } else { t.surface }.into(),
        border: border(line, 1.0, RADIUS),
        icon: t.muted,
        placeholder: t.subtle,
        value: t.text,
        selection: Color { a: 0.35, ..t.primary },
    }
}

pub fn select(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let t = tokens(theme);
    let line = match status {
        pick_list::Status::Opened { .. } => t.primary,
        pick_list::Status::Hovered => t.border_strong,
        pick_list::Status::Active => t.border,
    };
    pick_list::Style {
        text_color: t.text,
        placeholder_color: t.subtle,
        handle_color: t.muted,
        background: t.surface.into(),
        border: border(line, 1.0, RADIUS),
    }
}

pub fn switch(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let t = tokens(theme);
    let on =
        matches!(status, toggler::Status::Active { is_toggled: true } | toggler::Status::Hovered { is_toggled: true });
    let mut style = toggler::default(theme, status);
    style.background = if on { t.primary } else { t.border_strong }.into();
    style.foreground = if on { t.on_primary } else { t.surface }.into();
    style
}

pub fn progress(tone: Tone) -> impl Fn(&Theme) -> progress_bar::Style {
    move |theme| {
        let t = tokens(theme);
        let bar = match tone {
            Tone::Success => t.success,
            Tone::Danger => t.danger,
            Tone::Warning => t.warning,
            _ => t.primary,
        };
        progress_bar::Style {
            background: t.raised.into(),
            bar: bar.into(),
            border: border(Color::TRANSPARENT, 0.0, 999.0),
        }
    }
}

pub fn scroll(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let t = tokens(theme);
    let mut style = scrollable::default(theme, status);
    let hovered = matches!(
        status,
        scrollable::Status::Hovered { is_vertical_scrollbar_hovered: true, .. } | scrollable::Status::Dragged { .. }
    );
    for rail in [&mut style.vertical_rail, &mut style.horizontal_rail] {
        rail.background = None;
        rail.border = border(Color::TRANSPARENT, 0.0, 999.0);
        rail.scroller.background = if hovered { t.subtle } else { t.border_strong }.into();
        rail.scroller.border = border(Color::TRANSPARENT, 0.0, 999.0);
    }
    style
}
