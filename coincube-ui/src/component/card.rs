use crate::{
    color,
    component::text::text,
    icon, theme,
    widget::{Container, Element, Row, Text},
};

pub fn modal<'a, T: 'a, C: Into<Element<'a, T>>>(content: C) -> Container<'a, T> {
    Container::new(content)
        .padding(15)
        .style(theme::card::modal)
}

pub fn simple<'a, T: 'a, C: Into<Element<'a, T>>>(content: C) -> Container<'a, T> {
    Container::new(content)
        .padding(15)
        .style(theme::card::simple)
}

/// A card inside a panel-filled card: background fill, no outline (R4).
pub fn inset<'a, T: 'a, C: Into<Element<'a, T>>>(content: C) -> Container<'a, T> {
    Container::new(content)
        .padding(15)
        .style(theme::card::inset)
}

/// A card that keeps its line outline because its parent fill varies by
/// context or theme (R4).
pub fn outlined<'a, T: 'a, C: Into<Element<'a, T>>>(content: C) -> Container<'a, T> {
    Container::new(content)
        .padding(15)
        .style(theme::card::outlined)
}

pub fn invalid<'a, T: 'a, C: Into<Element<'a, T>>>(content: C) -> Container<'a, T> {
    Container::new(content)
        .padding(15)
        .style(theme::card::invalid)
}

/// A warning notice with `message` as its title, in the one notice anatomy
/// ([`super::notice`]).
pub fn warning<'a, T: 'a + Clone>(message: String) -> Container<'a, T> {
    super::notice::notice(
        super::notice::NoticeTone::Warning,
        crate::locale::notice(message),
    )
    .into_container()
}

/// display an error card with the message and the error in a tooltip.
pub fn error<'a, T: 'a>(message: &'static str, error: String) -> Container<'a, T> {
    Container::new(
        iced::widget::tooltip::Tooltip::new(
            Row::new()
                .spacing(20)
                .align_y(iced::Alignment::Center)
                .push(icon::warning_icon().color(color::RED))
                .push(text(message).color(color::RED)),
            Text::new(crate::locale::notice(error)),
            iced::widget::tooltip::Position::Bottom,
        )
        .style(theme::card::error),
    )
    .padding(15)
    .style(theme::card::error)
}
