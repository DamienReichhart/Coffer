//! The in-app side of automatic updates: checking in the background, the
//! banner, downloading with progress, and handing over to the installer.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use iced::Task;
use iced::futures::SinkExt;
use sevenzip_updater::{Handover, Progress, Update};

use crate::app::Message;

/// Where the updater is.
#[derive(Debug, Clone, Default)]
pub enum State {
    #[default]
    Idle,
    Checking {
        manual: bool,
    },
    UpToDate,
    Available(Update),
    Downloading {
        update: Update,
        progress: Progress,
    },
    Installing(Update),
    /// The installer was opened for the user to finish (Linux fallback).
    HandedOver,
    Failed(String),
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)] // one message per check, not a hot path
pub enum Msg {
    /// Check now. `manual` checks report "up to date"; automatic ones stay quiet.
    Check {
        manual: bool,
    },
    Checked {
        manual: bool,
        result: Result<Option<Update>, String>,
    },
    Install,
    Progress(Progress),
    Done(Result<Handover, String>),
    Cancel,
    Skip,
    Dismiss,
}

/// The updater's state plus what it needs between messages.
#[derive(Debug)]
pub struct Updater {
    pub state: State,
    pub dismissed: bool,
    cancel: Arc<AtomicBool>,
    exe: PathBuf,
}

impl Default for Updater {
    fn default() -> Self {
        Self {
            state: State::Idle,
            dismissed: false,
            cancel: Arc::new(AtomicBool::new(false)),
            // Captured at startup: once a package replaces the binary, Linux
            // reports the old path with " (deleted)".
            exe: std::env::current_exe().unwrap_or_default(),
        }
    }
}

/// What the app should do after an updater message.
#[derive(Debug)]
pub enum Effect {
    None,
    Task(Task<Message>),
    /// The installer has taken over: quit.
    Quit,
    Toast(String),
}

impl Updater {
    /// The update to advertise in the banner, unless dismissed or skipped.
    pub fn banner(&self, skipped: Option<&str>) -> Option<&State> {
        if self.dismissed {
            return None;
        }
        match &self.state {
            State::Available(update) if skipped == Some(update.version.to_string().as_str()) => None,
            state @ (State::Available(_) | State::Downloading { .. } | State::Installing(_)) => Some(state),
            _ => None,
        }
    }

    pub fn update(&mut self, msg: Msg) -> Effect {
        match msg {
            Msg::Check { manual } => {
                if matches!(self.state, State::Checking { .. } | State::Downloading { .. } | State::Installing(_)) {
                    return Effect::None;
                }
                self.state = State::Checking { manual };
                if manual {
                    self.dismissed = false;
                }
                Effect::Task(Task::perform(check(), move |result| Message::Update(Msg::Checked { manual, result })))
            }
            Msg::Checked { manual, result } => match result {
                Ok(Some(update)) => {
                    tracing::info!(version = %update.version, "an update is available");
                    self.state = State::Available(update);
                    Effect::None
                }
                Ok(None) => {
                    self.state = State::UpToDate;
                    if manual {
                        Effect::Toast(format!("{} {} is the latest version.", crate::APP_NAME, crate::VERSION))
                    } else {
                        Effect::None
                    }
                }
                Err(error) => {
                    tracing::warn!(%error, "update check failed");
                    self.state = State::Failed(error.clone());
                    if manual { Effect::Toast(format!("Couldn't check for updates: {error}")) } else { Effect::None }
                }
            },
            Msg::Install => {
                let State::Available(update) = self.state.clone() else { return Effect::None };
                let Some(asset) = update.asset.clone() else {
                    // A development build, or no installer of this kind: the website takes over.
                    let _ = open::that_detached(crate::DOWNLOAD_PAGE);
                    return Effect::None;
                };
                let Some(kind) = update.kind else { return Effect::None };
                self.cancel.store(false, Ordering::Relaxed);
                self.state = State::Downloading {
                    update: update.clone(),
                    progress: Progress { received: 0, total: asset.size },
                };
                let cancel = self.cancel.clone();
                let exe = self.exe.clone();
                let version = update.version.clone();
                let stream = iced::stream::channel(
                    32,
                    async move |mut output: iced::futures::channel::mpsc::Sender<Message>| {
                        let (tx, mut rx) = iced::futures::channel::mpsc::unbounded::<Progress>();
                        let worker = tokio::task::spawn_blocking(move || {
                            let agent = sevenzip_updater::agent();
                            let file = sevenzip_updater::download(
                                &agent,
                                &asset,
                                &version,
                                &crate::settings::paths::update_dir(),
                                sevenzip_updater::PUBLIC_KEY,
                                &cancel,
                                |p| {
                                    let _ = tx.unbounded_send(p);
                                },
                            )
                            .map_err(|e| e.to_string())?;
                            sevenzip_updater::install(kind, &file, &exe).map_err(|e| e.to_string())
                        });
                        while let Some(p) = iced::futures::StreamExt::next(&mut rx).await {
                            let _ = output.send(Message::Update(Msg::Progress(p))).await;
                        }
                        let result = worker.await.unwrap_or_else(|e| Err(e.to_string()));
                        let _ = output.send(Message::Update(Msg::Done(result))).await;
                    },
                );
                Effect::Task(Task::stream(stream))
            }
            Msg::Progress(progress) => {
                if let State::Downloading { update, .. } = &self.state {
                    let update = update.clone();
                    self.state = if progress.received >= progress.total {
                        State::Installing(update)
                    } else {
                        State::Downloading { update, progress }
                    };
                }
                Effect::None
            }
            Msg::Done(result) => match result {
                Ok(handover) if handover.quit => Effect::Quit,
                Ok(_) => {
                    self.state = State::HandedOver;
                    Effect::Toast("The installer has opened. Finish the update there, then restart 7zip-GUI.".into())
                }
                Err(error) => {
                    let update = match &self.state {
                        State::Downloading { update, .. } | State::Installing(update) => Some(update.clone()),
                        _ => None,
                    };
                    if error == "cancelled" {
                        self.state = update.map_or(State::Idle, State::Available);
                        return Effect::None;
                    }
                    tracing::error!(%error, "update failed");
                    self.state = update.map_or_else(|| State::Failed(error.clone()), State::Available);
                    Effect::Toast(format!("The update failed: {error}"))
                }
            },
            Msg::Cancel => {
                self.cancel.store(true, Ordering::Relaxed);
                Effect::None
            }
            Msg::Skip | Msg::Dismiss => {
                self.dismissed = true;
                Effect::None
            }
        }
    }
}

async fn check() -> Result<Option<Update>, String> {
    tokio::task::spawn_blocking(|| {
        let current: semver::Version = crate::VERSION.parse().map_err(|e: semver::Error| e.to_string())?;
        sevenzip_updater::check(&sevenzip_updater::agent(), &sevenzip_updater::manifest_url(), &current)
            .map_err(|e| e.to_string())
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string()))
}
