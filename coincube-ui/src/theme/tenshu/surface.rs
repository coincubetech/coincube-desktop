//! Container styles for Tenshu surfaces.

use iced::widget::container::Style;
use iced::{Background, Border, Shadow, Vector};

use super::{radius, Theme};

/// Wallet card (`.card`): panel fill, 1px line, 16px radius.
pub fn card(theme: &Theme) -> Style {
    let shell = &theme.colors.shell;
    Style {
        text_color: Some(shell.ink),
        background: Some(Background::Color(shell.panel)),
        border: Border {
            color: shell.line,
            width: 1.0,
            radius: radius::CARD.into(),
        },
        ..Style::default()
    }
}

/// Menu, popover or sheet (`.menu`): the panel fill at a 10px radius with
/// the shell shadow and no outline. It floats, so the fill and the shadow set
/// it apart (V3: a box with a shadow has no outline).
pub fn panel(theme: &Theme) -> Style {
    let shell = &theme.colors.shell;
    Style {
        text_color: Some(shell.ink),
        background: Some(Background::Color(shell.panel)),
        border: Border {
            radius: radius::MENU.into(),
            ..Border::default()
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

    /// A floating panel is set apart by its shadow, so it carries no outline;
    /// a card has no shadow, so (until V4) it keeps its line.
    #[test]
    fn the_floating_panel_has_a_shadow_and_no_outline() {
        for theme in [Theme::dark(), Theme::light()] {
            let style = panel(&theme);
            assert_eq!(style.border.width, 0.0);
            assert_eq!(style.border.radius.top_left, radius::MENU);
            assert!(style.shadow.color.a > 0.0 && style.shadow.blur_radius > 0.0);
            assert_eq!(card(&theme).shadow.color.a, 0.0);
            assert_eq!(card(&theme).border.width, 1.0);
        }
    }
}
