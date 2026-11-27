//! The legacy card family.
//!
//! Desktop V3 (plan V5): a neutral card is a fill and a 16 px radius, with no
//! outline. Coloured alert cards (`invalid`, `warning`, `error`) keep their
//! coloured border and fill (invariant R5); they only share the radius.
//!
//! A box must never sit on a box of the same fill with nothing between them
//! (invariant R4). A card inside a panel-filled card uses [`inset`]; a card
//! whose parent fill differs by context or theme uses [`outlined`]; a tooltip
//! bubble, which floats over whatever is underneath, uses [`tooltip`].

use iced::widget::container::Style;
use iced::{Background, Border, Color};

use super::palette::ContainerPalette;
use super::tenshu::radius;
use super::Theme;

/// The one card radius, shared with `tenshu::surface::card`.
pub const RADIUS: f32 = radius::CARD;

/// A neutral container: fill only.
fn neutral(fill: Color, text_color: Option<Color>) -> Style {
    Style {
        background: Some(Background::Color(fill)),
        text_color,
        border: Border {
            radius: RADIUS.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

/// A coloured alert card: keeps its palette border.
fn alert(palette: &ContainerPalette) -> Style {
    Style {
        background: Some(Background::Color(palette.background)),
        text_color: palette.text,
        border: match palette.border {
            Some(color) => Border {
                radius: RADIUS.into(),
                width: 1.0,
                color,
            },
            None => Border::default(),
        },
        ..Default::default()
    }
}

pub fn simple(theme: &Theme) -> Style {
    let palette = &theme.colors.cards.simple;
    neutral(palette.background, palette.text)
}

pub fn modal(theme: &Theme) -> Style {
    let palette = &theme.colors.cards.modal;
    neutral(palette.background, palette.text)
}

/// Formerly a transparent box with a grey outline. With the outline gone it
/// takes the panel fill, so on the page background it still reads as a card.
/// Inside a panel-filled card use [`inset`] instead.
pub fn border(theme: &Theme) -> Style {
    neutral(theme.colors.shell.panel, theme.colors.cards.border.text)
}

/// A card inside a panel-filled card: the page background fill separates it
/// from its parent (R4).
pub fn inset(theme: &Theme) -> Style {
    neutral(theme.colors.shell.background, None)
}

/// A card that keeps a 1 px line outline and no fill, for a box whose parent
/// fill is not known (it is drawn on the page in one place and inside a
/// panel-filled sheet in another, or inside a modal whose fill differs by
/// theme). R4 allows such a box to keep its border.
pub fn outlined(theme: &Theme) -> Style {
    Style {
        background: None,
        text_color: None,
        border: Border {
            radius: RADIUS.into(),
            width: 1.0,
            color: theme.colors.shell.line,
        },
        ..Default::default()
    }
}

/// A tooltip bubble. It floats over a panel or the page alike, so it keeps a
/// 1 px line outline around its panel fill (R4).
pub fn tooltip(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(theme.colors.shell.panel)),
        text_color: None,
        border: Border {
            radius: RADIUS.into(),
            width: 1.0,
            color: theme.colors.shell.line,
        },
        ..Default::default()
    }
}

pub fn invalid(theme: &Theme) -> Style {
    alert(&theme.colors.cards.invalid)
}

pub fn warning(theme: &Theme) -> Style {
    alert(&theme.colors.cards.warning)
}

pub fn error(theme: &Theme) -> Style {
    alert(&theme.colors.cards.error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn themes() -> [Theme; 2] {
        [Theme::dark(), Theme::light()]
    }

    fn fill(style: &Style) -> Option<Color> {
        match style.background {
            Some(Background::Color(c)) => Some(c),
            _ => None,
        }
    }

    #[test]
    fn neutral_cards_have_no_border_and_radius_16() {
        for theme in themes() {
            for (name, style) in [
                ("simple", simple(&theme)),
                ("modal", modal(&theme)),
                ("border", border(&theme)),
                ("inset", inset(&theme)),
            ] {
                assert_eq!(style.border.width, 0.0, "{name} in {:?}", theme.mode);
                assert_eq!(
                    style.border.radius.top_left, 16.0,
                    "{name} in {:?}",
                    theme.mode
                );
                assert_eq!(style.border.radius.bottom_right, 16.0, "{name}");
                assert!(fill(&style).is_some(), "{name} must carry a fill");
            }
        }
    }

    #[test]
    fn border_card_takes_the_panel_fill() {
        for theme in themes() {
            assert_eq!(fill(&border(&theme)), Some(theme.colors.shell.panel));
            assert_eq!(fill(&simple(&theme)), Some(theme.colors.shell.panel));
        }
    }

    #[test]
    fn coloured_alerts_keep_their_border() {
        for theme in themes() {
            for (name, style, palette) in [
                ("invalid", invalid(&theme), theme.colors.cards.invalid),
                ("warning", warning(&theme), theme.colors.cards.warning),
                ("error", error(&theme), theme.colors.cards.error),
            ] {
                assert_eq!(style.border.width, 1.0, "{name} in {:?}", theme.mode);
                assert_eq!(Some(style.border.color), palette.border, "{name}");
                assert_eq!(fill(&style), Some(palette.background), "{name}");
                assert_eq!(style.text_color, palette.text, "{name}");
                assert_eq!(style.border.radius.top_left, 16.0, "{name}");
            }
        }
    }

    #[test]
    fn inset_separates_from_a_panel_card() {
        for theme in themes() {
            assert_ne!(
                fill(&inset(&theme)),
                fill(&simple(&theme)),
                "inset must differ from the panel fill in {:?}",
                theme.mode
            );
        }
    }

    #[test]
    fn outlined_and_tooltip_keep_a_line_border() {
        for theme in themes() {
            for (name, style) in [("outlined", outlined(&theme)), ("tooltip", tooltip(&theme))] {
                assert_eq!(style.border.width, 1.0, "{name}");
                assert_eq!(style.border.color, theme.colors.shell.line, "{name}");
                assert_eq!(style.border.radius.top_left, 16.0, "{name}");
            }
            assert_eq!(outlined(&theme).background, None);
        }
    }
}
