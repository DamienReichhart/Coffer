//! The Lucide icons the interface uses (ISC licence, `assets/icons/LICENSE`).

use iced::widget::svg::{self, Svg};
use iced::{Color, Theme};

/// An icon from `assets/icons`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Activity,
    Archive,
    ArrowDownToLine,
    ArrowLeft,
    ArrowUp,
    BookOpen,
    Box,
    Check,
    ChevronDown,
    ChevronRight,
    CircleAlert,
    CircleCheck,
    CircleX,
    Clock,
    Copy,
    Download,
    ExternalLink,
    Eye,
    EyeOff,
    File,
    FileArchive,
    FilePlus,
    Files,
    Folder,
    FolderOpen,
    FolderOutput,
    FolderPlus,
    GitBranch,
    Github,
    Hash,
    Heart,
    House,
    Info,
    Layers,
    LifeBuoy,
    ListFilter,
    LoaderCircle,
    Lock,
    Minus,
    Monitor,
    Moon,
    PackageOpen,
    PackagePlus,
    Pause,
    Pencil,
    Play,
    Plus,
    RefreshCw,
    RotateCcw,
    Search,
    Settings,
    ShieldCheck,
    Sparkles,
    Square,
    SquareCheck,
    Sun,
    Terminal,
    Trash2,
    TriangleAlert,
    Upload,
    X,
}

impl Icon {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Activity => include_bytes!("../assets/icons/activity.svg"),
            Self::Archive => include_bytes!("../assets/icons/archive.svg"),
            Self::ArrowDownToLine => include_bytes!("../assets/icons/arrow-down-to-line.svg"),
            Self::ArrowLeft => include_bytes!("../assets/icons/arrow-left.svg"),
            Self::ArrowUp => include_bytes!("../assets/icons/arrow-up.svg"),
            Self::BookOpen => include_bytes!("../assets/icons/book-open.svg"),
            Self::Box => include_bytes!("../assets/icons/box.svg"),
            Self::Check => include_bytes!("../assets/icons/check.svg"),
            Self::ChevronDown => include_bytes!("../assets/icons/chevron-down.svg"),
            Self::ChevronRight => include_bytes!("../assets/icons/chevron-right.svg"),
            Self::CircleAlert => include_bytes!("../assets/icons/circle-alert.svg"),
            Self::CircleCheck => include_bytes!("../assets/icons/circle-check.svg"),
            Self::CircleX => include_bytes!("../assets/icons/circle-x.svg"),
            Self::Clock => include_bytes!("../assets/icons/clock.svg"),
            Self::Copy => include_bytes!("../assets/icons/copy.svg"),
            Self::Download => include_bytes!("../assets/icons/download.svg"),
            Self::ExternalLink => include_bytes!("../assets/icons/external-link.svg"),
            Self::Eye => include_bytes!("../assets/icons/eye.svg"),
            Self::EyeOff => include_bytes!("../assets/icons/eye-off.svg"),
            Self::File => include_bytes!("../assets/icons/file.svg"),
            Self::FileArchive => include_bytes!("../assets/icons/file-archive.svg"),
            Self::FilePlus => include_bytes!("../assets/icons/file-plus.svg"),
            Self::Files => include_bytes!("../assets/icons/files.svg"),
            Self::Folder => include_bytes!("../assets/icons/folder.svg"),
            Self::FolderOpen => include_bytes!("../assets/icons/folder-open.svg"),
            Self::FolderOutput => include_bytes!("../assets/icons/folder-output.svg"),
            Self::FolderPlus => include_bytes!("../assets/icons/folder-plus.svg"),
            Self::GitBranch => include_bytes!("../assets/icons/git-branch.svg"),
            Self::Github => include_bytes!("../assets/icons/github.svg"),
            Self::Hash => include_bytes!("../assets/icons/hash.svg"),
            Self::Heart => include_bytes!("../assets/icons/heart.svg"),
            Self::House => include_bytes!("../assets/icons/house.svg"),
            Self::Info => include_bytes!("../assets/icons/info.svg"),
            Self::Layers => include_bytes!("../assets/icons/layers.svg"),
            Self::LifeBuoy => include_bytes!("../assets/icons/life-buoy.svg"),
            Self::ListFilter => include_bytes!("../assets/icons/list-filter.svg"),
            Self::LoaderCircle => include_bytes!("../assets/icons/loader-circle.svg"),
            Self::Lock => include_bytes!("../assets/icons/lock.svg"),
            Self::Minus => include_bytes!("../assets/icons/minus.svg"),
            Self::Monitor => include_bytes!("../assets/icons/monitor.svg"),
            Self::Moon => include_bytes!("../assets/icons/moon.svg"),
            Self::PackageOpen => include_bytes!("../assets/icons/package-open.svg"),
            Self::PackagePlus => include_bytes!("../assets/icons/package-plus.svg"),
            Self::Pause => include_bytes!("../assets/icons/pause.svg"),
            Self::Pencil => include_bytes!("../assets/icons/pencil.svg"),
            Self::Play => include_bytes!("../assets/icons/play.svg"),
            Self::Plus => include_bytes!("../assets/icons/plus.svg"),
            Self::RefreshCw => include_bytes!("../assets/icons/refresh-cw.svg"),
            Self::RotateCcw => include_bytes!("../assets/icons/rotate-ccw.svg"),
            Self::Search => include_bytes!("../assets/icons/search.svg"),
            Self::Settings => include_bytes!("../assets/icons/settings.svg"),
            Self::ShieldCheck => include_bytes!("../assets/icons/shield-check.svg"),
            Self::Sparkles => include_bytes!("../assets/icons/sparkles.svg"),
            Self::Square => include_bytes!("../assets/icons/square.svg"),
            Self::SquareCheck => include_bytes!("../assets/icons/square-check.svg"),
            Self::Sun => include_bytes!("../assets/icons/sun.svg"),
            Self::Terminal => include_bytes!("../assets/icons/terminal.svg"),
            Self::Trash2 => include_bytes!("../assets/icons/trash-2.svg"),
            Self::TriangleAlert => include_bytes!("../assets/icons/triangle-alert.svg"),
            Self::Upload => include_bytes!("../assets/icons/upload.svg"),
            Self::X => include_bytes!("../assets/icons/x.svg"),
        }
    }

    pub fn handle(self) -> svg::Handle {
        svg::Handle::from_memory(self.bytes())
    }
}

/// An icon drawn in the theme's muted text colour.
pub fn icon<'a>(icon: Icon, size: f32) -> Svg<'a, Theme> {
    Svg::new(icon.handle())
        .width(size)
        .height(size)
        .style(|theme: &Theme, _| svg::Style { color: Some(crate::theme::tokens(theme).muted) })
}

/// An icon drawn in a fixed colour picked from the theme.
pub fn tinted<'a>(icon: Icon, size: f32, pick: fn(&crate::theme::Tokens) -> Color) -> Svg<'a, Theme> {
    Svg::new(icon.handle())
        .width(size)
        .height(size)
        .style(move |theme: &Theme, _| svg::Style { color: Some(pick(crate::theme::tokens(theme))) })
}
