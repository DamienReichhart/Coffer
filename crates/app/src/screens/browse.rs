//! Browsing an open archive: folders, selection, sorting, search, and the
//! actions on the archive (extract, add, delete, rename, test, open a file).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use coffer_core::format::{bytes, ratio};
use coffer_core::listing::{Node, Tree, natural_cmp, parent};
use coffer_core::{ArchiveFormat, Listing};
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::widget::{Space, button, column, container, mouse_area, row, rule, scrollable, text, text_input};
use iced::{Alignment, Length, Padding, Task};

use crate::app::{App, Message, Screen};
use crate::fonts;
use crate::icons::{Icon, icon};
use crate::jobs::{Spec, plural};
use crate::screens::modal::Modal;
use crate::theme::{self, Tone};
use crate::ui::{self, El};

/// Rows drawn at once in one folder. Bigger folders ask for a search.
const MAX_ROWS: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Name,
    Size,
    Packed,
    Modified,
    Method,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Enter(String),
    Up,
    Select(String),
    Activate(String),
    Search(String),
    Sort(Column),
    SelectAll,
    ClearSelection,
    ExtractAll,
    ExtractSelected,
    AddDialog,
    AddFiles(Vec<PathBuf>),
    Delete,
    DeleteConfirmed(Vec<String>),
    Rename,
    Test,
    ToggleInfo,
    Reload,
    Key(Key, Modifiers),
}

/// An open archive.
#[derive(Debug)]
pub struct Browser {
    pub path: PathBuf,
    pub listing: Listing,
    pub tree: Tree,
    pub cwd: String,
    pub selection: BTreeSet<String>,
    anchor: Option<String>,
    pub search: String,
    pub sort: (Column, bool),
    pub password: Option<String>,
    pub show_info: bool,
    /// The volumes, when the archive is split into several files.
    pub volumes: Option<coffer_core::volumes::VolumeSet>,
}

impl Browser {
    pub fn new(path: PathBuf, listing: Listing, password: Option<String>) -> Self {
        let tree = Tree::new(&listing.entries);
        // An archive with a single top folder opens inside it.
        let roots = tree.children("");
        let cwd = if roots.len() == 1 && roots[0].is_dir { roots[0].path.clone() } else { String::new() };
        let volumes = coffer_core::volumes::detect(&path);
        Self {
            volumes,
            path,
            listing,
            tree,
            cwd,
            selection: BTreeSet::new(),
            anchor: None,
            search: String::new(),
            sort: (Column::Name, true),
            password,
            show_info: true,
        }
    }

    /// The archive's name; a split archive is named without its volume number.
    pub fn name(&self) -> &str {
        match &self.volumes {
            Some(set) => &set.name,
            None => self.path.file_name().and_then(|n| n.to_str()).unwrap_or("Archive"),
        }
    }

    /// The items to show: the current folder, or search results anywhere.
    pub fn items(&self) -> Vec<&Node> {
        let mut items = if self.search.trim().is_empty() {
            self.tree.children(&self.cwd)
        } else {
            self.tree.search(self.search.trim())
        };
        let (column, ascending) = self.sort;
        if column != Column::Name || !ascending {
            items.sort_by(|a, b| {
                let order = match column {
                    Column::Name => natural_cmp(&a.name, &b.name),
                    Column::Size => a.size.cmp(&b.size),
                    Column::Packed => a.packed.cmp(&b.packed),
                    Column::Modified => a.modified.cmp(&b.modified),
                    Column::Method => a.method.cmp(&b.method),
                };
                let order = if ascending { order } else { order.reverse() };
                b.is_dir.cmp(&a.is_dir).then(order).then_with(|| natural_cmp(&a.name, &b.name))
            });
        }
        items
    }

    fn enter(&mut self, dir: String) {
        self.cwd = dir;
        self.search.clear();
        self.selection.clear();
        self.anchor = None;
    }

    fn select(&mut self, path: String, modifiers: Modifiers) {
        if modifiers.shift()
            && let Some(anchor) = self.anchor.clone()
        {
            let items: Vec<String> = self.items().iter().map(|n| n.path.clone()).collect();
            let a = items.iter().position(|p| *p == anchor);
            let b = items.iter().position(|p| *p == path);
            if let (Some(a), Some(b)) = (a, b) {
                let (from, to) = if a <= b { (a, b) } else { (b, a) };
                if !modifiers.command() {
                    self.selection.clear();
                }
                self.selection.extend(items[from..=to].iter().cloned());
                return;
            }
        }
        if modifiers.command() {
            if !self.selection.remove(&path) {
                self.selection.insert(path.clone());
            }
        } else {
            self.selection.clear();
            self.selection.insert(path.clone());
        }
        self.anchor = Some(path);
    }

    fn selection_size(&self) -> u64 {
        self.selection.iter().filter_map(|p| self.tree.get(p)).map(|n| n.size).sum()
    }

    /// The archive format, for adding files with the right `-t`.
    fn writable_format(&self) -> Option<ArchiveFormat> {
        match self.listing.info.kind.to_ascii_lowercase().as_str() {
            "7z" => Some(ArchiveFormat::SevenZ),
            "zip" => Some(ArchiveFormat::Zip),
            "tar" => Some(ArchiveFormat::Tar),
            "wim" => Some(ArchiveFormat::Wim),
            _ => None,
        }
    }
}

fn preview_dir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
    std::env::temp_dir().join(format!("coffer-{}-{n}", std::process::id()))
}

impl App {
    pub(crate) fn update_browse(&mut self, msg: Msg) -> Task<Message> {
        let modifiers = self.modifiers;
        let confirm_delete = self.settings.confirm_delete;
        let Some(browser) = self.browser.as_mut() else { return Task::none() };
        match msg {
            Msg::Enter(dir) => browser.enter(dir),
            Msg::Up => {
                if !browser.search.is_empty() {
                    browser.search.clear();
                } else if !browser.cwd.is_empty() {
                    let up = parent(&browser.cwd).unwrap_or("").to_owned();
                    browser.enter(up);
                }
            }
            Msg::Select(path) => browser.select(path, modifiers),
            Msg::Activate(path) => {
                if browser.tree.is_dir(&path) {
                    browser.enter(path);
                } else {
                    let spec = Spec::Preview {
                        archive: browser.path.clone(),
                        entry: path,
                        password: browser.password.clone(),
                        dir: preview_dir(),
                    };
                    return self.start(spec);
                }
            }
            Msg::Search(query) => {
                browser.search = query;
                browser.selection.clear();
            }
            Msg::Sort(column) => {
                browser.sort = if browser.sort.0 == column { (column, !browser.sort.1) } else { (column, true) };
            }
            Msg::SelectAll => {
                let all: Vec<String> = browser.items().iter().map(|n| n.path.clone()).collect();
                browser.selection.extend(all);
            }
            Msg::ClearSelection => browser.selection.clear(),
            Msg::ExtractAll | Msg::ExtractSelected => {
                let only: Vec<String> = if matches!(msg, Msg::ExtractSelected) {
                    browser.selection.iter().cloned().collect()
                } else {
                    Vec::new()
                };
                let archive = browser.path.clone();
                let password = browser.password.clone();
                self.extract.set_archive(archive, &self.settings.extract);
                self.extract.only = only;
                if let Some(password) = password {
                    self.extract.password = password;
                }
                self.screen = Screen::Extract;
            }
            Msg::AddDialog => {
                return Task::perform(
                    rfd::AsyncFileDialog::new().set_title("Add files to the archive").pick_files(),
                    |files| {
                        let paths: Vec<PathBuf> =
                            files.unwrap_or_default().iter().map(|f| f.path().to_path_buf()).collect();
                        if paths.is_empty() { Message::Nothing } else { Message::Browse(Msg::AddFiles(paths)) }
                    },
                );
            }
            Msg::AddFiles(inputs) => {
                let Some(format) = browser.writable_format() else {
                    let kind = browser.listing.info.kind.clone();
                    self.toast(Tone::Warning, format!("7-Zip can't add files to {kind} archives."), None);
                    return Task::none();
                };
                if browser.volumes.is_some() || browser.listing.info.volumes.is_some_and(|v| v > 1) {
                    self.toast(Tone::Warning, "Split archives can't be changed.", None);
                    return Task::none();
                }
                let options = coffer_core::CompressOptions {
                    format,
                    level: self.settings.compress.level,
                    solid: self.settings.compress.solid,
                    password: browser.password.clone(),
                    encrypt_names: browser.password.is_some() && self.settings.compress.encrypt_names,
                    zip_aes: self.settings.compress.zip_aes,
                    ..Default::default()
                };
                let spec = Spec::Compress { archive: browser.path.clone(), inputs, options };
                return self.start(spec);
            }
            Msg::Delete => {
                let entries: Vec<String> = browser.selection.iter().cloned().collect();
                if entries.is_empty() {
                    return Task::none();
                }
                if confirm_delete {
                    self.modal = Some(Modal::confirm_delete(entries));
                    return Task::none();
                }
                return self.update_browse(Msg::DeleteConfirmed(entries));
            }
            Msg::DeleteConfirmed(entries) => {
                let spec = Spec::Delete { archive: browser.path.clone(), entries, password: browser.password.clone() };
                browser.selection.clear();
                return self.start(spec);
            }
            Msg::Rename => {
                if browser.selection.len() == 1
                    && let Some(path) = browser.selection.iter().next().cloned()
                {
                    self.modal = Some(Modal::rename(path));
                    return crate::screens::modal::focus();
                }
            }
            Msg::Test => {
                let spec = Spec::Test { archive: browser.path.clone(), password: browser.password.clone() };
                return self.start(spec);
            }
            Msg::ToggleInfo => browser.show_info = !browser.show_info,
            Msg::Reload => {
                let (path, password) = (browser.path.clone(), browser.password.clone());
                return self.open(path, password);
            }
            Msg::Key(key, modifiers) => {
                let command = modifiers.command();
                return match key.as_ref() {
                    Key::Named(Named::Backspace) => self.update_browse(Msg::Up),
                    Key::Named(Named::Delete) => self.update_browse(Msg::Delete),
                    Key::Named(Named::F2) => self.update_browse(Msg::Rename),
                    Key::Named(Named::F5) => self.update_browse(Msg::Reload),
                    Key::Named(Named::Enter) if browser.selection.len() == 1 => {
                        let path = browser.selection.iter().next().cloned().unwrap_or_default();
                        self.update_browse(Msg::Activate(path))
                    }
                    Key::Character("a") if command => self.update_browse(Msg::SelectAll),
                    Key::Character("f") if command => iced::widget::operation::focus(SEARCH_ID),
                    Key::Character("t") if command => self.update_browse(Msg::Test),
                    _ => Task::none(),
                };
            }
        }
        Task::none()
    }
}

const SEARCH_ID: &str = "archive-search";

pub fn view(app: &App) -> El<'_> {
    let Some(browser) = &app.browser else {
        let body: El<'_> = if app.opening.is_some() {
            ui::empty_state(
                Icon::LoaderCircle,
                Tone::Accent,
                "Opening the archive…",
                "7-Zip is reading its contents.",
                vec![],
            )
        } else {
            ui::empty_state(
                Icon::FolderOpen,
                Tone::Neutral,
                "No archive is open",
                "Open one, or drop it on this window.",
                vec![ui::primary(Some(Icon::FolderOpen), "Open an archive…", Some(Message::OpenDialog)).into()],
            )
        };
        return body;
    };

    iced::widget::responsive(move |size| layout(browser, size.width)).into()
}

/// The width of the details panel.
const PANEL_W: f32 = 272.0;

/// The browser at a given width: narrow windows drop the details panel, then
/// the secondary columns and the toolbar labels, so names always have room.
fn layout(browser: &Browser, width: f32) -> El<'_> {
    let panel = browser.show_info && width >= 900.0;
    let compact = width - if panel { PANEL_W } else { 0.0 } < 720.0;
    let content = column![
        toolbar(browser, compact),
        rule::horizontal(1).style(theme::hairline),
        table(browser, compact),
        rule::horizontal(1).style(theme::hairline),
        status_bar(browser),
    ];
    // The table is always the first child, so it keeps its scroll position
    // and the search its focus when the panel comes and goes.
    let main = container(content).width(Length::Fill).height(Length::Fill);
    let mut layout = row![main];
    if panel {
        layout = layout.push(rule::vertical(1).style(theme::hairline)).push(info_panel(browser));
    }
    layout.into()
}

/// A toolbar button: smaller than the form buttons, and only an icon when space is short.
fn tool(i: Icon, label: &str, msg: Option<Message>, compact: bool) -> El<'_> {
    if compact { ui::icon_button(i, label, msg) } else { ui::secondary(Some(i), label, msg).padding([5, 10]).into() }
}

fn toolbar(browser: &Browser, compact: bool) -> El<'_> {
    let mut crumbs = row![].spacing(0).align_y(Alignment::Center);
    crumbs = crumbs.push(crumb(
        browser.name().to_owned(),
        String::new(),
        browser.cwd.is_empty() && browser.search.is_empty(),
    ));
    if !browser.cwd.is_empty() {
        let mut path = String::new();
        let parts: Vec<&str> = browser.cwd.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            if !path.is_empty() {
                path.push('/');
            }
            path.push_str(part);
            crumbs = crumbs.push(icon(Icon::ChevronRight, 13.0));
            crumbs =
                crumbs.push(crumb((*part).to_owned(), path.clone(), i == parts.len() - 1 && browser.search.is_empty()));
        }
    }
    let has_selection = !browser.selection.is_empty();
    let extract: El<'_> = if has_selection {
        ui::primary(
            Some(Icon::FolderOutput),
            format!("Extract {}", plural(browser.selection.len(), "item")),
            Some(Message::Browse(Msg::ExtractSelected)),
        )
        .padding([5, 12])
        .into()
    } else {
        ui::primary(Some(Icon::FolderOutput), "Extract all", Some(Message::Browse(Msg::ExtractAll)))
            .padding([5, 12])
            .into()
    };
    let search = text_input("Search", &browser.search)
        .id(SEARCH_ID)
        .on_input(|q| Message::Browse(Msg::Search(q)))
        .padding(iced::Padding::new(6.0).left(30.0).right(10.0))
        .size(13)
        .width(Length::Fixed(if compact { 170.0 } else { 220.0 }))
        .style(theme::input);
    let search = iced::widget::stack![
        search,
        container(icon(Icon::Search, 14.0)).padding(iced::Padding::new(0.0).left(10.0)).center_y(Length::Fill)
    ];
    let top = row![
        ui::icon_button(
            Icon::ArrowUp,
            "Up one folder (Backspace)",
            (!browser.cwd.is_empty() || !browser.search.is_empty()).then_some(Message::Browse(Msg::Up))
        ),
        container(
            scrollable(crumbs)
                .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new().width(0).scroller_width(0)))
        )
        .width(Length::Fill),
        search,
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let actions = row![
        extract,
        tool(Icon::FilePlus, "Add", Some(Message::Browse(Msg::AddDialog)), compact),
        tool(Icon::ShieldCheck, "Test", Some(Message::Browse(Msg::Test)), compact),
        tool(Icon::Pencil, "Rename", (browser.selection.len() == 1).then_some(Message::Browse(Msg::Rename)), compact),
        tool(Icon::Trash2, "Delete", has_selection.then_some(Message::Browse(Msg::Delete)), compact),
        Space::new().width(Length::Fill),
        ui::icon_button(Icon::RefreshCw, "Reload (F5)", Some(Message::Browse(Msg::Reload))),
        ui::icon_button(
            Icon::Info,
            if browser.show_info { "Hide the details" } else { "Show the details" },
            Some(Message::Browse(Msg::ToggleInfo))
        ),
        ui::icon_button(Icon::X, "Close the archive", Some(Message::CloseArchive)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    container(column![top, actions].spacing(10)).padding([10, 14]).width(Length::Fill).style(theme::toolbar).into()
}

fn crumb<'a>(label: String, path: String, current: bool) -> El<'a> {
    let label = text(label).size(13).font(if current { fonts::SEMIBOLD } else { fonts::REGULAR });
    let label = if current { label } else { label.style(theme::muted) };
    button(label).padding([4, 6]).style(theme::ghost).on_press(Message::Browse(Msg::Enter(path))).into()
}

const SIZE_W: f32 = 92.0;
const PACKED_W: f32 = 92.0;
const DATE_W: f32 = 140.0;
const METHOD_W: f32 = 110.0;

fn header_cell<'a>(browser: &Browser, column: Column, label: &'a str, width: Length, right: bool) -> El<'a> {
    let (sorted, ascending) = browser.sort;
    let active = sorted == column;
    let arrow = if active { if ascending { " ↑" } else { " ↓" } } else { "" };
    let label = text(format!("{label}{arrow}")).size(12).font(fonts::MEDIUM);
    let label = if active { label } else { label.style(theme::muted) };
    let label = if right { container(label).align_right(Length::Fill) } else { container(label).width(Length::Fill) };
    button(label).padding([5, 6]).width(width).style(theme::ghost).on_press(Message::Browse(Msg::Sort(column))).into()
}

fn table(browser: &Browser, compact: bool) -> El<'_> {
    let mut cells = row![
        Space::new().width(24),
        header_cell(browser, Column::Name, "Name", Length::Fill, false),
        header_cell(browser, Column::Size, "Size", Length::Fixed(SIZE_W), true),
    ]
    .spacing(4)
    .align_y(Alignment::Center);
    if !compact {
        cells = cells.push(header_cell(browser, Column::Packed, "Packed", Length::Fixed(PACKED_W), true));
    }
    cells = cells.push(header_cell(browser, Column::Modified, "Modified", Length::Fixed(DATE_W), false));
    if !compact {
        cells = cells.push(header_cell(browser, Column::Method, "Method", Length::Fixed(METHOD_W), false));
    }
    let header = container(cells).padding([3, 14]).style(theme::table_header);

    let items = browser.items();
    if items.is_empty() {
        let message = if browser.search.is_empty() { "This folder is empty." } else { "Nothing matches your search." };
        return column![
            header,
            rule::horizontal(1).style(theme::hairline),
            ui::empty_state(Icon::Search, Tone::Neutral, message, "", vec![])
        ]
        .height(Length::Fill)
        .into();
    }
    let total = items.len();
    let searching = !browser.search.trim().is_empty();
    let rows = items.into_iter().take(MAX_ROWS).map(|node| entry_row(browser, node, searching, compact));
    let mut list = column(rows).spacing(0).padding(Padding::new(4.0).left(8.0).right(14.0));
    if total > MAX_ROWS {
        list = list.push(
            container(ui::small(format!("Showing {MAX_ROWS} of {total} items. Search to find the others.")))
                .padding(12)
                .center_x(Length::Fill),
        );
    }
    column![
        header,
        rule::horizontal(1).style(theme::hairline),
        scrollable(list).style(theme::scroll).height(Length::Fill)
    ]
    .height(Length::Fill)
    .into()
}

fn entry_row<'a>(browser: &Browser, node: &'a Node, searching: bool, compact: bool) -> El<'a> {
    let selected = browser.selection.contains(&node.path);
    let file_icon = if node.is_dir {
        Icon::Folder
    } else if node.encrypted {
        Icon::Lock
    } else {
        Icon::File
    };
    let tint: fn(&theme::Tokens) -> iced::Color = if node.is_dir { |t| t.primary } else { |t| t.subtle };
    let name: El<'a> = if searching {
        row![
            text(node.name.as_str()).size(13),
            ui::small(parent(&node.path).unwrap_or("/").to_owned()).wrapping(text::Wrapping::None),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    } else {
        text(node.name.as_str()).size(13).wrapping(text::Wrapping::None).into()
    };
    let size = if node.is_dir { plural(node.files as usize, "file") } else { bytes(node.size) };
    let packed = node.packed.map_or_else(|| "—".to_owned(), bytes);
    let cell = |content: String, width: f32, right: bool| -> El<'a> {
        let t = text(content).size(12).style(theme::muted).wrapping(text::Wrapping::None);
        let c = container(t).width(width).padding([0, 6]).clip(true);
        if right { c.align_right(Length::Fixed(width)).into() } else { c.into() }
    };
    let mut cells = row![
        container(crate::icons::tinted(file_icon, 15.0, tint)).width(24),
        container(name).width(Length::Fill).padding([0, 6]).clip(true),
        cell(size, SIZE_W, true),
    ]
    .spacing(4)
    .align_y(Alignment::Center);
    if !compact {
        cells = cells.push(cell(packed, PACKED_W, true));
    }
    cells = cells.push(cell(node.modified.clone().unwrap_or_default(), DATE_W, false));
    if !compact {
        cells = cells.push(cell(node.method.clone().unwrap_or_default(), METHOD_W, false));
    }
    let row_box = container(cells).padding([5, 6]).width(Length::Fill).style(theme::row(selected));
    mouse_area(row_box)
        .on_press(Message::Browse(Msg::Select(node.path.clone())))
        .on_double_click(Message::Browse(Msg::Activate(node.path.clone())))
        .into()
}

fn status_bar(browser: &Browser) -> El<'_> {
    let listing = &browser.listing;
    let mut left = format!("{} · {}", plural(listing.file_count(), "file"), bytes(listing.total_size()));
    if let Some(size) = listing.archive_size() {
        left = format!("{left} · {} on disk", bytes(size));
    }
    if let Some(set) = &browser.volumes {
        left = format!("{left} · {}", plural(set.volumes.len(), "volume"));
    }
    let right = if browser.selection.is_empty() {
        String::new()
    } else {
        format!("{} selected · {}", plural(browser.selection.len(), "item"), bytes(browser.selection_size()))
    };
    container(row![ui::small(left), Space::new().width(Length::Fill), ui::small(right)].align_y(Alignment::Center))
        .padding([6, 16])
        .width(Length::Fill)
        .style(theme::toolbar)
        .into()
}

fn info_panel(browser: &Browser) -> El<'_> {
    let info = &browser.listing.info;
    let listing = &browser.listing;
    let yes_no = |b: bool| if b { "Yes".to_owned() } else { "No".to_owned() };
    let mut props = vec![
        ("Format", info.kind.clone()),
        ("Files", listing.file_count().to_string()),
        ("Unpacked", bytes(listing.total_size())),
    ];
    if let Some(physical) = listing.archive_size() {
        props.push(("On disk", bytes(physical)));
        if let Some(r) = ratio(listing.total_size(), physical) {
            props.push(("Ratio", r));
        }
    }
    if let Some(method) = &info.method {
        props.push(("Method", method.clone()));
    }
    if let Some(solid) = info.solid {
        props.push(("Solid", yes_no(solid)));
    }
    props.push(("Encrypted", yes_no(info.encrypted)));
    match (&browser.volumes, info.volumes) {
        (Some(set), _) => props.push(("Volumes", format!("{} ({})", set.volumes.len(), set.scheme.label()))),
        (None, Some(volumes)) if volumes > 1 => props.push(("Volumes", volumes.to_string())),
        _ => {}
    }

    let label = |s: &'static str| text(s).size(11).font(fonts::MEDIUM).style(theme::subtle);
    let mut panel = column![
        column![
            text(browser.name()).font(fonts::SEMIBOLD).size(14),
            ui::small(browser.path.parent().map(|p| p.display().to_string()).unwrap_or_default()),
        ]
        .spacing(2),
        label("Archive"),
        ui::properties(props),
    ]
    .spacing(12);

    if let Some(comment) = &info.comment {
        panel = panel.push(column![label("Comment"), text(comment.as_str()).size(13)].spacing(4));
    }
    for warning in &info.warnings {
        panel =
            panel.push(ui::banner(Tone::Warning, Icon::TriangleAlert, text(warning.as_str()).size(12).into(), vec![]));
    }

    if browser.selection.len() == 1
        && let Some(node) = browser.selection.iter().next().and_then(|p| browser.tree.get(p))
    {
        let mut details = vec![("Name", node.name.clone()), ("Size", bytes(node.size))];
        if let Some(packed) = node.packed {
            details.push(("Packed", bytes(packed)));
        }
        if let Some(modified) = &node.modified {
            details.push(("Modified", modified.clone()));
        }
        if let Some(crc) = &node.crc {
            details.push(("CRC32", crc.clone()));
        }
        if let Some(method) = &node.method {
            details.push(("Method", method.clone()));
        }
        if node.is_dir {
            details.push(("Contains", plural(node.files as usize, "file")));
        }
        panel = panel.push(rule::horizontal(1).style(theme::hairline));
        panel = panel.push(label("Selection"));
        panel = panel.push(ui::properties(details));
    } else if browser.selection.len() > 1 {
        panel = panel.push(rule::horizontal(1).style(theme::hairline));
        panel = panel.push(label("Selection"));
        panel = panel.push(ui::properties(vec![
            ("Items", browser.selection.len().to_string()),
            ("Size", bytes(browser.selection_size())),
        ]));
    }

    container(scrollable(container(panel).padding(16)).style(theme::scroll).height(Length::Fill))
        .width(Length::Fixed(PANEL_W))
        .height(Length::Fill)
        .style(theme::side_panel)
        .into()
}

/// The folder an archive extracts into by default: next to it, named after it.
pub fn default_destination(archive: &Path, into_folder: bool) -> PathBuf {
    let dir = archive.parent().map(Path::to_path_buf).unwrap_or_default();
    if !into_folder {
        return dir;
    }
    dir.join(archive_stem(archive))
}

/// `photos.tar.gz` → `photos`, `backup.7z.001` → `backup`, `movie.part2.rar` → `movie`.
pub fn archive_stem(archive: &Path) -> String {
    let mut name = archive.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if let Some(part) = coffer_core::volumes::parse_name(&name) {
        name = part.display_name();
    }
    while let Some((stem, ext)) = name.rsplit_once('.') {
        let ext = ext.to_ascii_lowercase();
        let known = crate::app::ARCHIVE_EXTENSIONS.contains(&ext.as_str()) || ext.chars().all(|c| c.is_ascii_digit());
        if !known || stem.is_empty() {
            break;
        }
        name = stem.to_owned();
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems() {
        assert_eq!(archive_stem(Path::new("/x/photos.tar.gz")), "photos");
        assert_eq!(archive_stem(Path::new("backup.7z.001")), "backup");
        assert_eq!(archive_stem(Path::new("backup.zip.011")), "backup");
        assert_eq!(archive_stem(Path::new("movie.part02.rar")), "movie");
        assert_eq!(archive_stem(Path::new("movie.r03")), "movie");
        assert_eq!(archive_stem(Path::new("site.z01")), "site");
        assert_eq!(archive_stem(Path::new("report.v2.zip")), "report.v2");
        assert_eq!(archive_stem(Path::new(".zip")), ".zip");
        assert_eq!(default_destination(Path::new("/x/a.zip"), true), Path::new("/x/a"));
        assert_eq!(default_destination(Path::new("/x/a.zip"), false), Path::new("/x"));
    }
}
