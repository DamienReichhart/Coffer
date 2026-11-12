//! Creating an archive: the inputs, the output and every compression option.

use std::path::{Path, PathBuf};

use iced::widget::{Space, button, checkbox, column, container, row, scrollable, text};
use iced::{Alignment, Length, Task};
use sevenzip_core::{ArchiveFormat, CompressOptions, CompressionLevel, CompressionMethod, SplitSize};

use crate::app::{App, Message, Screen};
use crate::icons::{Icon, icon};
use crate::jobs::{Spec, plural};
use crate::settings::CompressDefaults;
use crate::theme::{self, Tone};
use crate::ui::{self, El};

/// A choice in a picker, with a readable label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice<T> {
    pub value: T,
    pub label: String,
}

impl<T> std::fmt::Display for Choice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    AddFilesDialog,
    AddFolderDialog,
    Add(Vec<PathBuf>),
    Remove(usize),
    ClearInputs,
    Output(String),
    BrowseOutput,
    Format(ArchiveFormat),
    Level(CompressionLevel),
    Method(Choice<Option<CompressionMethod>>),
    Dictionary(Choice<Option<u32>>),
    Threads(Choice<Option<u32>>),
    Solid(bool),
    Password(String),
    Confirm(String),
    ShowPassword(bool),
    EncryptNames(bool),
    ZipAes(bool),
    Split(Choice<Option<u64>>),
    CustomSplit(String),
    DeleteAfter(bool),
    Excludes(String),
    Create,
    Reset,
}

/// The form's state.
#[derive(Debug, Clone, PartialEq)]
pub struct Form {
    pub inputs: Vec<PathBuf>,
    pub output: String,
    /// The user typed or picked the output: stop suggesting one.
    pub output_touched: bool,
    pub format: ArchiveFormat,
    pub level: CompressionLevel,
    pub method: Option<CompressionMethod>,
    pub dictionary: Option<u32>,
    pub threads: Option<u32>,
    pub solid: bool,
    pub password: String,
    pub confirm: String,
    pub show_password: bool,
    pub encrypt_names: bool,
    pub zip_aes: bool,
    pub split: Option<u64>,
    pub custom_split: String,
    pub custom_split_on: bool,
    pub delete_after: bool,
    pub excludes: String,
}

impl Form {
    pub fn from_defaults(defaults: &CompressDefaults) -> Self {
        Self {
            inputs: Vec::new(),
            output: String::new(),
            output_touched: false,
            format: defaults.format,
            level: defaults.level,
            method: None,
            dictionary: None,
            threads: None,
            solid: defaults.solid,
            password: String::new(),
            confirm: String::new(),
            show_password: false,
            encrypt_names: defaults.encrypt_names,
            zip_aes: defaults.zip_aes,
            split: None,
            custom_split: String::new(),
            custom_split_on: false,
            delete_after: false,
            excludes: String::new(),
        }
    }

    pub fn add_inputs(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if !self.inputs.contains(&path) {
                self.inputs.push(path);
            }
        }
        self.suggest_output();
    }

    /// Proposes `<folder of the first input>/<name>.<ext>` until the user edits it.
    fn suggest_output(&mut self) {
        if self.output_touched {
            return;
        }
        let Some(first) = self.inputs.first() else {
            self.output.clear();
            return;
        };
        let dir = first.parent().map(Path::to_path_buf).unwrap_or_default();
        let base = if self.inputs.len() == 1 {
            let name = first.file_name().map_or_else(|| "Archive".into(), |n| n.to_string_lossy().into_owned());
            if self.format.is_single_file() || first.is_dir() {
                name
            } else {
                first.file_stem().map_or(name, |s| s.to_string_lossy().into_owned())
            }
        } else {
            dir.file_name().map_or_else(|| "Archive".to_owned(), |n| n.to_string_lossy().into_owned())
        };
        self.output = dir.join(format!("{base}.{}", self.format.extension())).display().to_string();
    }

    /// Changes the extension of the output to match the format.
    fn retarget_extension(&mut self) {
        if self.output.is_empty() {
            return;
        }
        let path = PathBuf::from(&self.output);
        let has_known = ArchiveFormat::from_path(&path).is_some();
        let new = if has_known {
            path.with_extension(self.format.extension())
        } else {
            PathBuf::from(format!("{}.{}", self.output, self.format.extension()))
        };
        self.output = new.display().to_string();
    }

    pub fn options(&self) -> CompressOptions {
        let split =
            if self.custom_split_on { SplitSize::parse(&self.custom_split) } else { self.split.map(SplitSize::Bytes) };
        CompressOptions {
            format: self.format,
            level: self.level,
            method: self.method,
            dictionary_mib: self.dictionary,
            solid: self.solid,
            threads: self.threads,
            password: (!self.password.is_empty()).then(|| self.password.clone()),
            encrypt_names: self.encrypt_names,
            zip_aes: self.zip_aes,
            split,
            delete_after: self.delete_after,
            excludes: self.excludes.split([',', ';']).map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect(),
        }
    }

    /// Why the form can't be submitted yet, if it can't.
    pub fn problem(&self) -> Option<String> {
        if self.inputs.is_empty() {
            return Some("Add the files and folders to compress.".into());
        }
        if self.output.trim().is_empty() {
            return Some("Choose where to save the archive.".into());
        }
        if self.password != self.confirm {
            return Some("The passwords don't match.".into());
        }
        if self.custom_split_on && SplitSize::parse(&self.custom_split).is_none() {
            return Some("Enter a volume size such as 250m or 2g.".into());
        }
        let output = PathBuf::from(self.output.trim());
        if self.inputs.contains(&output) {
            return Some("The archive can't be one of its own inputs.".into());
        }
        self.options().validate(&self.inputs).err()
    }
}

const DICTIONARIES: [u32; 9] = [1, 4, 16, 32, 64, 128, 256, 512, 1024];

fn method_choices(format: ArchiveFormat) -> Vec<Choice<Option<CompressionMethod>>> {
    let mut choices = vec![Choice {
        value: None,
        label: format!("Default ({})", format.methods().first().map_or("—".into(), ToString::to_string)),
    }];
    choices.extend(format.methods().iter().map(|m| Choice { value: Some(*m), label: m.to_string() }));
    choices
}

fn dictionary_choices() -> Vec<Choice<Option<u32>>> {
    let mut choices = vec![Choice { value: None, label: "Automatic".to_owned() }];
    choices.extend(DICTIONARIES.iter().map(|d| Choice {
        value: Some(*d),
        label: if *d >= 1024 { format!("{} GB", d / 1024) } else { format!("{d} MB") },
    }));
    choices
}

fn thread_choices() -> Vec<Choice<Option<u32>>> {
    let cpus = std::thread::available_parallelism().map_or(4, |n| n.get() as u32);
    let mut choices = vec![Choice { value: None, label: format!("Automatic ({cpus} cores)") }];
    choices.extend((1..=cpus.min(64)).map(|n| Choice { value: Some(n), label: plural(n as usize, "thread") }));
    choices
}

fn split_choices() -> Vec<Choice<Option<u64>>> {
    let mut choices = vec![Choice { value: None, label: "Don't split".to_owned() }];
    choices.extend(
        SplitSize::PRESETS.iter().map(|(label, bytes)| Choice { value: Some(*bytes), label: (*label).to_owned() }),
    );
    choices.push(Choice { value: Some(u64::MAX), label: "Custom size…".to_owned() });
    choices
}

impl App {
    pub(crate) fn update_compress(&mut self, msg: Msg) -> Task<Message> {
        let form = &mut self.compress;
        match msg {
            Msg::AddFilesDialog => {
                return Task::perform(rfd::AsyncFileDialog::new().set_title("Add files").pick_files(), |files| {
                    let paths: Vec<PathBuf> =
                        files.unwrap_or_default().iter().map(|f| f.path().to_path_buf()).collect();
                    Message::Compress(Msg::Add(paths))
                });
            }
            Msg::AddFolderDialog => {
                return Task::perform(rfd::AsyncFileDialog::new().set_title("Add folders").pick_folders(), |dirs| {
                    let paths: Vec<PathBuf> = dirs.unwrap_or_default().iter().map(|f| f.path().to_path_buf()).collect();
                    Message::Compress(Msg::Add(paths))
                });
            }
            Msg::Add(paths) => form.add_inputs(paths),
            Msg::Remove(i) => {
                if i < form.inputs.len() {
                    form.inputs.remove(i);
                    form.suggest_output();
                }
            }
            Msg::ClearInputs => {
                form.inputs.clear();
                form.suggest_output();
            }
            Msg::Output(value) => {
                form.output = value;
                form.output_touched = true;
                if let Some(format) = ArchiveFormat::from_path(Path::new(&form.output)) {
                    form.format = format;
                }
            }
            Msg::BrowseOutput => {
                let current = PathBuf::from(&form.output);
                let mut dialog = rfd::AsyncFileDialog::new().set_title("Save the archive as");
                if let Some(dir) = current.parent().filter(|d| d.is_dir()) {
                    dialog = dialog.set_directory(dir);
                }
                if let Some(name) = current.file_name() {
                    dialog = dialog.set_file_name(name.to_string_lossy());
                }
                return Task::perform(dialog.save_file(), |file| {
                    file.map_or(Message::Nothing, |f| Message::Compress(Msg::Output(f.path().display().to_string())))
                });
            }
            Msg::Format(format) => {
                form.format = format;
                form.method = None;
                if !format.supports_encryption() {
                    form.password.clear();
                    form.confirm.clear();
                }
                form.retarget_extension();
                form.suggest_output();
            }
            Msg::Level(level) => form.level = level,
            Msg::Method(choice) => form.method = choice.value,
            Msg::Dictionary(choice) => form.dictionary = choice.value,
            Msg::Threads(choice) => form.threads = choice.value,
            Msg::Solid(on) => form.solid = on,
            Msg::Password(p) => form.password = p,
            Msg::Confirm(p) => form.confirm = p,
            Msg::ShowPassword(on) => form.show_password = on,
            Msg::EncryptNames(on) => form.encrypt_names = on,
            Msg::ZipAes(on) => form.zip_aes = on,
            Msg::Split(choice) => {
                form.custom_split_on = choice.value == Some(u64::MAX);
                form.split = choice.value.filter(|v| *v != u64::MAX);
            }
            Msg::CustomSplit(value) => form.custom_split = value,
            Msg::DeleteAfter(on) => form.delete_after = on,
            Msg::Excludes(value) => form.excludes = value,
            Msg::Reset => *form = Form::from_defaults(&self.settings.compress),
            Msg::Create => {
                if form.problem().is_some() {
                    return Task::none();
                }
                let archive = PathBuf::from(form.output.trim());
                let spec = Spec::Compress { archive, inputs: form.inputs.clone(), options: form.options() };
                // Passwords don't linger in the form once used.
                form.password.clear();
                form.confirm.clear();
                self.screen = Screen::Activity;
                return self.start(spec);
            }
        }
        Task::none()
    }
}

pub fn view(app: &App) -> El<'_> {
    let form = &app.compress;
    let header = ui::page_header(
        "Compress",
        "Create a new archive from files and folders.",
        vec![ui::ghost(Some(Icon::RotateCcw), "Reset", Some(Message::Compress(Msg::Reset))).into()],
    );

    let problem = form.problem();
    let create = ui::primary(
        Some(Icon::PackagePlus),
        "Create archive",
        problem.is_none().then_some(Message::Compress(Msg::Create)),
    );
    let footer: El<'_> = match &problem {
        Some(p) if !form.inputs.is_empty() => {
            row![ui::banner(Tone::Warning, Icon::CircleAlert, text(p.clone()).into(), vec![]), create]
                .spacing(12)
                .align_y(Alignment::Center)
                .into()
        }
        _ => row![Space::new().width(Length::Fill), create].into(),
    };

    let body = column![inputs(form), output(form), options(form), security(form), footer].spacing(ui::L);
    ui::page(header, body.into())
}

fn inputs(form: &Form) -> El<'_> {
    let list: El<'_> = if form.inputs.is_empty() {
        container(
            column![
                ui::icon_tile(Icon::Files, Tone::Accent, 44.0),
                ui::strong("Drop files and folders here"),
                ui::small("or add them with the buttons below."),
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .padding(24)
        .center_x(Length::Fill)
        .style(theme::drop_zone(false))
        .into()
    } else {
        let rows = form.inputs.iter().enumerate().map(|(i, path)| {
            let is_dir = path.is_dir();
            row![
                icon(if is_dir { Icon::Folder } else { Icon::File }, 17.0),
                column![
                    text(
                        path.file_name()
                            .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned())
                    ),
                    ui::small(path.parent().map(|p| p.display().to_string()).unwrap_or_default()),
                ]
                .width(Length::Fill),
                ui::icon_button(Icon::X, "Remove", Some(Message::Compress(Msg::Remove(i)))),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        });
        scrollable(column(rows).spacing(6)).style(theme::scroll).height(Length::Shrink).into()
    };
    let actions = row![
        ui::secondary(Some(Icon::FilePlus), "Add files…", Some(Message::Compress(Msg::AddFilesDialog))),
        ui::secondary(Some(Icon::FolderPlus), "Add folder…", Some(Message::Compress(Msg::AddFolderDialog))),
        Space::new().width(Length::Fill),
        ui::small(if form.inputs.is_empty() { String::new() } else { plural(form.inputs.len(), "item") }),
        ui::ghost(None, "Clear", (!form.inputs.is_empty()).then_some(Message::Compress(Msg::ClearInputs))),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    ui::section("What to compress", None, column![list, actions].spacing(14).into())
}

fn output(form: &Form) -> El<'_> {
    let formats = row(ArchiveFormat::ALL.iter().map(|f| {
        button(text(f.to_string()).size(13))
            .padding([7, 14])
            .style(theme::chip(form.format == *f))
            .on_press(Message::Compress(Msg::Format(*f)))
            .into()
    }))
    .spacing(6);
    let hint = match form.format {
        ArchiveFormat::SevenZ => "The best compression, solid archives and encrypted file names.",
        ArchiveFormat::Zip => "Opens everywhere, including Windows and macOS without extra software.",
        ArchiveFormat::Tar => "Bundles files with Unix permissions, without compressing them.",
        ArchiveFormat::Gzip | ArchiveFormat::Bzip2 | ArchiveFormat::Xz => "Compresses a single file.",
        ArchiveFormat::Wim => "Windows imaging format.",
    };
    ui::section(
        "Archive",
        None,
        column![
            ui::field("Format", Some(hint), formats.into()),
            ui::field(
                "Save as",
                None,
                ui::path_input(
                    "Where to save the archive",
                    &form.output,
                    |v| Message::Compress(Msg::Output(v)),
                    Message::Compress(Msg::BrowseOutput)
                ),
            ),
        ]
        .spacing(16)
        .into(),
    )
}

fn options(form: &Form) -> El<'_> {
    let format = form.format;
    let mut col = column![].spacing(16);
    if format.supports_compression() {
        col = col.push(ui::field(
            "Compression level",
            Some("Higher levels make smaller archives, more slowly."),
            ui::select(&CompressionLevel::ALL, Some(form.level), |l| Message::Compress(Msg::Level(l))),
        ));
        let methods = method_choices(format);
        let selected = methods.iter().find(|c| c.value == form.method).cloned();
        col =
            col.push(ui::field("Method", None, owned_select(methods, selected, |c| Message::Compress(Msg::Method(c)))));
        if matches!(format, ArchiveFormat::SevenZ | ArchiveFormat::Xz) {
            let dictionaries = dictionary_choices();
            let selected = dictionaries.iter().find(|c| c.value == form.dictionary).cloned();
            col = col.push(ui::field(
                "Dictionary size",
                Some("Larger dictionaries find more repetition and use more memory."),
                owned_select(dictionaries, selected, |c| Message::Compress(Msg::Dictionary(c))),
            ));
        }
    }
    if format.supports_multithreading() {
        let threads = thread_choices();
        let selected = threads.iter().find(|c| c.value == form.threads).cloned();
        col = col.push(ui::field(
            "Threads",
            None,
            owned_select(threads, selected, |c| Message::Compress(Msg::Threads(c))),
        ));
    }
    if format.supports_solid() {
        col = col.push(ui::switch(
            "Solid archive",
            Some("Compresses files together: smaller, but slower to update."),
            form.solid,
            |v| Message::Compress(Msg::Solid(v)),
        ));
    }
    let splits = split_choices();
    let current = if form.custom_split_on { Some(u64::MAX) } else { form.split };
    let selected = splits.iter().find(|c| c.value == current).cloned();
    let mut split_control = column![owned_select(splits, selected, |c| Message::Compress(Msg::Split(c)))].spacing(8);
    if form.custom_split_on {
        split_control = split_control
            .push(ui::input("e.g. 250m or 2g", &form.custom_split, |v| Message::Compress(Msg::CustomSplit(v))));
    }
    col = col.push(ui::field("Split into volumes", Some("For size-limited media and uploads."), split_control.into()));
    col = col.push(ui::field(
        "Leave out",
        Some("Wildcards, separated by commas: *.tmp, node_modules"),
        ui::input("Nothing", &form.excludes, |v| Message::Compress(Msg::Excludes(v))).into(),
    ));
    col = col.push(ui::switch(
        "Delete the originals afterwards",
        Some("Only once the archive was written without errors."),
        form.delete_after,
        |v| Message::Compress(Msg::DeleteAfter(v)),
    ));
    ui::section("Options", None, col.into())
}

fn security(form: &Form) -> El<'_> {
    let format = form.format;
    if !format.supports_encryption() {
        return ui::section(
            "Encryption",
            None,
            ui::muted(format!(
                "{format} archives can't be encrypted. Choose 7z or ZIP to protect them with a password."
            ))
            .into(),
        );
    }
    let mut col = column![ui::field(
        "Password",
        Some("Leave empty for no encryption."),
        column![
            ui::input("Password", &form.password, |v| Message::Compress(Msg::Password(v))).secure(!form.show_password),
            ui::input("Repeat the password", &form.confirm, |v| Message::Compress(Msg::Confirm(v)))
                .secure(!form.show_password),
            checkbox(form.show_password)
                .label("Show the password")
                .on_toggle(|v| Message::Compress(Msg::ShowPassword(v)))
                .size(16),
        ]
        .spacing(8)
        .into(),
    ),]
    .spacing(16);
    if format.supports_header_encryption() {
        col = col.push(ui::switch(
            "Encrypt file names",
            Some("Without the password, nobody can even list the files."),
            form.encrypt_names,
            |v| Message::Compress(Msg::EncryptNames(v)),
        ));
    }
    if format == ArchiveFormat::Zip {
        col = col.push(ui::switch(
            "Use AES-256",
            Some("Much stronger than ZipCrypto, but Windows Explorer can't open it."),
            form.zip_aes,
            |v| Message::Compress(Msg::ZipAes(v)),
        ));
    }
    let strength = password_strength(&form.password);
    if !form.password.is_empty() {
        col = col.push(
            row![ui::small("Strength:"), ui::badge(strength.0, strength.1)].spacing(8).align_y(Alignment::Center),
        );
    }
    ui::section("Encryption", Some("AES-256. A lost password can't be recovered by anyone."), col.into())
}

/// A rough, honest strength estimate based on length and variety.
pub fn password_strength(password: &str) -> (&'static str, Tone) {
    let classes = [
        password.chars().any(char::is_lowercase),
        password.chars().any(char::is_uppercase),
        password.chars().any(|c| c.is_ascii_digit()),
        password.chars().any(|c| !c.is_alphanumeric()),
    ]
    .iter()
    .filter(|b| **b)
    .count();
    let length = password.chars().count();
    match (length, classes) {
        (0..8, _) => ("Weak", Tone::Danger),
        (8..12, 0..=2) => ("Fair", Tone::Warning),
        (8..16, _) | (16.., 0..=1) => ("Good", Tone::Accent),
        _ => ("Strong", Tone::Success),
    }
}

/// A pick list over choices built for this frame.
fn owned_select<'a, T>(
    choices: Vec<Choice<T>>,
    selected: Option<Choice<T>>,
    on_select: impl Fn(Choice<T>) -> Message + 'a,
) -> El<'a>
where
    T: Clone + PartialEq + 'a,
{
    iced::widget::pick_list(choices, selected, on_select)
        .padding([8, 12])
        .width(Length::Fill)
        .style(theme::select)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_an_output_next_to_the_input() {
        let mut form = Form::from_defaults(&CompressDefaults::default());
        form.add_inputs(vec![PathBuf::from("/home/me/report.pdf")]);
        assert_eq!(PathBuf::from(&form.output), Path::new("/home/me").join("report.7z"));
        form.update_format_for_test(ArchiveFormat::Zip);
        assert_eq!(PathBuf::from(&form.output), Path::new("/home/me").join("report.zip"));
    }

    #[test]
    fn several_inputs_are_named_after_their_folder() {
        let mut form = Form::from_defaults(&CompressDefaults::default());
        form.add_inputs(vec![PathBuf::from("/home/me/a.txt"), PathBuf::from("/home/me/b.txt")]);
        assert_eq!(PathBuf::from(&form.output), Path::new("/home/me").join("me.7z"));
    }

    #[test]
    fn problems_block_submission() {
        let mut form = Form::from_defaults(&CompressDefaults::default());
        assert!(form.problem().is_some());
        form.add_inputs(vec![PathBuf::from("/tmp/x.txt")]);
        assert!(form.problem().is_none());
        form.password = "a".into();
        assert_eq!(form.problem().as_deref(), Some("The passwords don't match."));
        form.confirm = "a".into();
        assert!(form.problem().is_none());
    }

    #[test]
    fn strength() {
        assert_eq!(password_strength("abc").0, "Weak");
        assert_eq!(password_strength("abcdefghij").0, "Fair");
        assert_eq!(password_strength("Abcdefgh1!xy").0, "Good");
        assert_eq!(password_strength("Correct-Horse-Battery-9").0, "Strong");
    }

    impl Form {
        fn update_format_for_test(&mut self, format: ArchiveFormat) {
            self.format = format;
            self.retarget_extension();
            self.suggest_output();
        }
    }
}
