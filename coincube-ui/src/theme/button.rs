use iced::widget::button::{Catalog, Status, Style, StyleFn};
use iced::{Background, Border, Color};

use super::palette::Button;
use super::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// Filled accent. Hover and press use the app's one primary hover
/// (`tenshu::ACCENT_HOVER`); disabled is the resting style at
/// `tenshu::DISABLED_ALPHA`, as in Tenshu.
pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = &theme.colors.buttons.primary;
    match status {
        Status::Disabled => faded_style(button(palette, Status::Active)),
        _ => button(palette, status),
    }
}

/// The content button rule (`cc-buttons v1`): a 1px ink outline, ink text and
/// no fill at rest; hover and press add the shell's light fill and the
/// outline stays ink. Disabled fades the resting style.
pub fn secondary(theme: &Theme, status: Status) -> Style {
    let shell = &theme.colors.shell;
    let resting = Style {
        background: None,
        text_color: shell.ink,
        border: Border {
            radius: super::tenshu::radius::CONTROL.into(),
            width: 1.0,
            color: shell.ink,
        },
        ..Default::default()
    };
    match status {
        Status::Active => resting,
        Status::Hovered | Status::Pressed => Style {
            background: Some(Background::Color(shell.hover)),
            ..resting
        },
        Status::Disabled => faded_style(resting),
    }
}

/// `style` at `tenshu::DISABLED_ALPHA`, like a disabled Tenshu action.
fn faded_style(style: Style) -> Style {
    let fade = |color: Color| super::tenshu::faded(color, super::tenshu::DISABLED_ALPHA);
    Style {
        background: style.background.map(|background| match background {
            Background::Color(color) => Background::Color(fade(color)),
            other => other,
        }),
        text_color: fade(style.text_color),
        border: Border {
            color: fade(style.border.color),
            ..style.border
        },
        ..style
    }
}

pub fn destructive(theme: &Theme, status: Status) -> Style {
    button(&theme.colors.buttons.destructive, status)
}

pub fn container(theme: &Theme, status: Status) -> Style {
    button(&theme.colors.buttons.container, status)
}

pub fn container_border(theme: &Theme, status: Status) -> Style {
    button(&theme.colors.buttons.container_border, status)
}

pub fn menu(theme: &Theme, status: Status) -> Style {
    button(&theme.colors.buttons.menu, status)
}

pub fn menu_pressed(theme: &Theme, _status: Status) -> Style {
    button(&theme.colors.buttons.menu, Status::Pressed)
}

pub fn transparent(theme: &Theme, status: Status) -> Style {
    button(&theme.colors.buttons.transparent, status)
}

pub fn transparent_border(theme: &Theme, status: Status) -> Style {
    button(&theme.colors.buttons.transparent_border, status)
}

pub fn link(theme: &Theme, status: Status) -> Style {
    button(&theme.colors.buttons.link, status)
}

/// Bordered buttons take the Tenshu `.btn` corner (8px), not a pill: the
/// artboards keep pills only for wallet actions such as Send and Receive.
fn button(p: &Button, status: Status) -> Style {
    match status {
        Status::Active => palette_style(&p.active, p.active.text),
        Status::Pressed => {
            if let Some(pressed) = p.pressed {
                palette_style(&pressed, pressed.text)
            } else {
                button(p, Status::Active)
            }
        }
        Status::Hovered => palette_style(&p.hovered, p.hovered.text),
        Status::Disabled => {
            if let Some(disabled) = p.disabled {
                palette_style(
                    &disabled,
                    Color {
                        a: 0.2,
                        r: disabled.text.r,
                        g: disabled.text.g,
                        b: disabled.text.b,
                    },
                )
            } else {
                let active: Style = button(p, Status::Active);

                Style {
                    text_color: Color {
                        a: 0.2,
                        ..active.text_color
                    },
                    ..active
                }
            }
        }
    }
}

fn palette_style(palette: &super::palette::ButtonPalette, text_color: Color) -> Style {
    Style {
        background: Some(Background::Color(palette.background)),
        text_color,
        border: if let Some(color) = palette.border {
            Border {
                radius: super::tenshu::radius::CONTROL.into(),
                width: 1.0,
                color,
            }
        } else {
            Border {
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub fn tab(theme: &Theme, status: Status) -> Style {
    let mut style = button(&theme.colors.buttons.tab, status);
    style.border.radius = 0.0.into();
    style.border.width = 0.0;
    style
}

/// Text-first rows of the Tenshu sidebar and Cube switcher.
pub fn shell_nav(theme: &Theme, status: Status) -> Style {
    shell_nav_with_state(theme, status, false)
}

pub fn shell_nav_active(theme: &Theme, status: Status) -> Style {
    shell_nav_with_state(theme, status, true)
}

fn shell_nav_with_state(theme: &Theme, status: Status, selected: bool) -> Style {
    let colors = &theme.colors.shell;
    let background = if matches!(status, Status::Hovered | Status::Pressed) {
        colors.hover
    } else if selected {
        colors.selected
    } else {
        Color::TRANSPARENT
    };
    Style {
        background: Some(Background::Color(background)),
        text_color: if matches!(status, Status::Disabled) {
            colors.muted
        } else {
            colors.ink
        },
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Inactive primary-rail item. Square corners, no border, transparent
/// background with the standard menu text color — hover darkens the
/// background to the content-area tone to give click feedback.
pub fn rail(theme: &Theme, status: Status) -> Style {
    let mut style = button(&theme.colors.buttons.menu, status);
    style.border.radius = 0.0.into();
    style.border.width = 0.0;
    style
}

/// Active primary-rail item. Same square corners as `rail` but forced
/// into the pressed state so the active section reads as "selected"
/// even when not hovered.
pub fn rail_active(theme: &Theme, _status: Status) -> Style {
    let mut style = button(&theme.colors.buttons.menu, Status::Pressed);
    style.border.radius = 0.0.into();
    style.border.width = 0.0;
    style
}

/// Disabled rail item — network-gated features that are greyed out with
/// a hover popover instead of being hidden. Forces the disabled
/// styling (muted text + icon) regardless of hover so the item reads as
/// inert. Used by both the primary and secondary rails.
pub fn rail_disabled(theme: &Theme, _status: Status) -> Style {
    let mut style = button(&theme.colors.buttons.menu, Status::Disabled);
    style.border.radius = 0.0.into();
    style.border.width = 0.0;
    // The default disabled alpha (0.2) is nearly invisible on the dark
    // rail. Lift it so the greyed icon + label stay legible while still
    // reading clearly as inert next to the full-strength active items.
    style.text_color.a = 0.55;
    style
}

/// The selected tab: [`tab`] held in its pressed state.
pub fn tab_active(theme: &Theme, _status: Status) -> Style {
    tab(theme, Status::Pressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::tenshu::{ACCENT_HOVER, DISABLED_ALPHA};

    fn themes() -> [(&'static str, Theme); 2] {
        [("dark", Theme::dark()), ("light", Theme::light())]
    }

    fn fill(style: &Style) -> Option<Color> {
        match style.background {
            Some(Background::Color(color)) => Some(color),
            None => None,
            Some(other) => panic!("unexpected background {other:?}"),
        }
    }

    /// The legacy secondary follows the content rule: an ink outline, and a
    /// hover that adds the light fill without turning the label orange.
    #[test]
    fn legacy_secondary_is_the_content_outline_in_both_themes() {
        for (name, theme) in themes() {
            let shell = theme.colors.shell;
            let rest = secondary(&theme, Status::Active);
            assert_eq!(fill(&rest), None, "{name}");
            assert_eq!(rest.border.width, 1.0, "{name}");
            assert_eq!(rest.border.color, shell.ink, "{name}");
            assert_eq!(rest.border.radius.top_left, 8.0, "{name}");
            assert_eq!(rest.text_color, shell.ink, "{name}");
            for status in [Status::Hovered, Status::Pressed] {
                let style = secondary(&theme, status);
                assert_eq!(fill(&style), Some(shell.hover), "{name} {status:?}");
                assert_eq!(style.border.color, shell.ink, "{name} {status:?}");
                assert_eq!(style.text_color, shell.ink, "{name} {status:?}");
            }
            let off = secondary(&theme, Status::Disabled);
            assert_eq!(off.text_color.a, DISABLED_ALPHA, "{name}");
        }
    }

    #[test]
    fn legacy_primary_shares_the_one_primary_hover_in_both_themes() {
        for (name, theme) in themes() {
            let rest = primary(&theme, Status::Active);
            assert_eq!(fill(&rest), Some(theme.colors.shell.accent), "{name}");
            assert_eq!(rest.border.width, 1.0, "{name}");
            for status in [Status::Hovered, Status::Pressed] {
                let style = primary(&theme, status);
                assert_eq!(fill(&style), Some(ACCENT_HOVER), "{name} {status:?}");
                assert_eq!(style.border.color, ACCENT_HOVER, "{name} {status:?}");
            }
            let off = primary(&theme, Status::Disabled);
            assert_eq!(
                fill(&off),
                Some(crate::theme::tenshu::faded(
                    theme.colors.shell.accent,
                    DISABLED_ALPHA
                )),
                "{name}"
            );
            assert_eq!(off.text_color.a, DISABLED_ALPHA, "{name}");
        }
    }
}
