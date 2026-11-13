---
title: Building from source
description: Build and run Coffer from its source code.
---

Coffer is a Rust workspace; the website is an Astro project in `website/`.

## Requirements

- **Rust** stable, 1.92 or newer, from [rustup](https://rustup.rs) (the repository’s
  `rust-toolchain.toml` selects the right toolchain).
- **7-Zip**, to use the app and to run the integration tests — see
  [Installing 7-Zip](/docs/installing-7zip/).
- **Linux only**, the libraries iced links against:

  ```sh
  # Debian, Ubuntu
  sudo apt install build-essential pkg-config libxkbcommon-dev libwayland-dev \
    libx11-dev libxcursor-dev libxrandr-dev libxi-dev libvulkan-dev libfontconfig1-dev
  # Fedora
  sudo dnf install gcc pkgconf-pkg-config libxkbcommon-devel wayland-devel \
    libX11-devel libXcursor-devel libXrandr-devel libXi-devel vulkan-loader-devel fontconfig-devel
  ```

- **Windows:** Visual Studio Build Tools with the “Desktop development with C++” workload.
- **macOS:** Xcode Command Line Tools (`xcode-select --install`).

## Run

```sh
git clone https://github.com/DamienReichhart/Coffer.git
cd Coffer
cargo run -p coffer
```

A release build:

```sh
cargo build --release -p coffer
./target/release/coffer
```

## Repository layout

| Path | Contents |
| --- | --- |
| `crates/core` | Finding and driving 7-Zip: command lines, listings, progress. |
| `crates/updater` | The update manifest, verification and installers. |
| `crates/app` | The iced application. |
| `xtask` | Packaging and release tooling (`cargo xtask …`). |
| `packaging/` | Desktop entry, AppStream metadata, WiX and macOS bundle files, public keys. |
| `website/` | This website. |

`docs/ARCHITECTURE.md` in the repository describes the contracts between them.

## Checks

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Tests that need a real 7-Zip find it the same way the app does, and are skipped when there is none.

## Packages

```sh
cargo xtask package --help
```

builds the installers for your platform (deb, rpm and AppImage on Linux; MSI and portable zip on
Windows; dmg on macOS), as the release workflow does.

## Screenshots and the website

The screenshots on this site are rendered by the app itself, headlessly, from demo data:

```sh
cargo run --release -p coffer --features demo --example screenshots -- website/src/assets/screenshots
cd website
npm ci
npm run dev
```
