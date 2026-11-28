use std::fmt::Display;

use crate::{
    component::{collapse, text},
    icon, theme,
    widget::{Button, Container, Element, Row},
};
use iced::{
    widget::{column, container, row},
    Alignment, Length,
};

/// A warning notice: `message` as the title and `error` as its line (left
/// out when it only repeats the title). Draws the one notice anatomy
/// ([`super::notice`]).
pub fn warning<'a, T: 'a + Clone>(message: String, error: String) -> Container<'a, T> {
    let title = crate::locale::notice(message.clone());
    let mut notice = super::notice::notice(super::notice::NoticeTone::Warning, title);
    if error != message {
        notice = notice.detail(crate::locale::notice(error));
    }
    notice.into_container()
}

pub fn processing_hardware_wallet<'a, T: 'a, K: Display, V: Display, F: Display>(
    kind: K,
    version: Option<V>,
    fingerprint: F,
    alias: Option<&'a str>,
) -> Container<'a, T> {
    container(
        row(vec![
            column(vec![
                Row::new()
                    .spacing(5)
                    .push(alias.map(text::p1_bold))
                    .push(text::p1_regular(format!("#{}", fingerprint)))
                    .into(),
                Row::new()
                    .spacing(5)
                    .push(text::caption(kind.to_string()))
                    .push(version.map(|v| text::caption(v.to_string())))
                    .into(),
            ])
            .width(Length::Fill)
            .into(),
            column(vec![
                text::p2_regular("Processing...").into(),
                text::p2_regular("Please check your device").into(),
            ])
            .into(),
        ])
        .align_y(Alignment::Center),
    )
    .style(theme::notification::pending)
    .padding(10)
}

pub fn processing_hardware_wallet_error<'a, T: 'a + Clone>(
    message: String,
    error: String,
) -> Container<'a, T> {
    let message = crate::locale::notice(message);
    let error = crate::locale::notice(error);
    let learn_more = crate::locale::copy("Learn more");
    let message_clone = message.clone();
    Container::new(Container::new(collapse::Collapse::new(
        move || {
            Button::new(
                Row::new()
                    .push(
                        Container::new(text::p1_bold(message_clone.to_string()))
                            .width(Length::Fill),
                    )
                    .push(
                        Row::new()
                            .align_y(Alignment::Center)
                            .spacing(10)
                            .push(text::p1_bold(learn_more))
                            .push(icon::collapse_icon()),
                    ),
            )
            .style(theme::button::transparent)
        },
        move || {
            Button::new(
                Row::new()
                    .push(Container::new(text::p1_bold(message.to_owned())).width(Length::Fill))
                    .push(
                        Row::new()
                            .align_y(Alignment::Center)
                            .spacing(10)
                            .push(text::p1_bold(learn_more))
                            .push(icon::collapsed_icon()),
                    ),
            )
            .style(theme::button::transparent)
        },
        move || Element::<'a, T>::from(text::p2_regular(error.to_owned())),
    )))
    .padding(10)
    .style(theme::notification::error)
    .width(Length::Fill)
}

/// A blocking error card: a plain-language headline, a line saying what to do
/// next, an optional action button, and a short support reference.
///
/// This is the component every user-visible failure should route through. The
/// raw technical detail (a `reqwest::Error`, an SDK string, an HTTP body)
/// deliberately has no parameter here — it belongs in the log file, not on
/// screen. `reference` is the opaque code that ties what the user sees to the
/// logged detail, so support can triage without asking for a log dump (S7).
///
/// Draws the one notice anatomy ([`super::notice`]) in the error tone: the
/// panel washed in the tone, the action an outlined content button.
pub fn error_card<'a, T: 'a + Clone>(
    title: impl Into<String>,
    guidance: impl Into<String>,
    reference: impl Into<String>,
    action: Option<(&'static str, T)>,
) -> Container<'a, T> {
    let mut notice = super::notice::notice(
        super::notice::NoticeTone::Error,
        crate::locale::notice(title.into()),
    )
    .detail(crate::locale::notice(guidance.into()))
    .reference(reference);
    if let Some((label, message)) = action {
        notice = notice.action(crate::locale::copy(label), message);
    }
    notice.into_container()
}
