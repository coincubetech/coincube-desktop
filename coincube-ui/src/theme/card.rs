//! The legacy card family.
//!
//! Desktop V3 (plan V5): a neutral card is a fill and a 16 px radius, with no
//! outline. Coloured alert cards (`invalid`, `warning`, `error`) are not
//! restyled in Phase 1 (invariant R5): they keep their coloured border, fill
//! and the legacy 25 px radius.
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

/// The legacy radius coloured alerts keep until Phase 2 (R5).
pub const ALERT_RADIUS: f32 = 25.0;

/// A coloured alert card, unchanged from before V3 (R5).
fn alert(palette: &ContainerPalette) -> Style {
    Style {
        background: Some(Background::Color(palette.background)),
        text_color: palette.text,
        border: match palette.border {
            Some(color) => Border {
                radius: ALERT_RADIUS.into(),
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
                assert_eq!(style.border.radius.top_left, 25.0, "{name} keeps 25 (R5)");
            }
        }
    }

    /// WCAG relative luminance of an sRGB colour.
    fn luminance(c: Color) -> f32 {
        let lin = |v: f32| {
            if v <= 0.040_45 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }

    /// WCAG contrast ratio between two opaque colours (always >= 1).
    fn contrast(a: Color, b: Color) -> f32 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    /// The fill-only design separates a card from what it sits on by fill
    /// alone. The pair is the same as a card on the page (panel on
    /// background): about 1.10:1 in dark and 1.05:1 in light. This floor is
    /// a regression guard on that pair, not an accessibility claim; it is
    /// far below WCAG 1.4.11's 3:1 for UI components.
    const MIN_FILL_SEPARATION: f32 = 1.04;

    #[test]
    fn inset_separates_from_a_panel_card() {
        for theme in themes() {
            let inset = fill(&inset(&theme)).unwrap();
            let panel = fill(&simple(&theme)).unwrap();
            let ratio = contrast(inset, panel);
            assert!(
                ratio >= MIN_FILL_SEPARATION,
                "inset vs panel is {ratio:.3}:1 in {:?}, below {MIN_FILL_SEPARATION}:1",
                theme.mode
            );
        }
    }

    /// The `outlined` and `tooltip` line must stand apart from the panel it
    /// borders: about 1.62:1 in dark and 1.85:1 in light. Also below WCAG's
    /// 3:1, like every line token in the shell palette.
    const MIN_LINE_SEPARATION: f32 = 1.5;

    #[test]
    fn outline_stands_apart_from_the_panel() {
        for theme in themes() {
            let panel = theme.colors.shell.panel;
            for (name, style) in [("outlined", outlined(&theme)), ("tooltip", tooltip(&theme))] {
                let ratio = contrast(style.border.color, panel);
                assert!(
                    ratio >= MIN_LINE_SEPARATION,
                    "{name} line vs panel is {ratio:.3}:1 in {:?}, below {MIN_LINE_SEPARATION}:1",
                    theme.mode
                );
                // Over the page background too, where `outlined` often sits.
                let ratio = contrast(style.border.color, theme.colors.shell.background);
                assert!(
                    ratio >= MIN_LINE_SEPARATION,
                    "{name} line vs background {ratio:.3}:1"
                );
            }
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
