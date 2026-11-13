//! Coffer: a modern, native desktop interface for 7-Zip, built with iced.
//!
//! The binary (`src/main.rs`) only parses the command line and calls [`run`].
//! Everything else lives here so the screenshot generator
//! (`examples/screenshots.rs`) and the tests drive the very same code.

pub mod app;
pub mod cli;
pub mod demo;
pub mod fonts;
pub mod icons;
pub mod jobs;
pub mod logging;
pub mod screens;
pub mod settings;
pub mod theme;
pub mod ui;
pub mod update;

pub use app::{App, Message};

/// The product name, as shown to people.
pub const APP_NAME: &str = "Coffer";
/// The running version (the workspace version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The website and the pages the app links to.
pub const WEBSITE: &str = "https://damienreichhart.github.io/Coffer/";
pub const DOCS: &str = "https://damienreichhart.github.io/Coffer/docs/";
pub const DOWNLOAD_PAGE: &str = "https://damienreichhart.github.io/Coffer/download/";
pub const CHANGELOG: &str = "https://damienreichhart.github.io/Coffer/changelog/";
pub const REPOSITORY: &str = "https://github.com/DamienReichhart/Coffer";
pub const ISSUES: &str = "https://github.com/DamienReichhart/Coffer/issues/new/choose";

/// Starts the application with what the command line asked for.
pub fn run(launch: cli::Launch) -> iced::Result {
    iced::application(move || App::new(launch.clone()), App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .settings(iced::Settings {
            id: Some("io.github.damienreichhart.Coffer".into()),
            default_font: fonts::REGULAR,
            default_text_size: iced::Pixels(14.0),
            antialiasing: true,
            ..iced::Settings::default()
        })
        .font(fonts::REGULAR_BYTES)
        .font(fonts::MEDIUM_BYTES)
        .font(fonts::SEMIBOLD_BYTES)
        .window(window_settings())
        .exit_on_close_request(false)
        .run()
}

fn window_settings() -> iced::window::Settings {
    iced::window::Settings {
        size: iced::Size::new(1180.0, 760.0),
        min_size: Some(iced::Size::new(880.0, 580.0)),
        icon: window_icon(),
        platform_specific: platform_window(),
        ..iced::window::Settings::default()
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_window() -> iced::window::settings::PlatformSpecific {
    iced::window::settings::PlatformSpecific {
        // Matches the desktop entry, so docks group the window with the launcher.
        application_id: "io.github.damienreichhart.Coffer".into(),
        ..Default::default()
    }
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn platform_window() -> iced::window::settings::PlatformSpecific {
    iced::window::settings::PlatformSpecific::default()
}

fn window_icon() -> Option<iced::window::Icon> {
    let decoder = png::Decoder::new(std::io::Cursor::new(include_bytes!("../assets/icon-256.png")));
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buffer).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buffer.truncate(info.buffer_size());
    iced::window::icon::from_rgba(buffer, info.width, info.height).ok()
}
