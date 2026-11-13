//! Every operation of this session, with live progress, cancel, logs and
//! "show in folder".

use iced::widget::{Space, column, container, progress_bar, row, scrollable, text};
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
        "Operations of this session. Cancelling stops 7-Zip at once; files it already wrote stay.",
        vec![
            ui::ghost(Some(Icon::Trash2), "Clear finished", finished.then_some(Message::Activity(Msg::ClearFinished)))
                .into(),
        ],
    );
    if app.jobs.list.is_empty() {
        return column![
            container(header).padding([28, 32]),
            ui::empty_state(
                Icon::Activity,
                Tone::Neutral,
                "Nothing has run yet",
                "Extractions, new archives and tests show their progress here.",
                vec![]
            ),
        ]
        .into();
    }
    let jobs = column(app.jobs.list.iter().map(job_card)).spacing(ui::M);
    ui::page(header, jobs.into())
}

fn job_card(job: &Job) -> El<'_> {
    let (tone, status_label) = match &job.status {
        Status::Running => (Tone::Accent, format!("{}%", job.percent)),
        Status::Succeeded { warnings: false } => (Tone::Success, "Done".to_owned()),
        Status::Succeeded { warnings: true } => (Tone::Warning, "Done with warnings".to_owned()),
        Status::Failed(_) => (Tone::Danger, "Failed".to_owned()),
        Status::Cancelled => (Tone::Neutral, "Cancelled".to_owned()),
    };
    let mut actions = row![].spacing(6).align_y(Alignment::Center);
    match &job.status {
        Status::Running => {
            actions =
                actions.push(ui::secondary(Some(Icon::X), "Cancel", Some(Message::Activity(Msg::Cancel(job.id)))));
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
                actions =
                    actions.push(ui::secondary(Some(Icon::FolderOpen), "Show in folder", Some(Message::Reveal(dir))));
            }
        }
    }
    actions = actions.push(ui::ghost(
        None,
        if job.show_log { "Hide log" } else { "Log" },
        Some(Message::Activity(Msg::ToggleLog(job.id))),
    ));

    let mut card = column![
        row![
            ui::icon_tile(job.spec.icon(), tone, 40.0),
            column![ui::strong(job.spec.title()), ui::small(job.spec.detail())].spacing(2).width(Length::Fill),
            column![ui::badge(status_label, tone), ui::small(coffer_core::format::duration(job.elapsed()))]
                .spacing(4)
                .align_x(Alignment::End),
            actions,
        ]
        .spacing(14)
        .align_y(Alignment::Center),
    ]
    .spacing(12);

    if job.is_running() {
        card =
            card.push(progress_bar(0.0..=100.0, f32::from(job.percent)).girth(6).style(theme::progress(Tone::Accent)));
        card = card.push(ui::small(job.current.clone().unwrap_or_else(|| "Starting…".to_owned())));
    }
    if let Status::Failed(error) = &job.status {
        card = card.push(ui::banner(Tone::Danger, Icon::CircleX, text(error.message.as_str()).into(), vec![]));
    }
    if job.show_log {
        let lines = if job.log.is_empty() {
            "No output.".to_owned()
        } else {
            job.log.iter().cloned().collect::<Vec<_>>().join("\n")
        };
        card = card.push(
            container(
                scrollable(container(ui::mono(lines).size(12)).padding(12))
                    .style(theme::scroll)
                    .height(Length::Fixed(180.0)),
            )
            .width(Length::Fill)
            .style(theme::inset),
        );
    }
    container(card).padding(18).width(Length::Fill).style(theme::card).into()
}

#[allow(dead_code)]
fn spacer() -> Space {
    Space::new()
}
