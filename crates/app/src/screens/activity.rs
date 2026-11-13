//! Every operation of this session, with live progress, cancel, logs and
//! "show in folder".

use iced::widget::{column, container, progress_bar, row, scrollable, text};
use iced::{Alignment, Length, Task};

use crate::app::{App, Message};
use crate::icons::Icon;
use crate::jobs::{Job, JobId, Status};
use crate::theme::{self, Tone};
use crate::ui::{self, El};

#[derive(Debug, Clone)]
pub enum Msg {
    Cancel(JobId),
    Retry(JobId),
    ToggleLog(JobId),
    ClearFinished,
}

impl App {
    pub(crate) fn update_activity(&mut self, msg: Msg) -> Task<Message> {
        match msg {
            Msg::Cancel(id) => {
                if let Some(job) = self.jobs.get_mut(id) {
                    job.cancel();
                }
                Task::none()
            }
            Msg::Retry(id) => {
                let Some(spec) = self.jobs.get(id).map(|j| j.spec.clone()) else { return Task::none() };
                self.start(spec)
            }
            Msg::ToggleLog(id) => {
                if let Some(job) = self.jobs.get_mut(id) {
                    job.show_log = !job.show_log;
                }
                Task::none()
            }
            Msg::ClearFinished => {
                self.jobs.clear_finished();
                Task::none()
            }
        }
    }
}

pub fn view(app: &App) -> El<'_> {
    let finished = app.jobs.list.iter().any(|j| !j.is_running());
    let header = ui::page_header(
        "Activity",
        "This session's operations. Cancelling stops 7-Zip at once; files it already wrote stay.",
        vec![
            ui::ghost(Some(Icon::Trash2), "Clear finished", finished.then_some(Message::Activity(Msg::ClearFinished)))
                .into(),
        ],
    );
    if app.jobs.list.is_empty() {
        return ui::page(
            header,
            container(ui::empty_state(
                Icon::Activity,
                Tone::Neutral,
                "Nothing has run yet",
                "Extractions, new archives and tests show their progress here.",
                vec![],
            ))
            .height(Length::Fixed(320.0))
            .into(),
        );
    }
    ui::page(header, ui::list(app.jobs.list.iter().map(job_row).collect()))
}

fn job_row(job: &Job) -> El<'_> {
    let (tone, status_label) = match &job.status {
        Status::Running => (Tone::Accent, format!("{}%", job.percent)),
        Status::Succeeded { warnings: false } => (Tone::Success, "Done".to_owned()),
        Status::Succeeded { warnings: true } => (Tone::Warning, "Done with warnings".to_owned()),
        Status::Failed(_) => (Tone::Danger, "Failed".to_owned()),
        Status::Cancelled => (Tone::Neutral, "Cancelled".to_owned()),
    };
    let mut actions = row![].spacing(4).align_y(Alignment::Center);
    match &job.status {
        Status::Running => {
            actions = actions.push(ui::secondary(None, "Cancel", Some(Message::Activity(Msg::Cancel(job.id)))));
        }
        Status::Failed(_) | Status::Cancelled => {
            actions = actions.push(ui::secondary(
                Some(Icon::RotateCcw),
                "Retry",
                Some(Message::Activity(Msg::Retry(job.id))),
            ));
        }
        Status::Succeeded { .. } => {
            if let Some(dir) = job.spec.output_folder() {
                actions = actions.push(ui::secondary(Some(Icon::FolderOpen), "Show", Some(Message::Reveal(dir))));
            }
        }
    }
    actions = actions.push(ui::ghost(
        None,
        if job.show_log { "Hide log" } else { "Log" },
        Some(Message::Activity(Msg::ToggleLog(job.id))),
    ));

    let pick: fn(&crate::theme::Tokens) -> iced::Color = match tone {
        Tone::Success => |t| t.success,
        Tone::Warning => |t| t.warning,
        Tone::Danger => |t| t.danger,
        Tone::Accent => |t| t.primary,
        Tone::Neutral => |t| t.subtle,
    };
    let mut body = column![
        row![
            crate::icons::tinted(job.spec.icon(), 18.0, pick),
            column![
                text(job.spec.title()).size(14).font(crate::fonts::MEDIUM),
                ui::small(job.spec.detail()).wrapping(text::Wrapping::None),
            ]
            .spacing(1)
            .width(Length::Fill)
            .clip(true),
            column![
                text(status_label).size(12).font(crate::fonts::MEDIUM).style(theme::tone_text(tone)),
                ui::small(coffer_core::format::duration(job.elapsed())),
            ]
            .spacing(1)
            .align_x(Alignment::End),
            actions,
        ]
        .spacing(14)
        .align_y(Alignment::Center),
    ]
    .spacing(10);

    if job.is_running() {
        body = body.push(
            column![
                progress_bar(0.0..=100.0, f32::from(job.percent)).girth(4).style(theme::progress(Tone::Accent)),
                ui::small(job.current.clone().unwrap_or_else(|| "Starting…".to_owned())).wrapping(text::Wrapping::None),
            ]
            .spacing(6)
            .padding(iced::Padding::ZERO.left(32.0)),
        );
    }
    if let Status::Failed(error) = &job.status {
        body = body.push(
            container(text(error.message.as_str()).size(13).style(theme::tone_text(Tone::Danger)))
                .padding(iced::Padding::ZERO.left(32.0)),
        );
    }
    if job.show_log {
        let lines = if job.log.is_empty() {
            "No output.".to_owned()
        } else {
            job.log.iter().cloned().collect::<Vec<_>>().join("\n")
        };
        body = body.push(
            container(
                scrollable(container(ui::mono(lines).size(12)).padding(12))
                    .style(theme::scroll)
                    .height(Length::Fixed(180.0)),
            )
            .width(Length::Fill)
            .style(theme::inset),
        );
    }
    container(body).padding([12, 14]).width(Length::Fill).into()
}
