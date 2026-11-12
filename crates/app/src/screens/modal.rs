//! Dialogs over the window: password, confirmations, rename and errors.

use iced::widget::{Space, center, column, container, mouse_area, opaque, row, text, text_input};
use iced::{Alignment, Length, Task};

use crate::app::{App, Message};
use crate::icons::Icon;
use crate::jobs::{Spec, plural};
use crate::screens::browse;
use crate::theme::{self, Tone};
use crate::ui::{self, El};

/// What a password is for.
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordFor {
    Open(std::path::PathBuf),
    Retry(Box<Spec>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    Password { purpose: PasswordFor, value: String, wrong: bool, show: bool },
    ConfirmDelete { entries: Vec<String> },
    Rename { from: String, value: String },
    ConfirmQuit { running: usize },
    Error { title: String, message: String },
}

impl Modal {
    pub fn password(purpose: PasswordFor, wrong: bool) -> Self {
        Self::Password { purpose, value: String::new(), wrong, show: false }
    }

    pub fn confirm_delete(entries: Vec<String>) -> Self {
        Self::ConfirmDelete { entries }
    }

    pub fn rename(from: String) -> Self {
        let value = from.rsplit('/').next().unwrap_or(&from).to_owned();
        Self::Rename { from, value }
    }

    pub fn confirm_quit(running: usize) -> Self {
        Self::ConfirmQuit { running }
    }

    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error { title: title.into(), message: message.into() }
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    Input(String),
    ToggleShow(bool),
    Submit,
    Close,
}

const INPUT_ID: &str = "modal-input";

impl App {
    pub(crate) fn update_modal(&mut self, msg: Msg) -> Task<Message> {
        let Some(modal) = self.modal.as_mut() else { return Task::none() };
        match msg {
            Msg::Input(new) => match modal {
                Modal::Password { value, .. } | Modal::Rename { value, .. } => *value = new,
                _ => {}
            },
            Msg::ToggleShow(on) => {
                if let Modal::Password { show, .. } = modal {
                    *show = on;
                }
            }
            Msg::Close => {
                self.modal = None;
            }
            Msg::Submit => {
                let Some(modal) = self.modal.take() else { return Task::none() };
                return match modal {
                    Modal::Password { purpose, value, .. } => {
                        if value.is_empty() {
                            self.modal = Some(Modal::Password { purpose, value, wrong: false, show: false });
                            return Task::none();
                        }
                        match purpose {
                            PasswordFor::Open(path) => self.open(path, Some(value)),
                            PasswordFor::Retry(spec) => self.start(spec.with_password(value)),
                        }
                    }
                    Modal::ConfirmDelete { entries } => self.update_browse(browse::Msg::DeleteConfirmed(entries)),
                    Modal::Rename { from, value } => {
                        let value = value.trim().to_owned();
                        let Some(browser) = &self.browser else { return Task::none() };
                        if value.is_empty() || value.contains('/') || value.contains('\\') {
                            self.modal = Some(Modal::Rename { from, value });
                            return Task::none();
                        }
                        let to = match from.rsplit_once('/') {
                            Some((dir, _)) => format!("{dir}/{value}"),
                            None => value,
                        };
                        if to == from {
                            return Task::none();
                        }
                        let spec = Spec::Rename {
                            archive: browser.path.clone(),
                            from,
                            to,
                            password: browser.password.clone(),
                        };
                        self.start(spec)
                    }
                    Modal::ConfirmQuit { .. } => Task::done(Message::Quit),
                    Modal::Error { .. } => Task::none(),
                };
            }
        }
        Task::none()
    }
}

fn dialog<'a>(i: Icon, tone: Tone, title: String, body: El<'a>, actions: Vec<El<'a>>) -> El<'a> {
    container(
        column![
            row![ui::icon_tile(i, tone, 40.0), ui::heading(title).size(17).width(Length::Fill)]
                .spacing(14)
                .align_y(Alignment::Center),
            body,
            row![Space::new().width(Length::Fill), row(actions).spacing(8)].align_y(Alignment::Center),
        ]
        .spacing(18),
    )
    .padding(24)
    .width(Length::Fixed(460.0))
    .style(theme::modal)
    .into()
}

pub fn view(modal: &Modal) -> El<'_> {
    let cancel = || ui::secondary(None, "Cancel", Some(Message::Modal(Msg::Close))).into();
    let content = match modal {
        Modal::Password { purpose, value, wrong, show } => {
            let what = match purpose {
                PasswordFor::Open(path) => {
                    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
                }
                PasswordFor::Retry(spec) => spec.title(),
            };
            let mut body = column![
                ui::muted(format!("{what} is encrypted. Enter its password to continue.")),
                text_input("Password", value)
                    .id(INPUT_ID)
                    .secure(!show)
                    .on_input(|v| Message::Modal(Msg::Input(v)))
                    .on_submit(Message::Modal(Msg::Submit))
                    .padding([9, 12])
                    .style(theme::input),
                iced::widget::checkbox(*show)
                    .label("Show the password")
                    .on_toggle(|v| Message::Modal(Msg::ToggleShow(v)))
                    .size(16),
            ]
            .spacing(12);
            if *wrong {
                body = body.push(ui::banner(
                    Tone::Danger,
                    Icon::CircleX,
                    text("That password is not correct.").into(),
                    vec![],
                ));
            }
            dialog(
                Icon::Lock,
                Tone::Accent,
                "Password required".into(),
                body.into(),
                vec![
                    cancel(),
                    ui::primary(None, "Unlock", (!value.is_empty()).then_some(Message::Modal(Msg::Submit))).into(),
                ],
            )
        }
        Modal::ConfirmDelete { entries } => {
            let preview: Vec<String> = entries.iter().take(5).cloned().collect();
            let more = entries.len().saturating_sub(preview.len());
            let mut list = column(preview.into_iter().map(|e| ui::mono(e).into())).spacing(2);
            if more > 0 {
                list = list.push(ui::small(format!("and {more} more")));
            }
            dialog(
                Icon::Trash2,
                Tone::Danger,
                format!("Delete {} from the archive?", plural(entries.len(), "item")),
                column![
                    container(list).padding(12).width(Length::Fill).style(theme::inset),
                    ui::muted("This rewrites the archive and can't be undone."),
                ]
                .spacing(12)
                .into(),
                vec![cancel(), ui::danger(Some(Icon::Trash2), "Delete", Some(Message::Modal(Msg::Submit))).into()],
            )
        }
        Modal::Rename { from, value } => dialog(
            Icon::Pencil,
            Tone::Accent,
            "Rename".into(),
            column![
                ui::small(from.clone()),
                text_input("New name", value)
                    .id(INPUT_ID)
                    .on_input(|v| Message::Modal(Msg::Input(v)))
                    .on_submit(Message::Modal(Msg::Submit))
                    .padding([9, 12])
                    .style(theme::input),
            ]
            .spacing(10)
            .into(),
            vec![cancel(), ui::primary(None, "Rename", Some(Message::Modal(Msg::Submit))).into()],
        ),
        Modal::ConfirmQuit { running } => dialog(
            Icon::TriangleAlert,
            Tone::Warning,
            format!("{} still running", plural(*running, "operation")),
            ui::muted("Quitting cancels them. Files they already wrote are kept, and may be incomplete.").into(),
            vec![cancel(), ui::danger(None, "Cancel them and quit", Some(Message::Modal(Msg::Submit))).into()],
        ),
        Modal::Error { title, message } => dialog(
            Icon::CircleX,
            Tone::Danger,
            title.clone(),
            ui::muted(message.as_str()).into(),
            vec![ui::primary(None, "OK", Some(Message::Modal(Msg::Close))).into()],
        ),
    };
    opaque(mouse_area(center(opaque(content)).style(theme::backdrop)).on_press(Message::Modal(Msg::Close)))
}

/// Focuses the dialog's text field.
pub fn focus() -> Task<Message> {
    iced::widget::operation::focus(INPUT_ID)
}
