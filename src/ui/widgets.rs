// Éléments communs aux deux écrans : en-tête, barre de statut, infobulles, états vides
use iced::widget::{
    Text, button, checkbox, column, container, pick_list, progress_bar, row, space, text, tooltip,
};
use iced::{Center, Element, Fill, Length};
use lucide_icons::iced::{
    icon_download, icon_history, icon_image_down, icon_info, icon_moon, icon_sun, icon_x,
};

use super::theme::{self, Tokens};
use super::{App, LANGUAGE_CHOICES, Message, Tab};
use crate::t;
use crate::update::Version;

pub fn with_tooltip<'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl text::IntoFragment<'a>,
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
pub fn empty_state<'a>(
    icon: Text<'a>,
    label: impl text::IntoFragment<'a>,
    tokens: Tokens,
) -> Element<'a, Message> {
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

    let tab = |label: String, icon: Text<'static>, tab: Tab| {
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
            t!("theme-to-light")
        } else {
            t!("theme-to-dark")
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
            tab(t!("tab-convert"), icon_image_down(), Tab::Convert),
            tab(t!("tab-history"), icon_history(), Tab::History),
            space().width(Fill),
            language_picker(app),
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

/// Langue de l'interface : celle du système ou une langue imposée.
fn language_picker(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    with_tooltip(
        pick_list(
            &LANGUAGE_CHOICES[..],
            Some(app.preferences.language),
            Message::LanguageSelected,
        )
        .padding([theme::SPACE_XS, theme::SPACE_SM])
        .text_size(theme::TEXT_SM)
        .style(move |_, status| theme::pick_list_style(tokens, status))
        .menu_style(move |_| theme::menu(tokens)),
        t!("language"),
        tokens,
    )
}

pub fn update_banner<'a>(version: Version, tokens: Tokens) -> Element<'a, Message> {
    container(
        row![
            icon_download().size(theme::ICON_MD),
            text(t!("update-available", version = version.to_string()))
                .size(theme::TEXT_SM)
                .font(theme::FONT_MEDIUM),
            space().width(Fill),
            button(
                text(t!("update-download"))
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
                t!("update-dismiss"),
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
        t!("status-images", count = app.images.len()),
        t!("status-converted", count = converted),
    );
    if failed > 0 {
        summary.push_str(&format!(" • {}", t!("status-failed", count = failed)));
    }

    let mut bar = row![
        icon_info().size(theme::TEXT_XS),
        text(summary).size(theme::TEXT_XS)
    ]
    .spacing(theme::SPACE_SM)
    .align_y(Center);

    // Un message trop long pour la langue courante est coupé, plutôt que de repousser le reste
    // de la barre hors de la fenêtre
    bar = match &app.notice {
        Some(notice) => bar.push(
            container(
                text(format!("— {notice}"))
                    .size(theme::TEXT_XS)
                    .color(tokens.warning)
                    .wrapping(text::Wrapping::None),
            )
            .width(Fill)
            .clip(true),
        ),
        None => bar.push(space().width(Fill)),
    };

    if app.is_converting {
        let (done, total) = app.batch;
        bar = bar
            .push(text(t!("status-converting", done = done, total = total)).size(theme::TEXT_XS))
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
                .label(t!("status-check-updates"))
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
