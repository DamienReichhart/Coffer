// No console window behind the app on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() -> iced::Result {
    let cli = sevenzip_gui::cli::Cli::parse();
    let _guard = sevenzip_gui::logging::init();
    tracing::info!(version = sevenzip_gui::VERSION, "starting 7zip-GUI");
    sevenzip_gui::run(cli.into_launch())
}
