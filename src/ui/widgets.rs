// Éléments communs aux deux écrans : en-tête, barre de statut, infobulles, états vides
use iced::widget::{
    Text, button, checkbox, column, container, progress_bar, row, space, text, tooltip,
};
use iced::{Center, Element, Fill, Length};
use lucide_icons::iced::{
    icon_download, icon_history, icon_image_down, icon_info, icon_moon, icon_sun, icon_x,
};

use super::theme::{self, Tokens};
use super::{App, Message, Tab, plural};
use crate::update::Version;

pub fn with_tooltip<'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    tokens: Tokens,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(label).size(theme::TEXT_XS))
            .padding([theme::SPACE_XS, theme::SPACE_SM])
            .style(move |_| theme::tooltip(tokens)),
        tooltip::Position::Bottom,
    )
    .gap(theme::SPACE_XS)
    .into()
}

/// Écran ou liste sans contenu : une grande icône estompée et une phrase.
pub fn empty_state<'a>(icon: Text<'a>, label: &'a str, tokens: Tokens) -> Element<'a, Message> {
    container(
        column![
            icon.size(48).color(tokens.border_strong),
            text(label).color(tokens.text_muted),
        ]
        .spacing(theme::SPACE_SM)
        .align_x(Center),
    )
    .center(Fill)
    .into()
}

/// Ligne de séparation horizontale.
pub fn separator<'a>(tokens: Tokens) -> Element<'a, Message> {
    container(space().width(Fill).height(1))
        .style(move |_| theme::separator(tokens))
        .into()
}

pub fn header(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;

    let tab = |label: &'static str, icon: Text<'static>, tab: Tab| {
        let is_active = app.tab == tab;
        button(
            row![
                icon.size(theme::ICON_MD),
                text(label).size(theme::TEXT_SM).font(theme::FONT_MEDIUM)
            ]
            .spacing(theme::SPACE_XS + 2.0)
            .align_y(Center),
        )
        .padding([theme::SPACE_SM, theme::SPACE_MD])
        .on_press(Message::TabSelected(tab))
        .style(move |_, status| theme::tab_button(tokens, is_active, status))
    };

    let dark_mode = app.preferences.dark_mode;
    let dark_mode_toggle = with_tooltip(
        button(if dark_mode { icon_sun() } else { icon_moon() }.size(20))
            .padding(theme::SPACE_SM)
            .on_press(Message::DarkModeToggled)
            .style(move |_, status| theme::icon_button(tokens, status)),
        if dark_mode {
            "Passer en mode clair"
        } else {
            "Passer en mode sombre"
        },
        tokens,
    );

    container(
        row![
            icon_image_down().size(30).color(tokens.primary_text),
            text("Lotpix")
                .size(theme::TEXT_XL)
                .font(theme::FONT_SEMIBOLD)
                .color(tokens.text),
            space().width(theme::SPACE_XL),
            tab("Convertir", icon_image_down(), Tab::Convert),
            tab("Historique", icon_history(), Tab::History),
            space().width(Fill),
            dark_mode_toggle,
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Center),
    )
    .padding([0.0, theme::SPACE_XL])
    .height(64)
    .align_y(Center)
    .width(Fill)
    .style(move |_| theme::header(tokens))
    .into()
}

pub fn update_banner<'a>(version: Version, tokens: Tokens) -> Element<'a, Message> {
    container(
        row![
            icon_download().size(theme::ICON_MD),
            text(format!("La version {version} est disponible"))
                .size(theme::TEXT_SM)
                .font(theme::FONT_MEDIUM),
            space().width(Fill),
            button(
                text("Télécharger")
                    .size(theme::TEXT_SM)
                    .font(theme::FONT_MEDIUM)
            )
            .padding([theme::SPACE_XS + 2.0, theme::SPACE_MD])
            .on_press(Message::DownloadUpdate)
            .style(move |_, status| theme::primary_button(tokens, status)),
            with_tooltip(
                button(icon_x().size(theme::ICON_MD))
                    .padding(theme::SPACE_XS)
                    .on_press(Message::UpdateDismissed)
                    .style(move |_, status| theme::text_button(
                        status,
                        tokens.primary_soft_text,
                        tokens.text
                    )),
                "Fermer",
                tokens,
            ),
        ]
        .spacing(theme::SPACE_MD)
        .align_y(Center),
    )
    .padding([theme::SPACE_SM, theme::SPACE_XL])
    .width(Fill)
    .style(move |_| theme::banner(tokens))
    .into()
}

pub fn status_bar(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    let count = |predicate: fn(&super::Status) -> bool| {
        app.images
            .iter()
            .filter(|image| predicate(&image.status))
            .count()
    };
    let converted = count(|status| matches!(status, super::Status::Converted(_)));
    let failed = count(|status| matches!(status, super::Status::Failed(_)));

    let mut summary = format!(
        "{} • {}",
        plural(app.images.len(), "image", "images"),
        plural(converted, "convertie", "converties"),
    );
    if failed > 0 {
        summary.push_str(&format!(" • {failed} en erreur"));
    }

    let mut bar = row![
        icon_info().size(theme::TEXT_XS),
        text(summary).size(theme::TEXT_XS)
    ]
    .spacing(theme::SPACE_SM)
    .align_y(Center);

    if let Some(notice) = &app.notice {
        bar = bar.push(
            text(format!("— {notice}"))
                .size(theme::TEXT_XS)
                .color(tokens.warning),
        );
    }
    bar = bar.push(space().width(Fill));

    if app.is_converting {
        let (done, total) = app.batch;
        bar = bar
            .push(text(format!("Conversion en cours… {done} / {total}")).size(theme::TEXT_XS))
            .push(
                progress_bar(0.0..=total as f32, done as f32)
                    .length(Length::Fixed(96.0))
                    .girth(4)
                    .style(move |_| theme::progress_bar_style(tokens)),
            )
            .push(space().width(theme::SPACE_SM));
    }
    bar = bar
        .push(
            checkbox(app.preferences.check_updates)
                .label("Vérifier les mises à jour")
                .on_toggle(Message::UpdateCheckToggled)
                .size(12)
                .text_size(theme::TEXT_XS)
                .spacing(theme::SPACE_XS + 2.0)
                .style(move |_, status| checkbox::Style {
                    text_color: Some(tokens.text_muted),
                    ..theme::checkbox_style(tokens, status)
                }),
        )
        .push(text(format!("v{}", env!("CARGO_PKG_VERSION"))).size(theme::TEXT_XS));

    column![
        separator(tokens),
        container(bar)
            .padding([theme::SPACE_SM, theme::SPACE_LG])
            .width(Fill)
            .style(move |_| theme::status_bar(tokens)),
    ]
    .into()
}
