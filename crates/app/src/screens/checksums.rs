//! Checksums of files and folders (`7z h`), with verification against a
//! known value.

use std::path::PathBuf;

use coffer_core::hash::FileHashes;
use coffer_core::{HashAlgorithm, Version};
use iced::widget::{button, column, container, progress_bar, row, text};
use iced::{Alignment, Length, Task};

use crate::app::{App, Message};
use crate::icons::{Icon, icon};
use crate::jobs::{JobId, Spec, Status};
use crate::theme::{self, Tone};
use crate::ui::{self, El};

#[derive(Debug, Clone)]
pub enum Msg {
    AddDialog,
    Add(Vec<PathBuf>),
    Remove(usize),
    Clear,
    Toggle(HashAlgorithm),
    Compare(String),
    Compute,
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub files: Vec<PathBuf>,
    pub algorithms: Vec<HashAlgorithm>,
    pub available: Vec<HashAlgorithm>,
    pub compare: String,
    pub job: Option<JobId>,
    pub results: Vec<FileHashes>,
    pub error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            algorithms: vec![HashAlgorithm::Crc32, HashAlgorithm::Sha256],
            available: HashAlgorithm::ALL.to_vec(),
            compare: String::new(),
            job: None,
            results: Vec::new(),
            error: None,
        }
    }
}

impl State {
    /// Offers only the algorithms the installed 7-Zip supports.
    pub fn restrict_to(&mut self, version: Option<Version>) {
        let Some(version) = version else { return };
        self.available = HashAlgorithm::ALL.into_iter().filter(|a| a.supported_by(version)).collect();
        self.algorithms.retain(|a| self.available.contains(a));
        if self.algorithms.is_empty() {
            self.algorithms.push(HashAlgorithm::Sha256);
        }
    }

    pub fn finished(&mut self, id: JobId, output: Option<&str>) {
        if self.job != Some(id) {
            return;
        }
        match output.map(coffer_core::hash::parse) {
            Some(Ok(results)) => self.results = results,
            Some(Err(error)) => self.error = Some(error.to_string()),
            None => {}
        }
    }

    pub fn failed(&mut self, id: JobId, message: String) {
        if self.job == Some(id) {
            self.error = Some(message);
        }
    }

    /// The digest that matches the comparison value, if any.
    fn matches(&self, digest: &str) -> bool {
        let wanted = self.compare.trim();
        !wanted.is_empty() && digest.eq_ignore_ascii_case(wanted)
    }
}

impl App {
    pub(crate) fn update_checksums(&mut self, msg: Msg) -> Task<Message> {
        let state = &mut self.checksums;
        match msg {
            Msg::AddDialog => {
                return Task::perform(rfd::AsyncFileDialog::new().set_title("Choose files").pick_files(), |files| {
                    let paths: Vec<PathBuf> =
                        files.unwrap_or_default().iter().map(|f| f.path().to_path_buf()).collect();
                    Message::Checksums(Msg::Add(paths))
                });
            }
            Msg::Add(paths) => {
                for path in paths {
                    if !state.files.contains(&path) {
                        state.files.push(path);
                    }
                }
                state.results.clear();
                state.error = None;
            }
            Msg::Remove(i) => {
                if i < state.files.len() {
                    state.files.remove(i);
                    state.results.clear();
                }
            }
            Msg::Clear => {
                state.files.clear();
                state.results.clear();
                state.error = None;
            }
            Msg::Toggle(algorithm) => {
                if let Some(i) = state.algorithms.iter().position(|a| *a == algorithm) {
                    if state.algorithms.len() > 1 {
                        state.algorithms.remove(i);
                    }
                } else {
                    state.algorithms.push(algorithm);
                    state.algorithms.sort_by_key(|a| HashAlgorithm::ALL.iter().position(|b| b == a));
                }
                state.results.clear();
            }
            Msg::Compare(value) => state.compare = value,
            Msg::Compute => {
                if state.files.is_empty() {
                    return Task::none();
                }
                state.results.clear();
                state.error = None;
                let spec = Spec::Hash { files: state.files.clone(), algorithms: state.algorithms.clone() };
                let task = self.start(spec);
                self.checksums.job = self.jobs.list.first().map(|j| j.id);
                return task;
            }
        }
        Task::none()
    }
}

pub fn view(app: &App) -> El<'_> {
    let state = &app.checksums;
    let running = state.job.and_then(|id| app.jobs.get(id)).filter(|j| j.status == Status::Running);
    let header = ui::page_header(
        "Checksums",
        "Compute and verify the fingerprints of files, to check a download or a copy.",
        vec![],
    );

    let files: El<'_> = if state.files.is_empty() {
        container(
            column![
                ui::icon_tile(Icon::Hash, Tone::Accent, 44.0),
                ui::strong("Drop files or folders here"),
                ui::small("Folders are hashed file by file.")
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .padding(24)
        .center_x(Length::Fill)
        .style(theme::drop_zone(false))
        .into()
    } else {
        column(state.files.iter().enumerate().map(|(i, p)| {
            row![
                icon(if p.is_dir() { Icon::Folder } else { Icon::File }, 17.0),
                text(p.display().to_string()).width(Length::Fill),
                ui::icon_button(Icon::X, "Remove", Some(Message::Checksums(Msg::Remove(i)))),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        }))
        .spacing(6)
        .into()
    };

    let algorithms = row(state.available.iter().map(|a| {
        button(text(a.to_string()).size(13))
            .padding([6, 12])
            .style(theme::chip(state.algorithms.contains(a)))
            .on_press(Message::Checksums(Msg::Toggle(*a)))
            .into()
    }))
    .spacing(6)
    .wrap();

    let compute = ui::primary(
        Some(Icon::Hash),
        "Compute",
        (!state.files.is_empty() && running.is_none()).then_some(Message::Checksums(Msg::Compute)),
    );
    let input = column![
        files,
        row![
            ui::secondary(Some(Icon::FilePlus), "Add files…", Some(Message::Checksums(Msg::AddDialog))),
            ui::ghost(None, "Clear", (!state.files.is_empty()).then_some(Message::Checksums(Msg::Clear))),
            ui::fill(),
            compute,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(14);

    let mut body = column![
        ui::section("Files", None, input.into()),
        ui::section(
            "Algorithms",
            Some("Choose one or more. Those your 7-Zip doesn't support are hidden."),
            column![
                algorithms,
                ui::field(
                    "Verify against",
                    Some("Paste a published checksum: the matching line turns green."),
                    ui::input("Expected checksum", &state.compare, |v| Message::Checksums(Msg::Compare(v))).into(),
                ),
            ]
            .spacing(16)
            .into(),
        ),
    ]
    .spacing(ui::L);

    if let Some(job) = running {
        body = body.push(ui::section(
            "Computing…",
            None,
            column![
                progress_bar(0.0..=100.0, f32::from(job.percent)).girth(6).style(theme::progress(Tone::Accent)),
                ui::small(job.current.clone().unwrap_or_default()),
            ]
            .spacing(8)
            .into(),
        ));
    }
    if let Some(error) = &state.error {
        body = body.push(ui::banner(Tone::Danger, Icon::CircleX, text(error.as_str()).into(), vec![]));
    }
    if !state.results.is_empty() {
        body = body.push(results(state));
    }
    ui::page(header, body.into())
}

fn results(state: &State) -> El<'_> {
    let any_match = state.results.iter().any(|f| f.digests.iter().any(|(_, d)| state.matches(d)));
    let mut col = column![].spacing(14);
    if !state.compare.trim().is_empty() {
        col = col.push(if any_match {
            ui::banner(
                Tone::Success,
                Icon::CircleCheck,
                ui::strong("Match: the checksum below is identical.").into(),
                vec![],
            )
        } else {
            ui::banner(Tone::Danger, Icon::CircleX, ui::strong("No match: no file has this checksum.").into(), vec![])
        });
    }
    for file in &state.results {
        let mut lines = column![
            row![
                icon(Icon::File, 16.0),
                ui::strong(file.path.as_str()).width(Length::Fill),
                ui::small(file.size.map(coffer_core::format::bytes).unwrap_or_default()),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        ]
        .spacing(6);
        for (name, digest) in &file.digests {
            let matched = state.matches(digest);
            lines = lines.push(
                row![
                    ui::small(name.as_str()).width(Length::Fixed(90.0)),
                    container(ui::mono(digest.as_str()).style(move |t: &iced::Theme| {
                        let tokens = theme::tokens(t);
                        iced::widget::text::Style { color: Some(if matched { tokens.success } else { tokens.text }) }
                    }))
                    .padding([5, 10])
                    .width(Length::Fill)
                    .style(move |t: &iced::Theme| if matched {
                        theme::tinted(Tone::Success)(t)
                    } else {
                        theme::inset(t)
                    }),
                    ui::icon_button(Icon::Copy, "Copy", Some(Message::Copy(digest.clone()))),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }
        col = col.push(lines);
    }
    ui::section("Results", Some(results_caption(state.results.len())), col.into())
}

fn results_caption(n: usize) -> &'static str {
    match n {
        1 => "1 file",
        _ => "Every file, in order",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_algorithms_the_installed_7zip_lacks() {
        let mut state = State::default();
        state.algorithms.push(HashAlgorithm::Md5);
        state.restrict_to(Some(Version::new(23, 1)));
        assert!(!state.available.contains(&HashAlgorithm::Md5));
        assert!(!state.algorithms.contains(&HashAlgorithm::Md5));
        assert!(state.algorithms.contains(&HashAlgorithm::Sha256));
    }

    #[test]
    fn comparison_ignores_case_and_spaces() {
        let state = State { compare: "  ABCDEF ".into(), ..State::default() };
        assert!(state.matches("abcdef"));
        assert!(!state.matches("abcde0"));
    }
}
