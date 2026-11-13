//! The application state, its messages and the main update loop.
//!
//! Each screen keeps its own state and message type in `screens/`, and adds
//! its update function to [`App`] with an `impl App` block there; this file
//! only routes messages and owns what screens share: 7-Zip, jobs, modals,
//! toasts, drag-and-drop and the updater.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use coffer_core::command::Dialect;
use coffer_core::{Listing, SevenZip, runner};
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget::{column, container, row, stack};
use iced::{Element, Event, Length, Subscription, Task, Theme, event, window};

use crate::cli::Launch;
use crate::jobs::{AppError, JobId, Jobs, Spec, Status};
use crate::screens::{self, activity, browse, checksums, compress, extract, modal, preferences};
use crate::settings::Settings;
use crate::theme::{ThemeChoice, Tone};
use crate::update::{self, Updater};

/// The screens of the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    Home,
    Browse,
    Compress,
    Extract,
    Checksums,
    Activity,
    Settings,
    About,
}

/// Whether 7-Zip is usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SevenState {
    Detecting,
    Ready(SevenZip),
    Missing,
    Invalid { path: PathBuf, error: String },
}

/// A short-lived notification in the corner.
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub tone: Tone,
    pub text: String,
    pub action: Option<(String, Message)>,
    pub created: Instant,
}

const TOAST_LIFETIME: Duration = Duration::from_secs(7);

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Screen),
    Located(Result<SevenZip, String>),
    Relocate,
    OpenDialog,
    Open(PathBuf, Option<String>),
    Opened { path: PathBuf, password: Option<String>, result: Result<Listing, AppError> },
    CloseArchive,
    Browse(browse::Msg),
    Compress(compress::Msg),
    Extract(extract::Msg),
    Checksums(checksums::Msg),
    Preferences(preferences::Msg),
    Activity(activity::Msg),
    Modal(modal::Msg),
    Update(update::Msg),
    Job(JobId, coffer_core::Event),
    SystemTheme(iced::theme::Mode),
    FileHovered,
    FilesHoveredLeft,
    FileDropped(PathBuf),
    ProcessDrops,
    KeyPressed(Key, Modifiers),
    ModifiersChanged(Modifiers),
    CloseRequested(window::Id),
    Quit,
    Tick,
    DismissToast(u64),
    OpenUrl(String),
    Reveal(PathBuf),
    OpenFile(PathBuf),
    Copy(String),
    Nothing,
}

/// The whole application.
#[derive(Debug)]
pub struct App {
    pub settings: Settings,
    pub seven: SevenState,
    pub screen: Screen,
    pub browser: Option<browse::Browser>,
    pub opening: Option<PathBuf>,
    pub compress: compress::Form,
    pub extract: extract::Form,
    pub checksums: checksums::State,
    pub jobs: Jobs,
    pub modal: Option<modal::Modal>,
    pub toasts: Vec<Toast>,
    pub updater: Updater,
    pub system_dark: bool,
    pub drop_hover: bool,
    pub pending_drops: Vec<PathBuf>,
    pub modifiers: Modifiers,
    pub launch: Option<Launch>,
    /// The 7-Zip path being typed in Settings, applied on Enter.
    pub seven_path_input: String,
    next_toast: u64,
}

impl App {
    /// A fresh state with the given settings, before any task has run.
    pub fn with_settings(settings: Settings) -> Self {
        let compress = compress::Form::from_defaults(&settings.compress);
        let extract = extract::Form::from_defaults(&settings.extract);
        let seven_path_input = settings.seven_zip_path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
        Self {
            seven_path_input,
            settings,
            seven: SevenState::Detecting,
            screen: Screen::Home,
            browser: None,
            opening: None,
            compress,
            extract,
            checksums: checksums::State::default(),
            jobs: Jobs::default(),
            modal: None,
            toasts: Vec::new(),
            updater: Updater::default(),
            system_dark: true,
            drop_hover: false,
            pending_drops: Vec::new(),
            modifiers: Modifiers::default(),
            launch: None,
            next_toast: 0,
        }
    }

    pub fn new(launch: Launch) -> (Self, Task<Message>) {
        let mut app = Self::with_settings(Settings::load());
        app.launch = Some(launch);
        let mut tasks = vec![app.locate(), iced::system::theme().map(Message::SystemTheme)];
        if app.settings.updates.check_automatically {
            // A moment after startup, so the first frame is never delayed.
            tasks.push(Task::perform(tokio::time::sleep(Duration::from_secs(4)), |()| {
                Message::Update(update::Msg::Check { manual: false })
            }));
        }
        (app, Task::batch(tasks))
    }

    pub fn title(&self) -> String {
        match (&self.browser, self.screen) {
            (Some(b), Screen::Browse) => format!("{} — {}", b.name(), crate::APP_NAME),
            _ => crate::APP_NAME.to_owned(),
        }
    }

    pub fn is_dark(&self) -> bool {
        match self.settings.theme {
            ThemeChoice::System => self.system_dark,
            ThemeChoice::Light => false,
            ThemeChoice::Dark => true,
        }
    }

    pub fn theme(&self) -> Theme {
        crate::theme::theme(self.is_dark())
    }

    pub fn seven(&self) -> Option<&SevenZip> {
        match &self.seven {
            SevenState::Ready(seven) => Some(seven),
            _ => None,
        }
    }

    fn locate(&self) -> Task<Message> {
        let configured = self.settings.seven_zip_path.clone();
        Task::perform(
            async move { SevenZip::locate(configured.as_deref()).await.map_err(|e| e.to_string()) },
            Message::Located,
        )
    }

    pub fn toast(&mut self, tone: Tone, text: impl Into<String>, action: Option<(String, Message)>) {
        self.next_toast += 1;
        self.toasts.push(Toast { id: self.next_toast, tone, text: text.into(), action, created: Instant::now() });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    pub fn save_settings(&self) {
        self.settings.save();
    }

    /// Starts a job, or explains why it can't.
    pub fn start(&mut self, spec: Spec) -> Task<Message> {
        let Some(seven) = self.seven().cloned() else {
            self.toast(Tone::Danger, "7-Zip is not available. Install it or set its location in Settings.", None);
            return Task::none();
        };
        let args = spec.args(Dialect::of(&seven));
        let archive = spec.archive().map(Path::to_path_buf);
        let id = self.jobs.add(spec);
        let stream = runner::run(seven, args, archive);
        let (task, handle) = Task::run(stream, move |event| Message::Job(id, event)).abortable();
        if let Some(job) = self.jobs.get_mut(id) {
            job.handle = Some(handle);
        }
        task
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(screen) => {
                self.screen = screen;
                Task::none()
            }
            Message::Located(result) => {
                self.seven = match result {
                    Ok(seven) => {
                        tracing::info!(path = %seven.path.display(), version = %seven.version, "found 7-Zip");
                        SevenState::Ready(seven)
                    }
                    Err(error) => match &self.settings.seven_zip_path {
                        Some(path) => SevenState::Invalid { path: path.clone(), error },
                        None => SevenState::Missing,
                    },
                };
                self.checksums.restrict_to(self.seven().map(|s| s.version));
                match self.launch.take() {
                    Some(launch) => self.apply_launch(launch),
                    None => Task::none(),
                }
            }
            Message::Relocate => {
                self.seven = SevenState::Detecting;
                self.locate()
            }
            Message::OpenDialog => Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Open an archive")
                    .add_filter("Archives", ARCHIVE_EXTENSIONS)
                    .add_filter("All files", &["*"])
                    .pick_file(),
                |file| file.map_or(Message::Nothing, |f| Message::Open(f.path().to_path_buf(), None)),
            ),
            Message::Open(path, password) => self.open(path, password),
            Message::Opened { path, password, result } => self.opened(path, password, result),
            Message::CloseArchive => {
                self.browser = None;
                if self.screen == Screen::Browse {
                    self.screen = Screen::Home;
                }
                Task::none()
            }
            Message::Browse(msg) => self.update_browse(msg),
            Message::Compress(msg) => self.update_compress(msg),
            Message::Extract(msg) => self.update_extract(msg),
            Message::Checksums(msg) => self.update_checksums(msg),
            Message::Preferences(msg) => self.update_preferences(msg),
            Message::Activity(msg) => self.update_activity(msg),
            Message::Modal(msg) => self.update_modal(msg),
            Message::Update(msg) => match self.updater.update(msg) {
                update::Effect::None => Task::none(),
                update::Effect::Task(task) => task,
                update::Effect::Toast(text) => {
                    self.toast(Tone::Neutral, text, None);
                    Task::none()
                }
                update::Effect::Quit => {
                    self.jobs.cancel_all();
                    iced::exit()
                }
            },
            Message::Job(id, event) => self.job_event(id, event),
            Message::SystemTheme(mode) => {
                self.system_dark = mode != iced::theme::Mode::Light;
                Task::none()
            }
            Message::FileHovered => {
                self.drop_hover = true;
                Task::none()
            }
            Message::FilesHoveredLeft => {
                self.drop_hover = false;
                Task::none()
            }
            Message::FileDropped(path) => {
                self.drop_hover = false;
                let first = self.pending_drops.is_empty();
                self.pending_drops.push(path);
                if first {
                    // Several files arrive as several events: gather them first.
                    Task::perform(tokio::time::sleep(Duration::from_millis(120)), |()| Message::ProcessDrops)
                } else {
                    Task::none()
                }
            }
            Message::ProcessDrops => {
                let drops = std::mem::take(&mut self.pending_drops);
                self.dropped(drops)
            }
            Message::KeyPressed(key, modifiers) => self.shortcut(&key, modifiers),
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }
            Message::CloseRequested(id) => {
                let running = self.jobs.running();
                if running == 0 {
                    return window::close(id);
                }
                self.modal = Some(modal::Modal::confirm_quit(running));
                Task::none()
            }
            Message::Quit => {
                self.jobs.cancel_all();
                iced::exit()
            }
            Message::Tick => {
                self.toasts.retain(|t| t.created.elapsed() < TOAST_LIFETIME);
                Task::none()
            }
            Message::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
                Task::none()
            }
            Message::OpenUrl(url) => {
                if let Err(error) = open::that_detached(&url) {
                    self.toast(Tone::Danger, format!("Couldn't open {url}: {error}"), None);
                }
                Task::none()
            }
            Message::Reveal(path) => {
                if let Err(error) = open::that_detached(&path) {
                    self.toast(Tone::Danger, format!("Couldn't open {}: {error}", path.display()), None);
                }
                Task::none()
            }
            Message::OpenFile(path) => {
                if let Err(error) = open::that_detached(&path) {
                    self.toast(Tone::Danger, format!("No application could open {}: {error}", path.display()), None);
                }
                Task::none()
            }
            Message::Copy(text) => {
                self.toast(Tone::Success, "Copied to the clipboard.", None);
                iced::clipboard::write(text)
            }
            Message::Nothing => Task::none(),
        }
    }

    fn apply_launch(&mut self, launch: Launch) -> Task<Message> {
        match launch {
            Launch::Home => Task::none(),
            Launch::Open(path) => self.open(path, None),
            Launch::Extract { archive, to } => {
                self.extract.set_archive(archive, &self.settings.extract);
                if let Some(to) = to {
                    self.extract.destination = to.display().to_string();
                    self.extract.into_folder = false;
                }
                self.screen = Screen::Extract;
                Task::none()
            }
            Launch::Compress(paths) => {
                self.compress.add_inputs(paths);
                self.screen = Screen::Compress;
                Task::none()
            }
        }
    }

    pub fn open(&mut self, path: PathBuf, password: Option<String>) -> Task<Message> {
        let Some(seven) = self.seven().cloned() else {
            self.toast(Tone::Danger, "7-Zip is not available, so archives can't be opened yet.", None);
            return Task::none();
        };
        // Any volume of a multi-volume archive opens the whole archive.
        let path = match coffer_core::volumes::detect(&path) {
            Some(set) if !set.is_complete() => {
                self.modal = Some(modal::Modal::error(
                    "Some volumes of this archive are missing",
                    missing_volumes_message(&set),
                ));
                return Task::none();
            }
            Some(set) => set.primary,
            None => path,
        };
        self.opening = Some(path.clone());
        self.screen = Screen::Browse;
        Task::perform(
            async move {
                let result = runner::list(&seven, &path, password.as_deref()).await.map_err(|e| AppError::from(&e));
                (path, password, result)
            },
            |(path, password, result)| Message::Opened { path, password, result },
        )
    }

    fn opened(&mut self, path: PathBuf, password: Option<String>, result: Result<Listing, AppError>) -> Task<Message> {
        self.opening = None;
        match result {
            Ok(listing) => {
                self.settings.remember(&path);
                self.save_settings();
                let keep_dir = self.browser.as_ref().filter(|b| b.path == path).map(|b| b.cwd.clone());
                let mut browser = browse::Browser::new(path, listing, password);
                if let Some(dir) = keep_dir.filter(|d| browser.tree.is_dir(d)) {
                    browser.cwd = dir;
                }
                self.browser = Some(browser);
                self.screen = Screen::Browse;
            }
            Err(error) if error.needs_password => {
                self.modal = Some(modal::Modal::password(modal::PasswordFor::Open(path), error.wrong_password));
                if self.browser.is_none() {
                    self.screen = Screen::Home;
                }
                return modal::focus();
            }
            Err(error) => {
                if self.browser.is_none() {
                    self.screen = Screen::Home;
                }
                self.modal = Some(modal::Modal::error("This archive can't be opened", error.message));
            }
        }
        Task::none()
    }

    fn job_event(&mut self, id: JobId, event: coffer_core::Event) -> Task<Message> {
        let Some(job) = self.jobs.get_mut(id) else { return Task::none() };
        match event {
            coffer_core::Event::Progress { percent, file } => {
                job.percent = percent;
                if file.is_some() {
                    job.current = file;
                }
                Task::none()
            }
            coffer_core::Event::Log(line) => {
                if let Some(file) = coffer_core::progress::logged_file(&line) {
                    job.current = Some(file.to_owned());
                }
                job.push_log(line);
                Task::none()
            }
            coffer_core::Event::Finished(result) => {
                if !job.is_running() {
                    return Task::none(); // already cancelled
                }
                job.finish(&result);
                let spec = job.spec.clone();
                let status = job.status.clone();
                let output = job.output.clone();
                self.job_finished(id, spec, status, output)
            }
        }
    }

    fn job_finished(&mut self, id: JobId, spec: Spec, status: Status, output: Option<String>) -> Task<Message> {
        let mut tasks = Vec::new();
        // Anything that changed the open archive invalidates its listing.
        if spec.modifies_archive()
            && matches!(status, Status::Succeeded { .. })
            && let Some(browser) = &self.browser
            && Some(browser.path.as_path()) == spec.archive()
        {
            tasks.push(self.open(browser.path.clone(), browser.password.clone()));
        }
        match (&spec, &status) {
            (_, Status::Failed(error)) if error.needs_password => {
                self.modal = Some(modal::Modal::password(
                    modal::PasswordFor::Retry(Box::new(spec.clone())),
                    error.wrong_password,
                ));
                tasks.push(modal::focus());
            }
            (Spec::Hash { .. }, Status::Succeeded { .. }) => {
                self.checksums.finished(id, output.as_deref());
            }
            (Spec::Hash { .. }, Status::Failed(error)) => {
                self.checksums.failed(id, error.message.clone());
            }
            (Spec::Preview { dir, entry, .. }, Status::Succeeded { .. }) => {
                let name = entry.rsplit('/').next().unwrap_or(entry);
                tasks.push(Task::done(Message::OpenFile(dir.join(name))));
            }
            (Spec::Extract { open_when_done: true, options, .. }, Status::Succeeded { .. }) => {
                tasks.push(Task::done(Message::Reveal(options.destination.clone())));
                self.toast(Tone::Success, spec.title() + " — done.", None);
            }
            (_, Status::Succeeded { warnings }) => {
                let action = spec.output_folder().map(|dir| ("Show".to_owned(), Message::Reveal(dir)));
                if *warnings {
                    self.toast(
                        Tone::Warning,
                        spec.title() + " — finished with warnings.",
                        Some(("Details".into(), Message::Navigate(Screen::Activity))),
                    );
                } else if matches!(spec, Spec::Test { .. }) {
                    self.toast(Tone::Success, spec.title() + ": no errors found.", None);
                } else {
                    self.toast(Tone::Success, spec.title() + " — done.", action);
                }
            }
            (_, Status::Failed(error)) => {
                self.toast(
                    Tone::Danger,
                    format!("{} failed: {}", spec.title(), error.message),
                    Some(("Details".into(), Message::Navigate(Screen::Activity))),
                );
            }
            _ => {}
        }
        Task::batch(tasks)
    }

    fn dropped(&mut self, paths: Vec<PathBuf>) -> Task<Message> {
        // The volumes of a split archive count as one archive.
        let paths = coffer_core::volumes::collapse(&paths);
        if paths.is_empty() {
            return Task::none();
        }
        match self.screen {
            Screen::Compress => {
                self.compress.add_inputs(paths);
                Task::none()
            }
            Screen::Checksums => self.update_checksums(checksums::Msg::Add(paths)),
            Screen::Extract if paths.len() == 1 => {
                self.extract.set_archive(paths[0].clone(), &self.settings.extract);
                Task::none()
            }
            Screen::Browse if self.browser.is_some() && !(paths.len() == 1 && looks_like_archive(&paths[0])) => {
                self.update_browse(browse::Msg::AddFiles(paths))
            }
            _ if paths.len() == 1 && paths[0].is_file() && looks_like_archive(&paths[0]) => {
                self.open(paths[0].clone(), None)
            }
            _ => {
                self.compress.add_inputs(paths);
                self.screen = Screen::Compress;
                Task::none()
            }
        }
    }

    fn shortcut(&mut self, key: &Key, modifiers: Modifiers) -> Task<Message> {
        if let Key::Named(Named::Escape) = key {
            if self.modal.is_some() {
                self.modal = None;
            }
            return Task::none();
        }
        if self.modal.is_some() {
            return Task::none();
        }
        let command = modifiers.command();
        match key.as_ref() {
            Key::Character("o") if command => self.update(Message::OpenDialog),
            Key::Character("n") if command => self.update(Message::Navigate(Screen::Compress)),
            Key::Character("e") if command => self.update(Message::Navigate(Screen::Extract)),
            Key::Character(",") if command => self.update(Message::Navigate(Screen::Settings)),
            Key::Character("w") if command && self.browser.is_some() => self.update(Message::CloseArchive),
            _ if self.screen == Screen::Browse => self.update_browse(browse::Msg::Key(key.clone(), modifiers)),
            _ => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            event::listen_with(|event, status, id| match event {
                Event::Window(window::Event::FileHovered(_)) => Some(Message::FileHovered),
                Event::Window(window::Event::FilesHoveredLeft) => Some(Message::FilesHoveredLeft),
                Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
                Event::Window(window::Event::CloseRequested) => Some(Message::CloseRequested(id)),
                Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => Some(Message::ModifiersChanged(m)),
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    let escape = key == Key::Named(Named::Escape);
                    (status == event::Status::Ignored || escape).then_some(Message::KeyPressed(key, modifiers))
                }
                _ => None,
            }),
            iced::system::theme_changes().map(Message::SystemTheme),
        ];
        if !self.toasts.is_empty() || self.jobs.running() > 0 {
            subscriptions.push(iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick));
        }
        if self.settings.updates.check_automatically {
            subscriptions.push(
                iced::time::every(Duration::from_hours(6))
                    .map(|_| Message::Update(update::Msg::Check { manual: false })),
            );
        }
        Subscription::batch(subscriptions)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match self.screen {
            Screen::Home => screens::home::view(self),
            Screen::Browse => browse::view(self),
            Screen::Compress => compress::view(self),
            Screen::Extract => extract::view(self),
            Screen::Checksums => checksums::view(self),
            Screen::Activity => activity::view(self),
            Screen::Settings => preferences::view(self),
            Screen::About => screens::about::view(self),
        };
        let main = column![screens::shell::update_banner(self), content].height(Length::Fill);
        let body = row![screens::shell::sidebar(self), container(main).width(Length::Fill).height(Length::Fill)];
        let mut layers = stack![container(body).style(crate::theme::app).width(Length::Fill).height(Length::Fill)];
        if self.drop_hover {
            layers = layers.push(screens::shell::drop_overlay(self));
        }
        if !self.toasts.is_empty() {
            layers = layers.push(screens::shell::toasts(self));
        }
        if let Some(modal) = &self.modal {
            layers = layers.push(modal::view(modal));
        }
        layers.into()
    }
}

/// File extensions 7-Zip opens, for the open dialog and drop handling.
pub const ARCHIVE_EXTENSIONS: &[&str] = &[
    "7z", "zip", "rar", "tar", "gz", "tgz", "bz2", "tbz", "tbz2", "xz", "txz", "lzma", "tlz", "zst", "tzst", "lz",
    "lz4", "z", "taz", "cab", "iso", "img", "wim", "swm", "esd", "arj", "lzh", "lha", "cpio", "rpm", "deb", "dmg",
    "jar", "war", "ear", "apk", "xpi", "epub", "docx", "xlsx", "pptx", "odt", "ods", "vhd", "vhdx", "vmdk", "qcow2",
    "squashfs", "chm", "msi", "001", "z01", "r00", "zipx", "ar", "a", "udf", "hfs", "fat", "ntfs",
];

/// Whether a path looks like an archive, or a volume of one, from its name.
pub fn looks_like_archive(path: &Path) -> bool {
    let known = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| ARCHIVE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()));
    known || path.file_name().and_then(|n| n.to_str()).and_then(coffer_core::volumes::parse_name).is_some()
}

/// Explains which volumes of a set are missing.
pub fn missing_volumes_message(set: &coffer_core::volumes::VolumeSet) -> String {
    const SHOWN: usize = 6;
    let mut names = set.missing.iter().take(SHOWN).cloned().collect::<Vec<_>>().join(", ");
    if set.missing.len() > SHOWN {
        names = format!("{names} and {} more", set.missing.len() - SHOWN);
    }
    format!(
        "{} is split into several files, and these aren't next to the others: {names}. \
         Put every volume in the same folder, then try again.",
        set.name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_names() {
        assert!(looks_like_archive(Path::new("a.7z")));
        assert!(looks_like_archive(Path::new("A.TAR.GZ")));
        assert!(looks_like_archive(Path::new("backup.zip.001")));
        assert!(looks_like_archive(Path::new("backup.zip.011")));
        assert!(looks_like_archive(Path::new("movie.part2.rar")));
        assert!(looks_like_archive(Path::new("movie.r07")));
        assert!(looks_like_archive(Path::new("site.z03")));
        assert!(!looks_like_archive(Path::new("notes.txt")));
        assert!(!looks_like_archive(Path::new("folder")));
    }
}
