pub mod amount;
pub mod badge;
pub mod button;
pub mod card;
pub mod collapse;
pub mod date_picker;
pub mod form;
pub mod hw;
pub mod modal;
pub mod notice;
pub mod notification;
pub mod pagination;
pub mod payment_result;
pub mod quote_display;
pub mod ring_mark;
pub mod session_status;
pub mod spinner;
pub mod tabs;
pub mod tenshu;
pub mod test_network;
pub mod text;
pub mod toast;
pub mod tooltip;
pub mod transaction;
pub mod wordmark;

pub use test_network::{test_network_name, test_network_tag};
pub use tooltip::{tooltip, tooltip_custom};

use iced::Length;

use crate::{
    theme,
    widget::{Column, Container, Element},
};

use self::text::Text;

pub fn separation<'a, T: 'a>() -> Container<'a, T> {
    Container::new(Column::new().push(iced::widget::space()))
        .style(theme::container::border)
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
}

/// Empty-state placeholder card — centered icon + title + subtitle on a
/// card background with rounded corners. Used by transaction lists
/// across the Spark and Liquid wallets when the user has no payment
/// history yet. Parameterized over the view's message type so the same
/// helper serves panels that emit different message enums.
pub fn empty_placeholder<'a, M: 'a + 'static, T: Into<Element<'a, M>>>(
    icon: T,
    title: &'a str,
    subtitle: &'a str,
) -> Element<'a, M> {
    let content = Column::new()
        .push(icon)
        .push(text::text(title).style(theme::text::secondary).bold())
        .push(
            text::text(subtitle)
                .size(text::P2_SIZE)
                .style(theme::text::secondary)
                .align_x(iced::Alignment::Center),
        )
        .spacing(16)
        .align_x(iced::Alignment::Center);

    iced::widget::Container::new(content)
        .width(Length::Fill)
        .padding(60)
        .center_x(Length::Fill)
        .style(|t| iced::widget::container::Style {
            background: Some(iced::Background::Color(t.colors.cards.simple.background)),
            border: iced::Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
