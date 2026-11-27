use iced::widget::container::Style;
use iced::{Background, Border};

use super::palette::ContainerPalette;
use super::Theme;

fn notification(palette: &ContainerPalette) -> Style {
    Style {
        background: Some(Background::Color(palette.background)),
        text_color: palette.text,
        border: if let Some(color) = palette.border {
            Border {
                width: 1.0,
                color,
                radius: 25.0.into(),
            }
        } else {
            Border {
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub fn pending(theme: &Theme) -> Style {
    notification(&theme.colors.notifications.pending)
}

pub fn error(theme: &Theme) -> Style {
    notification(&theme.colors.notifications.error)
}

pub fn success(theme: &Theme) -> Style {
    notification(&theme.colors.notifications.success)
}

pub fn warning(theme: &Theme) -> Style {
    notification(&theme.colors.notifications.warning)
}

pub fn info(theme: &Theme) -> Style {
    notification(&theme.colors.notifications.info)
}

/// Severity of a toast. Sets the icon, its tint and whether the panel is
/// washed in the tone; every toast otherwise shares the panel surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastTone {
    /// Confirmations and background progress.
    Info,
    /// A user action finished and stuck.
    Success,
    /// Something degraded but the Cube still works.
    Warning,
    /// An action failed.
    Error,
    /// Developer-only detail.
    Debug,
}

impl From<log::Level> for ToastTone {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => ToastTone::Error,
            log::Level::Warn => ToastTone::Warning,
            log::Level::Info => ToastTone::Info,
            log::Level::Debug | log::Level::Trace => ToastTone::Debug,
        }
    }
}

impl ToastTone {
    /// Tone colour for the icon and, on toasts that need attention, the wash.
    pub fn color(self, theme: &Theme) -> iced::Color {
        use super::tenshu::Tone;
        match self {
            // Info has no chip tone of its own; it borrows the focus blue.
            ToastTone::Info => theme.colors.shell.focus,
            ToastTone::Success => Tone::Good.color(theme),
            ToastTone::Warning => Tone::Warn.color(theme),
            ToastTone::Error => Tone::Bad.color(theme),
            ToastTone::Debug => Tone::Muted.color(theme),
        }
    }

    /// Warnings and errors wash their panel in the tone so they stand apart
    /// from confirmations (option D, Robert 2026-11-27).
    pub fn needs_attention(self) -> bool {
        matches!(self, ToastTone::Warning | ToastTone::Error)
    }
}

/// Share of the tone washed over the panel of a warning or error,
/// `color-mix(in srgb, var(--tone) 8%, var(--panel))` on the canvas.
pub const ATTENTION_WASH: f32 = 0.08;
/// Opacity of the tone behind the toast icon.
const TOAST_TILE_ALPHA: f32 = 0.13;

/// The panel fill of a toast: the plain panel, or for a warning or error the
/// panel washed [`ATTENTION_WASH`] in its tone.
pub fn toast_fill(theme: &Theme, tone: ToastTone) -> iced::Color {
    let panel = theme.colors.shell.panel;
    if tone.needs_attention() {
        super::tenshu::mix(panel, tone.color(theme), ATTENTION_WASH)
    } else {
        panel
    }
}

/// Toast surface (`/* cc-toasts v1 */`, option D): the menu/popover panel
/// (`tenshu::surface::panel`) with a 1px line border on every tone. A
/// warning or error washes its fill in its tone instead of a tone edge.
pub fn toast(theme: &Theme, tone: ToastTone) -> Style {
    let mut style = super::tenshu::surface::panel(theme);
    style.background = Some(Background::Color(toast_fill(theme, tone)));
    style.border.width = 1.0;
    style.border.color = theme.colors.shell.line;
    style
}

/// The 28px tile behind a toast's icon: the tone at low opacity.
pub fn toast_icon_tile(theme: &Theme, tone: ToastTone) -> Style {
    Style {
        text_color: Some(tone.color(theme)),
        background: Some(Background::Color(super::tenshu::faded(
            tone.color(theme),
            TOAST_TILE_ALPHA,
        ))),
        border: Border {
            radius: super::tenshu::radius::ROW.into(),
            ..Border::default()
        },
        ..Style::default()
    }
}

#[cfg(test)]
mod toast_tests {
    use super::{toast, toast_fill, ToastTone, ATTENTION_WASH};
    use crate::theme::tenshu::contrast;
    use crate::theme::Theme;
    use iced::Background;

    const TONES: [ToastTone; 5] = [
        ToastTone::Info,
        ToastTone::Success,
        ToastTone::Warning,
        ToastTone::Error,
        ToastTone::Debug,
    ];

    /// Every toast keeps the plain line border; no tone edge (option D).
    #[test]
    fn every_toast_keeps_the_line_border() {
        for theme in [Theme::dark(), Theme::light()] {
            for tone in TONES {
                let style = toast(&theme, tone);
                assert_eq!(style.border.width, 1.0, "{tone:?}");
                assert_eq!(style.border.color, theme.colors.shell.line, "{tone:?}");
            }
        }
    }

    #[test]
    fn log_levels_map_to_tones() {
        assert_eq!(ToastTone::from(log::Level::Error), ToastTone::Error);
        assert_eq!(ToastTone::from(log::Level::Warn), ToastTone::Warning);
        assert_eq!(ToastTone::from(log::Level::Info), ToastTone::Info);
        assert_eq!(ToastTone::from(log::Level::Debug), ToastTone::Debug);
        assert_eq!(ToastTone::from(log::Level::Trace), ToastTone::Debug);
    }

    /// Warnings and errors wash the panel 8% in their tone; the rest stay
    /// plain.
    #[test]
    fn only_warnings_and_errors_wash_the_panel() {
        for theme in [Theme::dark(), Theme::light()] {
            let panel = theme.colors.shell.panel;
            for tone in TONES {
                let Some(Background::Color(fill)) = toast(&theme, tone).background else {
                    panic!("{tone:?} has no flat fill");
                };
                assert_eq!(fill, toast_fill(&theme, tone), "{tone:?}");
                if tone.needs_attention() {
                    let wash = tone.color(&theme);
                    for (f, p, w) in [
                        (fill.r, panel.r, wash.r),
                        (fill.g, panel.g, wash.g),
                        (fill.b, panel.b, wash.b),
                    ] {
                        assert!(
                            (f - (p + (w - p) * ATTENTION_WASH)).abs() < 1e-5,
                            "{tone:?}"
                        );
                    }
                    assert_ne!(fill, panel, "{tone:?}");
                } else {
                    assert_eq!(fill, panel, "{tone:?}");
                }
            }
        }
    }

    /// The title (ink) and detail (muted) stay readable on the washed panel.
    #[test]
    fn copy_holds_contrast_on_the_washed_panel() {
        for theme in [Theme::dark(), Theme::light()] {
            for tone in [ToastTone::Warning, ToastTone::Error] {
                let fill = toast_fill(&theme, tone);
                let ink = contrast(theme.colors.shell.ink, fill);
                let muted = contrast(theme.colors.shell.muted, fill);
                assert!(ink >= 4.5, "{tone:?} title is {ink:.2}:1");
                assert!(muted >= 4.5, "{tone:?} detail is {muted:.2}:1");
                let icon = contrast(tone.color(&theme), fill);
                assert!(icon >= 3.0, "{tone:?} icon is {icon:.2}:1");
            }
        }
    }

    /// Icon glyphs are non-text graphics: WCAG asks 3:1 against the panel.
    #[test]
    fn tone_icons_hold_contrast_on_the_panel_in_both_themes() {
        for theme in [Theme::dark(), Theme::light()] {
            let panel = theme.colors.shell.panel;
            for tone in TONES {
                let ratio = contrast(tone.color(&theme), panel);
                assert!(ratio >= 3.0, "{tone:?} is {ratio:.2}:1 on the panel");
            }
            // The dismiss cross is drawn in `shell.muted` (Tenshu icon action).
            let cross = contrast(theme.colors.shell.muted, panel);
            assert!(cross >= 3.0, "dismiss cross is {cross:.2}:1 on the panel");
        }
    }
}
