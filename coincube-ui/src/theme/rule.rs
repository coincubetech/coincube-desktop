use iced::widget::rule::{Catalog, FillMode, Style, StyleFn};

use super::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The default styling of a [`Rule`].
pub fn default(theme: &Theme) -> Style {
    Style {
        color: theme.colors.rule,
        radius: 0.0.into(),
        fill_mode: FillMode::Full,
        snap: true,
    }
}

/// The quiet region divider: the shell's `rule` token (ink at 10%), under the
/// top bar, beside the sidebar and under the pinned page head.
pub fn quiet(theme: &Theme) -> Style {
    Style {
        color: theme.colors.shell.rule,
        ..default(theme)
    }
}
