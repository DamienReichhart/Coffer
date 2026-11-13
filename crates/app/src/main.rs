// No console window behind the app on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() -> iced::Result {
    let cli = coffer::cli::Cli::parse();
    let _guard = coffer::logging::init();
    tracing::info!(version = coffer::VERSION, "starting Coffer");
    coffer::run(cli.into_launch())
}
