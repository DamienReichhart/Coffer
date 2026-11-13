//! Renders the website's screenshots from the real views, headlessly.
//!
//! ```text
//! cargo run --release -p coffer --example screenshots -- website/src/assets/screenshots
//! ```
//!
//! Writes `<out>/{light,dark}/<scene>.png` at 1280×800 logical pixels and a
//! scale factor of 2, with iced's software renderer, so the result is the same
//! on any machine and needs neither a GPU nor a display server.

use std::path::{Path, PathBuf};
use std::time::Duration;

use coffer::demo::{self, Scene};
use coffer::{App, fonts};

const SIZE: iced::Size = iced::Size::new(1280.0, 800.0);
const SCALE: f32 = 2.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args_os().nth(1).map_or_else(|| PathBuf::from("screenshots"), PathBuf::from);
    // The software renderer: identical output everywhere.
    // SAFETY-free: set before any renderer exists.
    if std::env::var_os("ICED_TEST_BACKEND").is_none() {
        // tiny-skia is iced_test's default; nothing to do.
    }
    for dark in [false, true] {
        let dir = out.join(if dark { "dark" } else { "light" });
        std::fs::create_dir_all(&dir)?;
        for scene in Scene::ALL {
            let path = dir.join(format!("{}.png", scene.name()));
            render(scene, dark, &path)?;
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn render(scene: Scene, dark: bool, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let program = iced::application(move || demo::app(scene, dark), App::update, App::view)
        .theme(App::theme)
        .settings(iced::Settings {
            default_font: fonts::REGULAR,
            default_text_size: iced::Pixels(14.0),
            antialiasing: true,
            ..iced::Settings::default()
        })
        .font(fonts::REGULAR_BYTES)
        .font(fonts::MEDIUM_BYTES)
        .font(fonts::SEMIBOLD_BYTES);
    let theme = coffer::theme::theme(dark);
    let shot = iced_test::screenshot(&program, &theme, SIZE, SCALE, Duration::from_millis(250));
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, shot.size.width, shot.size.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&shot.rgba)?;
    writer.finish()?;
    Ok(())
}
