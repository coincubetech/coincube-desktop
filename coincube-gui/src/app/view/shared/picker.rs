use coincube_ui::component::text::Text;
use coincube_ui::widget::{Column as UiColumn, Container as UiContainer, Row as UiRow};
use coincube_ui::{color, component::text::*, icon, theme, widget::*};
use iced::{
    widget::{button as iced_button, container, Column, Container, Row, Space},
    Alignment, Background, Length,
};

/// A single row in a picker modal — an asset or wallet icon, a label with an optional
/// balance subtitle, a right-aligned network badge, and a selection checkmark.
///
/// Reused across Liquid Send's asset pickers and the Transfer flow's wallet picker.
pub fn picker_row<'a, M>(
    ico: impl Into<Element<'a, M>>,
    label: &str,
    balance: &str,
    network: &str,
    is_selected: bool,
    on_press: M,
) -> Element<'a, M>
where
    M: Clone + 'a,
{
    let mut row = Row::new()
        .spacing(12)
        .align_y(Alignment::Center)
        .push(ico)
        .push(
            Column::new()
                .spacing(2)
                .push(
                    text(label.to_string())
                        .size(P1_SIZE)
                        .bold()
                        .style(theme::text::primary),
                )
                .push_maybe(if !balance.is_empty() {
                    Some(
                        text(balance.to_string())
                            .size(P2_SIZE)
                            .style(theme::text::secondary),
                    )
                } else {
                    None
                }),
        )
        .push(
            Container::new(text(network.to_uppercase()).size(10).color(color::ORANGE))
                .padding([2, 6])
                .style(|_: &theme::Theme| container::Style {
                    border: iced::Border {
                        color: color::ORANGE,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }),
        )
        .push(Space::new().width(Length::Fill));

    if is_selected {
        row = row.push(icon::check2_icon().size(18).color(color::ORANGE));
    }

    iced_button(
        Container::new(row)
            .padding([12, 16])
            .width(Length::Fill)
            .style(if is_selected {
                picker_row_selected
            } else {
                picker_row_idle
            }),
    )
    .on_press(on_press)
    .style(|_: &theme::Theme, _| iced_button::Style {
        background: Some(Background::Color(color::TRANSPARENT)),
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

/// An unselected picker row: the simple card surface with the square 8px
/// corner every button takes since canvas v122.
pub fn picker_row_idle(theme: &theme::Theme) -> container::Style {
    let mut style = theme::card::simple(theme);
    style.border.radius = 8.0.into();
    style
}

/// Selected row in picker modals — orange border with subtle tinted background.
pub fn picker_row_selected(theme: &theme::Theme) -> container::Style {
    let bg = match theme.mode {
        coincube_ui::theme::palette::ThemeMode::Dark => iced::color!(0x1a1a10),
        coincube_ui::theme::palette::ThemeMode::Light => iced::color!(0xFFF5E6),
    };
    container::Style {
        background: Some(Background::Color(bg)),
        border: iced::Border {
            color: color::ORANGE,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

// ── Route dropdowns ─────────────────────────────────────────────────────────
//
// Spark Send and Receive pick each side of a route from a menu anchored under
// its tile, not a modal: the form stays in view, and the menu is the tile's
// own width, on the tile's own surface.

/// The gap between a route tile and the menu under it.
pub const DROPDOWN_GAP: f32 = 6.0;
/// The least a dropdown keeps of a short window before it scrolls.
pub const DROPDOWN_MIN_HEIGHT: f32 = 220.0;
/// Logo size in a dropdown row.
pub const DROPDOWN_LOGO: f32 = 28.0;

/// A route tile with its menu under it while `menu` is set. The menu floats
/// over what follows (it takes no room), is as wide as the tile, scrolls when
/// the window is too short for it, and `on_dismiss` closes it on a press
/// outside it or Escape.
pub fn anchored_dropdown<'a, M>(
    tile: impl Into<Element<'a, M>>,
    menu: Option<Element<'a, M>>,
    on_dismiss: M,
) -> Element<'a, M>
where
    M: Clone + 'a,
{
    anchored_dropdown_min(tile, menu, on_dismiss, 0.0)
}

/// [`anchored_dropdown`] whose menu is at least `min_width` wide, as the
/// boards' `.nmenu { min-width }` sets it, while staying inside the window.
pub fn anchored_dropdown_min<'a, M>(
    tile: impl Into<Element<'a, M>>,
    menu: Option<Element<'a, M>>,
    on_dismiss: M,
    min_width: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
{
    let column = UiColumn::new().width(Length::Fill).push(tile);
    match menu {
        Some(menu) => column
            .push(
                coincube_ui::widget::floating::Floating::new(menu)
                    .gap(DROPDOWN_GAP)
                    .on_dismiss(on_dismiss)
                    .fit_height_to_window(DROPDOWN_MIN_HEIGHT)
                    .min_width(min_width),
            )
            .into(),
        None => column.into(),
    }
}

/// P-Receive's THEY SEND menu minimum (`.ndd:first-child .nmenu`): wide
/// enough for USDt's five network chips on one line.
pub const THEY_SEND_MENU_MIN: f32 = 440.0;
/// P-Receive's YOU RECEIVE menu minimum (`.ndd:last-child .nmenu`).
pub const YOU_RECEIVE_MENU_MIN: f32 = 400.0;
/// P-Send's menu minimum (`.rx .ndd .nmenu`), for both of Send's menus.
pub const SEND_MENU_MIN: f32 = 380.0;

/// The menu's surface: the panel fill and a rule line, 12px corners and a
/// shadow to lift it off what it opens over. `content` scrolls when the window caps the
/// menu's height.
pub fn dropdown_menu<'a, M>(content: UiColumn<'a, M>) -> Element<'a, M>
where
    M: Clone + 'a,
{
    UiContainer::new(
        // The scrollable clips what it draws, so the rows keep 4px clear on
        // every side for the focus ring (it paints 4px outside a row), and 4px
        // between them so a ring never covers the next row. The extra room on
        // the right is the scrollbar's, so it never draws over a row's check.
        iced::widget::scrollable(content.spacing(4).padding(iced::Padding {
            top: 4.0,
            bottom: 4.0,
            left: 4.0,
            right: 8.0,
        }))
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new()
                .width(4)
                .scroller_width(4),
        ))
        .height(Length::Shrink),
    )
    .padding(2)
    .width(Length::Fill)
    .style(dropdown_menu_style)
    .into()
}

/// A dropdown's floating list (`.nmenu`): the panel fill, 12px corners, a
/// 1px rule line and a shadow. Send's and Receive's route tiles share the
/// panel fill, and Transfer's tiles (the page fill, with a line) sit on a
/// panel card, so the menu keeps the line beside the shadow (R4: a shadow
/// alone does not separate same-fill boxes).
pub fn dropdown_menu_style(theme: &theme::Theme) -> container::Style {
    let shell = &theme.colors.shell;
    container::Style {
        background: Some(Background::Color(shell.panel)),
        border: iced::Border {
            color: shell.rule,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: iced::Shadow {
            color: shell.shadow,
            offset: iced::Vector::new(0.0, 16.0),
            blur_radius: 40.0,
        },
        ..Default::default()
    }
}

/// A group heading inside a dropdown ("Bitcoin", "Stablecoin (cross-chain)").
pub fn dropdown_heading<'a, M: 'a>(label: &str) -> Element<'a, M> {
    UiContainer::new(
        text(label.to_uppercase())
            .size(CAPTION_SIZE)
            .bold()
            .style(theme::text::secondary),
    )
    .padding(iced::Padding {
        top: 8.0,
        bottom: 4.0,
        left: 10.0,
        right: 10.0,
    })
    .into()
}

/// A note at the foot of a dropdown, under a rule.
pub fn dropdown_note<'a, M: 'a>(note: &str) -> Element<'a, M> {
    UiColumn::new()
        .spacing(8)
        .push(Space::new().height(Length::Fixed(4.0)))
        .push(
            iced::widget::rule::horizontal(1).style(|theme: &theme::Theme| {
                iced::widget::rule::Style {
                    color: theme.colors.shell.line,
                    radius: 0.0.into(),
                    fill_mode: iced::widget::rule::FillMode::Full,
                    snap: true,
                }
            }),
        )
        .push(
            UiContainer::new(
                text(note.to_string())
                    .size(CAPTION_SIZE + 1)
                    .style(theme::text::secondary),
            )
            .padding([2, 10]),
        )
        .into()
}

/// One option in a dropdown: logo, name, network chip, an optional line
/// under it, and a check when selected. `on_press: None` shows it disabled.
#[allow(clippy::too_many_arguments)]
pub fn dropdown_option<'a, M>(
    id: impl Into<iced::advanced::widget::Id>,
    logo: Element<'a, M>,
    title: &str,
    chip: &str,
    sub: Option<String>,
    selected: bool,
    on_press: Option<M>,
    focus_intent: impl FnOnce(iced::advanced::widget::Id) -> M,
) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let enabled = on_press.is_some();
    let ink = move |theme: &theme::Theme| iced::widget::text::Style {
        color: Some(if enabled {
            theme.colors.shell.ink
        } else {
            theme.colors.shell.muted
        }),
    };
    let chip_color = move |theme: &theme::Theme| {
        if enabled {
            theme.colors.shell.accent_ink
        } else {
            theme.colors.shell.muted
        }
    };
    let mut lines = UiColumn::new().spacing(1).width(Length::Fill).push(
        UiRow::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(text(title.to_string()).size(P1_SIZE).bold().style(ink))
            .push(
                UiContainer::new(text(chip.to_uppercase()).size(10).style(
                    move |theme: &theme::Theme| iced::widget::text::Style {
                        color: Some(chip_color(theme)),
                    },
                ))
                .padding([1, 5])
                .style(move |theme: &theme::Theme| container::Style {
                    border: iced::Border {
                        color: chip_color(theme),
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                }),
            ),
    );
    if let Some(sub) = sub {
        lines = lines.push(
            text(sub)
                .size(CAPTION_SIZE + 1)
                .style(theme::text::secondary)
                .wrapping(iced::widget::text::Wrapping::None),
        );
    }
    let row = UiRow::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(logo)
        .push(lines)
        .push(selected_tick(selected));
    coincube_ui::widget::action::Action::new(
        id.into(),
        UiContainer::new(row)
            .padding([6, 10])
            .width(Length::Fill)
            .height(Length::Shrink),
        focus_intent,
    )
    .padding(0)
    .width(Length::Fill)
    .focus_radius(6.0)
    .style(move |theme: &theme::Theme, status| dropdown_option_style(theme, status, selected))
    .on_press_maybe(on_press)
    .into()
}

fn dropdown_option_style(
    theme: &theme::Theme,
    status: iced_button::Status,
    selected: bool,
) -> iced_button::Style {
    let background = match (selected, status) {
        (_, iced_button::Status::Disabled) => color::TRANSPARENT,
        (true, _) => selected_row_fill(theme),
        (false, iced_button::Status::Hovered | iced_button::Status::Pressed) => {
            theme.colors.shell.hover
        }
        (false, _) => color::TRANSPARENT,
    };
    iced_button::Style {
        background: Some(Background::Color(background)),
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The selected row's fill: ink at 7% (`.nrow.sel`), neutral rather than the
/// accent-tinted `selected`, so a menu never reads as orange.
pub fn selected_row_fill(theme: &theme::Theme) -> iced::Color {
    let ink = theme.colors.shell.ink;
    iced::Color { a: 0.07, ..ink }
}

/// The selected row's tick, in ink. An unselected row keeps the same room so
/// the rows' text lines up.
fn selected_tick<'a, M: 'a>(selected: bool) -> Element<'a, M> {
    UiContainer::new(if selected {
        Element::from(icon::check2_icon().size(16).style(|theme: &theme::Theme| {
            iced::widget::text::Style {
                color: Some(theme.colors.shell.ink),
            }
        }))
    } else {
        Space::new().into()
    })
    .width(Length::Fixed(18.0))
    .into()
}

// ── Route lists (P-Receive, P-Send, MV-*-Proposed) ─────────────────────────
//
// Send's and Receive's route menus lead with what differs: a group heading in
// sentence case, then one row per method or coin with a small glyph or coin,
// its name, an optional hint, and a tick on the selected row. Stablecoins on
// Receive are one row per coin with its networks as chips.

/// The glyph or coin size in a route list row (`.ni2`).
pub const ROUTE_ICON: f32 = 20.0;

/// A route list's group heading (`.ngh`): "Bitcoin", or "Stablecoins" with a
/// `note` after a middle dot ("· Swapped to bitcoin by SideShift").
pub fn route_group<'a, M: 'a>(label: &str, note: Option<&str>) -> Element<'a, M> {
    let muted = |theme: &theme::Theme| iced::widget::text::Style {
        color: Some(theme.colors.shell.muted),
    };
    let mut row = UiRow::new().spacing(8).align_y(Alignment::Center).push(
        iced::widget::text(label.to_string())
            .font(coincube_ui::font::MEDIUM)
            .size(13)
            .style(muted),
    );
    if let Some(note) = note {
        row = row.push(
            iced::widget::text(format!("· {}", note))
                .size(13)
                .style(muted),
        );
    }
    UiContainer::new(row)
        .padding(iced::Padding {
            top: 8.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .into()
}

/// A method glyph for a route list row, in the muted ink (`.ni2` svg).
pub fn route_glyph<'a, M: 'a>(glyph: coincube_ui::widget::Text<'a>) -> Element<'a, M> {
    UiContainer::new(
        glyph
            .size(16)
            .style(|theme: &theme::Theme| iced::widget::text::Style {
                color: Some(theme.colors.shell.muted),
            }),
    )
    .width(Length::Fixed(ROUTE_ICON))
    .height(Length::Fixed(ROUTE_ICON))
    .center_x(Length::Fixed(ROUTE_ICON))
    .center_y(Length::Fixed(ROUTE_ICON))
    .into()
}

/// A coin for a route list row, at the row's icon size.
pub fn route_coin<'a, M: 'a>(asset: &str) -> Element<'a, M> {
    coincube_ui::image::asset_logo(asset)
        .width(Length::Fixed(ROUTE_ICON))
        .height(Length::Fixed(ROUTE_ICON))
        .into()
}

/// One row of a route list (`.nrow`): icon, name, an optional muted hint
/// after it, an optional right-aligned figure (a balance), and the tick when
/// selected. `on_press: None` shows it disabled and leaves the Tab order.
#[allow(clippy::too_many_arguments)]
pub fn route_option<'a, M>(
    id: impl Into<iced::advanced::widget::Id>,
    icon: Element<'a, M>,
    title: &str,
    hint: Option<String>,
    figure: Option<String>,
    selected: bool,
    on_press: Option<M>,
    focus_intent: impl FnOnce(iced::advanced::widget::Id) -> M,
) -> Element<'a, M>
where
    M: Clone + 'static,
{
    let enabled = on_press.is_some();
    let ink = move |theme: &theme::Theme| iced::widget::text::Style {
        color: Some(if enabled {
            theme.colors.shell.ink
        } else {
            theme.colors.shell.muted
        }),
    };
    let muted = |theme: &theme::Theme| iced::widget::text::Style {
        color: Some(theme.colors.shell.muted),
    };
    // The hint follows the name on its line; when the menu is too narrow for
    // both, the whole hint moves under the name rather than breaking beside it.
    let words = UiRow::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(
            iced::widget::text(title.to_string())
                .font(coincube_ui::font::MEDIUM)
                .size(P2_SIZE)
                .style(ink),
        )
        .push_maybe(hint.map(|hint| iced::widget::text(hint).size(13).style(muted)));
    let row = UiRow::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(icon)
        .push(UiContainer::new(words.wrap().vertical_spacing(2)).width(Length::Fill))
        .push_maybe(figure.map(|figure| iced::widget::text(figure).size(13).style(muted)))
        .push(selected_tick(selected));
    coincube_ui::widget::action::Action::new(
        id.into(),
        UiContainer::new(row)
            .padding([6, 10])
            .width(Length::Fill)
            .height(Length::Shrink)
            .center_y(Length::Shrink),
        focus_intent,
    )
    .padding(0)
    .width(Length::Fill)
    .focus_radius(8.0)
    .style(move |theme: &theme::Theme, status| route_option_style(theme, status, selected))
    .on_press_maybe(on_press)
    .into()
}

/// A route list row: no fill at rest, a light ink fill on hover, and the
/// neutral selected fill while selected. Never an accent.
pub fn route_option_style(
    theme: &theme::Theme,
    status: iced_button::Status,
    selected: bool,
) -> iced_button::Style {
    let background = match (selected, status) {
        (_, iced_button::Status::Disabled) => color::TRANSPARENT,
        (true, _) => selected_row_fill(theme),
        (false, iced_button::Status::Hovered | iced_button::Status::Pressed) => {
            theme.colors.shell.hover
        }
        (false, _) => color::TRANSPARENT,
    };
    iced_button::Style {
        background: Some(Background::Color(background)),
        text_color: theme.colors.shell.ink,
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// A coin's row with its networks as chips (`.nrow.chips`): the row itself
/// is not a button; each chip is. The chips sit at the row's end and wrap
/// under each other when the menu is narrow.
pub fn route_chip_row<'a, M: 'a>(
    icon: Element<'a, M>,
    title: &str,
    chips: Vec<Element<'a, M>>,
) -> Element<'a, M> {
    let mut row = UiRow::new().spacing(5);
    for chip in chips {
        row = row.push(chip);
    }
    UiContainer::new(
        UiRow::new()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(icon)
            .push(
                iced::widget::text(title.to_string())
                    .font(coincube_ui::font::MEDIUM)
                    .size(P2_SIZE)
                    .style(|theme: &theme::Theme| iced::widget::text::Style {
                        color: Some(theme.colors.shell.ink),
                    }),
            )
            .push(
                UiContainer::new(
                    row.wrap()
                        .vertical_spacing(5)
                        .align_x(iced::alignment::Horizontal::Right),
                )
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right),
            ),
    )
    .padding([6, 10])
    .width(Length::Fill)
    .into()
}

/// A network chip in a coin's row (`.nch`). It is a content button, so it is
/// outlined in ink (the button rule) and reads as clickable; hover adds the
/// light fill. The picked chip is filled ink with the page colour for text.
pub fn route_chip<'a, M>(
    id: impl Into<iced::advanced::widget::Id>,
    label: &str,
    selected: bool,
    on_press: M,
    focus_intent: impl FnOnce(iced::advanced::widget::Id) -> M,
) -> Element<'a, M>
where
    M: Clone + 'static,
{
    coincube_ui::widget::action::Action::new(
        id.into(),
        UiContainer::new(iced::widget::text(label.to_string()).size(13))
            .height(Length::Fixed(26.0))
            .center_y(Length::Fixed(26.0)),
        focus_intent,
    )
    .padding([0, 8])
    .focus_radius(8.0)
    .style(move |theme: &theme::Theme, status| route_chip_style(theme, status, selected))
    .on_press(on_press)
    .into()
}

/// The chip's look: 1px ink outline, radius 8, ink text; hover adds the
/// light fill; the picked chip is filled ink.
pub fn route_chip_style(
    theme: &theme::Theme,
    status: iced_button::Status,
    selected: bool,
) -> iced_button::Style {
    let shell = &theme.colors.shell;
    let background = if selected {
        Some(Background::Color(shell.ink))
    } else if matches!(
        status,
        iced_button::Status::Hovered | iced_button::Status::Pressed
    ) {
        Some(Background::Color(shell.hover))
    } else {
        None
    };
    iced_button::Style {
        background,
        text_color: if selected {
            shell.background
        } else {
            shell.ink
        },
        border: iced::Border {
            color: shell.ink,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// A note at the foot of a route list, under a rule (`.nfoot`).
pub fn route_note<'a, M: 'a>(note: &str) -> Element<'a, M> {
    dropdown_note(note)
}

#[cfg(test)]
mod menu_style_tests {
    use super::*;

    /// R4: the dropdown shares the route tiles' panel fill, so it keeps its
    /// line as well as its shadow.
    #[test]
    fn the_dropdown_keeps_its_border_over_same_fill_tiles() {
        for theme in [theme::Theme::dark(), theme::Theme::light()] {
            let style = dropdown_menu_style(&theme);
            assert_eq!(
                style.background,
                Some(Background::Color(theme.colors.shell.panel))
            );
            assert_eq!(style.border.width, 1.0);
            assert_eq!(style.border.color, theme.colors.shell.rule);
            assert!(style.shadow.color.a > 0.0 && style.shadow.blur_radius > 0.0);
        }
    }
}
