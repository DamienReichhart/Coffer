//! The bundled Inter typeface (SIL Open Font License, `assets/fonts/OFL.txt`).
//!
//! Bundling it makes the interface — and the generated screenshots — look the
//! same on every platform.

use iced::Font;
use iced::font::{Family, Weight};

pub const REGULAR_BYTES: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
pub const MEDIUM_BYTES: &[u8] = include_bytes!("../assets/fonts/Inter-Medium.ttf");
pub const SEMIBOLD_BYTES: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");

pub const REGULAR: Font = Font::with_name("Inter");
pub const MEDIUM: Font = Font { family: Family::Name("Inter"), weight: Weight::Medium, ..Font::DEFAULT };
pub const SEMIBOLD: Font = Font { family: Family::Name("Inter"), weight: Weight::Semibold, ..Font::DEFAULT };
pub const MONO: Font = Font::MONOSPACE;
