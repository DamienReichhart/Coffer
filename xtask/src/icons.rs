//! `cargo xtask icons`: every icon the app and its packages need, rendered
//! from `assets/brand/icon.svg`. The outputs are committed, so builds need
//! no SVG renderer; re-run this after changing the logo.

use std::path::Path;

use anyhow::{Context, Result};
use resvg::{tiny_skia, usvg};

const LINUX_SIZES: [u32; 8] = [16, 24, 32, 48, 64, 128, 256, 512];
const ICO_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];
/// (ICNS type, pixel size).
const ICNS: [(&[u8; 4], u32); 10] = [
    (b"icp4", 16),
    (b"icp5", 32),
    (b"icp6", 64),
    (b"ic07", 128),
    (b"ic08", 256),
    (b"ic09", 512),
    (b"ic10", 1024),
    (b"ic11", 32),
    (b"ic12", 64),
    (b"ic13", 256),
];

pub fn render(root: &Path) -> Result<()> {
    let svg = std::fs::read(root.join("assets/brand/icon.svg"))?;
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default()).context("invalid SVG")?;

    let linux = root.join("packaging/linux/icons");
    std::fs::create_dir_all(&linux)?;
    for size in LINUX_SIZES {
        std::fs::write(linux.join(format!("{size}.png")), png(&tree, size)?)?;
    }
    std::fs::copy(root.join("assets/brand/icon.svg"), linux.join("scalable.svg"))?;
    std::fs::write(root.join("crates/app/assets/icon-256.png"), png(&tree, 256)?)?;

    let windows = root.join("packaging/windows");
    std::fs::create_dir_all(&windows)?;
    std::fs::write(windows.join("coffer.ico"), ico(&tree)?)?;

    let macos = root.join("packaging/macos");
    std::fs::create_dir_all(&macos)?;
    std::fs::write(macos.join("AppIcon.icns"), icns(&tree)?)?;

    eprintln!("Rendered the icons into packaging/ and crates/app/assets/.");
    Ok(())
}

fn png(tree: &usvg::Tree, size: u32) -> Result<Vec<u8>> {
    let mut pixmap = tiny_skia::Pixmap::new(size, size).context("pixmap")?;
    let scale = size as f32 / tree.size().width();
    resvg::render(tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    pixmap.encode_png().context("PNG encoding")
}

/// A Windows icon holding PNG images (supported since Windows Vista).
fn ico(tree: &usvg::Tree) -> Result<Vec<u8>> {
    let images: Vec<(u32, Vec<u8>)> = ICO_SIZES.iter().map(|&s| png(tree, s).map(|p| (s, p))).collect::<Result<_>>()?;
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]);
    out.extend_from_slice(&(images.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * images.len() as u32;
    for (size, data) in &images {
        let dim = if *size >= 256 { 0 } else { *size as u8 };
        out.extend_from_slice(&[dim, dim, 0, 0]);
        out.extend_from_slice(&1_u16.to_le_bytes());
        out.extend_from_slice(&32_u16.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += data.len() as u32;
    }
    for (_, data) in &images {
        out.extend_from_slice(data);
    }
    Ok(out)
}

/// A macOS icon file with PNG entries.
fn icns(tree: &usvg::Tree) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    for (kind, size) in ICNS {
        let data = png(tree, size)?;
        body.extend_from_slice(kind);
        body.extend_from_slice(&((data.len() + 8) as u32).to_be_bytes());
        body.extend_from_slice(&data);
    }
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(b"icns");
    out.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}
