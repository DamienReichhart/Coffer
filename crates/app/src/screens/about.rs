//! Version, licences, links and the independence statement.

use iced::widget::{column, container, row, svg, text};
use iced::{Alignment, Length};

use crate::app::{App, Message};
use crate::fonts;
use crate::icons::Icon;
use crate::ui::{self, El};

const LOGO: &[u8] = include_bytes!("../../../../assets/brand/icon.svg");

pub fn view(app: &App) -> El<'_> {
    let seven = app.seven().map_or_else(|| "not found".to_owned(), |s| format!("{} ({})", s.version, s.path.display()));
    let target = coffer_updater::Target::current()
        .map_or_else(|| std::env::consts::OS.to_owned(), |t| format!("{}-{}", t.os, t.arch));
    let kind = coffer_updater::InstallKind::detect().map_or_else(|| "development build".to_owned(), |k| k.to_string());

    let hero = row![
        svg(svg::Handle::from_memory(LOGO)).width(56).height(56),
        column![
            text(crate::APP_NAME).font(fonts::SEMIBOLD).size(22),
            ui::muted("A native desktop interface for 7-Zip.").size(13),
            row![
                ui::badge(format!("Version {}", crate::VERSION), crate::theme::Tone::Accent),
                ui::badge(target, crate::theme::Tone::Neutral),
                ui::badge(kind, crate::theme::Tone::Neutral)
            ]
            .spacing(6),
        ]
        .spacing(8),
    ]
    .spacing(16)
    .align_y(Alignment::Center);

    let links = row![
        ui::secondary(Some(Icon::ExternalLink), "Website", Some(Message::OpenUrl(crate::WEBSITE.into()))),
        ui::secondary(Some(Icon::BookOpen), "Documentation", Some(Message::OpenUrl(crate::DOCS.into()))),
        ui::secondary(Some(Icon::Sparkles), "What's new", Some(Message::OpenUrl(crate::CHANGELOG.into()))),
        ui::secondary(Some(Icon::Github), "Source code", Some(Message::OpenUrl(crate::REPOSITORY.into()))),
        ui::secondary(Some(Icon::LifeBuoy), "Report a problem", Some(Message::OpenUrl(crate::ISSUES.into()))),
    ]
    .spacing(8)
    .wrap();

    let details = ui::section(
        "This installation",
        None,
        ui::properties(vec![
            ("Coffer", crate::VERSION.to_owned()),
            ("7-Zip", seven),
            ("Settings", crate::settings::paths::settings_file().map(|p| p.display().to_string()).unwrap_or_default()),
        ]),
    );

    let legal = ui::section(
        "Independent and open source",
        None,
        column![
            ui::muted("Coffer is free software under the MIT licence. It is an independent project: it is not affiliated with, endorsed by or sponsored by 7-Zip or Igor Pavlov. 7-Zip is not included — it runs the copy you installed, under 7-Zip's own licence."),
            ui::muted("Built with iced (MIT), the Inter typeface (SIL Open Font License 1.1) and Lucide icons (ISC)."),
            ui::muted("No telemetry: the only request Coffer makes on its own is the update check, which you can turn off."),
        ]
        .spacing(10)
        .into(),
    );

    ui::page(container(hero).width(Length::Fill).into(), column![links, details, legal].spacing(ui::XL).into())
}
