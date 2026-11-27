//! Container styles for Tenshu surfaces.

use iced::widget::container::Style;
use iced::{Background, Border, Shadow, Vector};

use super::{radius, Theme};

/// Wallet card (`.card`): panel fill, no outline, 16px radius.
///
/// Boxes have no outline (Desktop V3 Phase 1, V4): the panel fill against the
/// page background is the edge. Anything set inside a card takes the page
/// background, or keeps its own border, so it does not vanish (R4).
pub fn card(theme: &Theme) -> Style {
    let shell = &theme.colors.shell;
    Style {
        text_color: Some(shell.ink),
        background: Some(Background::Color(shell.panel)),
        border: Border {
            radius: radius::CARD.into(),
            ..Border::default()
        },
        ..Style::default()
    }
}

/// Menu, popover or sheet (`.menu`): card treatment at a 10px radius with the
/// shell shadow.
pub fn panel(theme: &Theme) -> Style {
    let shell = &theme.colors.shell;
    Style {
        text_color: Some(shell.ink),
        background: Some(Background::Color(shell.panel)),
        border: Border {
            color: shell.line,
            width: 1.0,
            radius: radius::MENU.into(),
        },
        shadow: Shadow {
            color: shell.shadow,
            offset: Vector::new(0.0, 12.0),
            blur_radius: 32.0,
        },
        ..Style::default()
    }
}

/// Window background behind cards (`--bg`).
pub fn page(theme: &Theme) -> Style {
    let shell = &theme.colors.shell;
    Style {
        text_color: Some(shell.ink),
        background: Some(Background::Color(shell.background)),
        ..Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A card is a fill with no outline, and its fill differs from the page
    /// behind it in both themes, so it never vanishes.
    #[test]
    fn card_is_a_fill_without_an_outline_in_both_themes() {
        for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
            let style = card(&theme);
            assert_eq!(style.border.width, 0.0, "{name}");
            assert_eq!(style.border.radius, radius::CARD.into(), "{name}");
            assert_eq!(
                style.background,
                Some(Background::Color(theme.colors.shell.panel)),
                "{name}"
            );
            // Visibly apart, not merely unequal (R4). Light theme's #ffffff
            // on #faf9f6 is the closest pair, at about 2.6.
            let fill = |s: Style| match s.background {
                Some(Background::Color(color)) => color,
                other => panic!("{:?}", other),
            };
            let separation = super::super::separation::delta_e(fill(style), fill(page(&theme)));
            assert!(
                separation >= super::super::separation::JUST_NOTICEABLE_DELTA_E,
                "{name}: card on page is only {separation} apart"
            );
        }
    }
}
