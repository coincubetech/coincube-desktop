//! Button styles for Tenshu actions.
//!
//! Pill actions follow `.pill.fill` / `.pill.out`, compact buttons `.btn`,
//! quiet links `.rlink`, chips `.chip` and icon actions `.eye` in the v113
//! artboards. Hover uses the shell `hover` overlay; pressed stacks it twice
//! (or darkens an accent fill). Disabled controls fade to `DISABLED_ALPHA`.
//!
//! The button rule (`cc-buttons v1`, canvas v337): every non-primary button in
//! page content is outlined 1px in ink with ink text and no fill at rest; hover
//! adds the light fill and the outline stays ink. Bar buttons are quiet and
//! styled by their bar (`component::tenshu::top_bar::button_style`).

use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color};

use super::{accent_pressed, faded, over, radius, Theme, ACCENT_HOVER, DISABLED_ALPHA, ON_ACCENT};

/// What an action does, which sets its emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    /// The one main action of a row or screen, such as Send.
    Primary,
    /// A peer of the primary action, such as Receive or Transfer.
    Secondary,
    /// A borderless text action, such as Recovery path.
    Quiet,
    /// An action that removes or revokes something.
    Destructive,
}

/// The geometry family of an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionShape {
    /// 40px pill (`.pill`), 10px corners.
    Pill,
    /// 32px button (`.btn`).
    Compact,
    /// 32px chip (`.chip`). The kind does not change its treatment.
    Chip,
    /// 32px square icon-only action (`.eye`). The kind does not change its treatment.
    Icon,
}

impl ActionShape {
    /// Corner radius for this shape and kind; the focus ring follows it.
    pub fn radius(self, kind: ActionKind) -> f32 {
        match (self, kind) {
            (ActionShape::Pill | ActionShape::Compact, ActionKind::Quiet) => radius::LINK,
            (ActionShape::Pill, _) => radius::PILL,
            (ActionShape::Compact, _) | (ActionShape::Icon, _) => radius::CONTROL,
            (ActionShape::Chip, _) => radius::CHIP,
        }
    }
}

/// Style of an enabled or disabled action in the given pointer state.
pub fn action(theme: &Theme, status: Status, kind: ActionKind, shape: ActionShape) -> Style {
    let shell = &theme.colors.shell;
    let radius = shape.radius(kind);
    // (fill, border, ink). A `None` fill shows the surface behind the action.
    let (fill, border, ink) = match (shape, kind) {
        (ActionShape::Icon, _) => (None, None, shell.muted),
        (ActionShape::Chip, _) => (None, Some(shell.ink), shell.ink),
        (_, ActionKind::Primary) => (Some(shell.accent), Some(shell.accent), ON_ACCENT),
        (_, ActionKind::Quiet) => (None, None, shell.accent_ink),
        (ActionShape::Pill | ActionShape::Compact, ActionKind::Secondary) => {
            (None, Some(shell.ink), shell.ink)
        }
        (ActionShape::Pill, ActionKind::Destructive) => (None, Some(shell.bad), shell.bad),
        (ActionShape::Compact, ActionKind::Destructive) => {
            (Some(shell.panel), Some(shell.bad), shell.bad)
        }
    };

    let (fill, ink) = pointer_colors(theme, status, kind, shape, fill, ink);

    let fade = |color: Color| {
        if status == Status::Disabled {
            faded(color, DISABLED_ALPHA)
        } else {
            color
        }
    };

    Style {
        background: fill.map(|color| Background::Color(fade(color))),
        text_color: fade(ink),
        border: Border {
            color: border.map(fade).unwrap_or(Color::TRANSPARENT),
            width: if border.is_some() { 1.0 } else { 0.0 },
            radius: radius.into(),
        },
        ..Style::default()
    }
}

fn pointer_colors(
    theme: &Theme,
    status: Status,
    kind: ActionKind,
    shape: ActionShape,
    fill: Option<Color>,
    ink: Color,
) -> (Option<Color>, Color) {
    let shell = &theme.colors.shell;
    match status {
        Status::Active | Status::Disabled => (fill, ink),
        Status::Hovered | Status::Pressed => {
            let pressed = status == Status::Pressed;
            let ink = if shape == ActionShape::Icon {
                shell.ink
            } else {
                ink
            };
            let fill = fill_for_pointer(theme, kind, shape, fill, pressed);
            (fill, ink)
        }
    }
}

fn fill_for_pointer(
    theme: &Theme,
    kind: ActionKind,
    shape: ActionShape,
    fill: Option<Color>,
    pressed: bool,
) -> Option<Color> {
    let shell = &theme.colors.shell;
    match (kind, shape, fill) {
        (ActionKind::Primary, ActionShape::Pill | ActionShape::Compact, _) => Some(if pressed {
            accent_pressed(theme)
        } else {
            ACCENT_HOVER
        }),
        (_, _, Some(base)) => {
            let once = over(base, shell.hover);
            Some(if pressed {
                over(once, shell.hover)
            } else {
                once
            })
        }
        (_, _, None) => Some(if pressed {
            stacked(shell.hover)
        } else {
            shell.hover
        }),
    }
}

/// Two hover layers over an unknown surface, kept translucent.
fn stacked(hover: Color) -> Color {
    let alpha = 1.0 - (1.0 - hover.a) * (1.0 - hover.a);
    Color { a: alpha, ..hover }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// The button rule's content outline: 1px ink, ink text, no fill at rest;
    /// hover adds the light fill and keeps the ink outline.
    #[test]
    fn content_secondaries_are_an_ink_outline_in_both_themes() {
        for (name, theme) in themes() {
            let shell = theme.colors.shell;
            for (shape, kind) in [
                (ActionShape::Pill, ActionKind::Secondary),
                (ActionShape::Compact, ActionKind::Secondary),
                (ActionShape::Chip, ActionKind::Secondary),
                (ActionShape::Chip, ActionKind::Primary),
            ] {
                let rest = action(&theme, Status::Active, kind, shape);
                assert_eq!(fill(&rest), None, "{name} {shape:?} {kind:?}");
                assert_eq!(rest.border.width, 1.0, "{name} {shape:?} {kind:?}");
                assert_eq!(rest.border.color, shell.ink, "{name} {shape:?} {kind:?}");
                assert_eq!(rest.text_color, shell.ink, "{name} {shape:?} {kind:?}");

                let hover = action(&theme, Status::Hovered, kind, shape);
                assert_eq!(fill(&hover), Some(shell.hover), "{name} {shape:?}");
                assert_eq!(hover.border.width, 1.0, "{name} {shape:?}");
                assert_eq!(hover.border.color, shell.ink, "{name} {shape:?}");
                assert_eq!(hover.text_color, shell.ink, "{name} {shape:?}");

                let off = action(&theme, Status::Disabled, kind, shape);
                assert_eq!(off.text_color.a, DISABLED_ALPHA, "{name} {shape:?}");
                assert_eq!(off.border.color.a, DISABLED_ALPHA, "{name} {shape:?}");
            }
        }
    }

    #[test]
    fn the_primary_is_filled_accent_with_one_hover_in_both_themes() {
        for (name, theme) in themes() {
            let shell = theme.colors.shell;
            for shape in [ActionShape::Pill, ActionShape::Compact] {
                let rest = action(&theme, Status::Active, ActionKind::Primary, shape);
                assert_eq!(fill(&rest), Some(shell.accent), "{name} {shape:?}");
                assert_eq!(rest.border.width, 1.0, "{name} {shape:?}");
                assert_eq!(rest.border.color, shell.accent, "{name} {shape:?}");
                assert_eq!(rest.text_color, ON_ACCENT, "{name} {shape:?}");

                let hover = action(&theme, Status::Hovered, ActionKind::Primary, shape);
                assert_eq!(fill(&hover), Some(ACCENT_HOVER), "{name} {shape:?}");
                assert_eq!(ACCENT_HOVER, iced::color!(0xff9a3f));

                let off = action(&theme, Status::Disabled, ActionKind::Primary, shape);
                assert_eq!(fill(&off).unwrap().a, DISABLED_ALPHA, "{name} {shape:?}");
                assert_eq!(off.text_color.a, DISABLED_ALPHA, "{name} {shape:?}");
            }
        }
    }

    /// Phase 1 D4: the Receive and Send pill is 10px; other buttons stay 8px.
    #[test]
    fn only_the_pill_takes_the_10px_corner() {
        let theme = Theme::dark();
        for kind in [
            ActionKind::Primary,
            ActionKind::Secondary,
            ActionKind::Destructive,
        ] {
            let pill = action(&theme, Status::Active, kind, ActionShape::Pill);
            assert_eq!(pill.border.radius.top_left, 10.0, "{kind:?}");
            let compact = action(&theme, Status::Active, kind, ActionShape::Compact);
            assert_eq!(compact.border.radius.top_left, 8.0, "{kind:?}");
        }
        assert_eq!(super::super::size::PILL_HEIGHT, 40.0);
    }
}
