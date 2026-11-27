//! The Overview's Total balance row (`.hero`): the balance block on the left
//! and the worldview's voice line on the far right, both standing on one
//! baseline. A narrower column keeps the row with a smaller line; one too
//! narrow for the row sets the line under the balance. The row draws no
//! artwork (Desktop V3: Bitcoin Worldview is wording only).
//!
//! The breakpoints are the width the row is given and what its parts need,
//! not the window's.

use iced::advanced::{layout, widget::Tree};
use iced::alignment::Horizontal;
use iced::widget::text::{Alignment as TextAlignment, LineHeight};
use iced::{Length, Size};

use super::arranged::arranged;
use crate::font;
use crate::theme::{self, Theme};
use crate::widget::{Column, Container, Element, Renderer};

/// `.hero { column-gap: 32px }`, between the balance and the worldview.
pub const COLUMN_GAP: f32 = 32.0;
/// Below this the row uses the narrow line.
pub const WIDE_MIN_WIDTH: f32 = 640.0;
/// Below this there is no row: the line sits under the balance.
pub const ROW_MIN_WIDTH: f32 = 520.0;
/// The least the balance column keeps beside the worldview: room for a
/// jumbo fiat total in the hundreds of thousands. The balance fills its
/// column (`minmax(0, 1fr)`), so the worldview, at its own width, gives way.
pub const BALANCE_MIN_WIDTH: f32 = 280.0;

/// `.line { max-width: 190px }`, `.say { font-size: 18px }`.
const LINE_MAX: f32 = 190.0;
const LINE_SIZE: f32 = 18.0;
/// `.narrow .line { max-width: 150px }`, `.narrow .say { font-size: 16px }`.
const NARROW_LINE_MAX: f32 = 150.0;
const NARROW_LINE_SIZE: f32 = 16.0;
/// `.say { line-height: 1.35 }`.
const LINE_HEIGHT: f32 = 1.35;
/// `.rule { width: 28px; height: 1px; margin-bottom: 10px }`.
const RULE_WIDTH: f32 = 28.0;
const RULE_GAP: f32 = 10.0;
/// `.line { padding-bottom: 2px }`: the line's last baseline on the SATS line.
const LINE_BOTTOM: f32 = 2.0;
/// The line's distance under the balance when stacked.
const STACK_GAP: f32 = 12.0;

/// How the row arranges itself at a given width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroLayout {
    /// The balance, then the line at the far right.
    Wide,
    /// The same with the narrower, smaller line.
    Narrow,
    /// The line under the balance.
    Stacked,
}

/// The arrangement for a row `width` wide, given the worldview line's width
/// in its wide and its narrow form.
pub fn hero_layout(width: f32, wide_side: f32, narrow_side: f32) -> HeroLayout {
    let fits = |side: f32| width - COLUMN_GAP - side >= BALANCE_MIN_WIDTH;
    if width >= WIDE_MIN_WIDTH && fits(wide_side) {
        HeroLayout::Wide
    } else if width >= ROW_MIN_WIDTH && fits(narrow_side) {
        HeroLayout::Narrow
    } else {
        HeroLayout::Stacked
    }
}

/// `balance` with the worldview's `line`, if the mode has one.
pub fn worldview_hero<'a, M: 'a>(
    balance: impl Into<Element<'a, M>>,
    line: Option<&'a str>,
) -> Element<'a, M> {
    let voice = |size: f32, max: f32, align: Horizontal| -> Element<'a, M> {
        match line {
            Some(line) => voice_line(line, size, max, align),
            None => iced::widget::Space::new().into(),
        }
    };
    let children = vec![
        balance.into(),
        voice(LINE_SIZE, LINE_MAX, Horizontal::Right),
        voice(NARROW_LINE_SIZE, NARROW_LINE_MAX, Horizontal::Right),
        voice(NARROW_LINE_SIZE, f32::INFINITY, Horizontal::Left),
    ];
    arranged(
        children,
        Size::new(Length::Fill, Length::Shrink),
        layout_hero,
    )
    .skip_empty()
    .into()
}

fn layout_hero<'a, M: 'a>(
    children: &mut [Element<'a, M>],
    trees: &mut [Tree],
    renderer: &Renderer,
    limits: &layout::Limits,
) -> layout::Node {
    let max = limits.max();
    let width = if max.width.is_finite() {
        max.width
    } else {
        WIDE_MIN_WIDTH
    };
    let free = layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY));
    let mut lay = |index: usize, limits: &layout::Limits| {
        children[index]
            .as_widget_mut()
            .layout(&mut trees[index], renderer, limits)
    };
    // The worldview at its own widths: the lines are no wider than their
    // maximum.
    let wide_line = lay(1, &free);
    let narrow_line = lay(2, &free);
    let hidden = || layout::Node::new(Size::ZERO);
    let (nodes, height) = match hero_layout(width, wide_line.size().width, narrow_line.size().width)
    {
        arrangement @ (HeroLayout::Wide | HeroLayout::Narrow) => {
            let line = if arrangement == HeroLayout::Wide {
                wide_line
            } else {
                narrow_line
            };
            let line_size = line.size();
            let column = if line_size.width > 0.0 {
                width - COLUMN_GAP - line_size.width
            } else {
                width
            };
            let balance = lay(
                0,
                &layout::Limits::new(Size::ZERO, Size::new(column, f32::INFINITY)),
            );
            let balance_size = balance.size();
            let height = if line_size.width > 0.0 {
                balance_size.height.max(line_size.height + LINE_BOTTOM)
            } else {
                balance_size.height
            };
            let balance = balance.move_to((0.0, height - balance_size.height));
            let line = line.move_to((
                width - line_size.width,
                height - LINE_BOTTOM - line_size.height,
            ));
            let nodes = if arrangement == HeroLayout::Wide {
                vec![balance, line, hidden(), hidden()]
            } else {
                vec![balance, hidden(), line, hidden()]
            };
            (nodes, height)
        }
        HeroLayout::Stacked => {
            let balance = lay(0, &free);
            let stacked = lay(3, &free);
            let balance_height = balance.size().height;
            let (stacked, height) = if stacked.size().width > 0.0 {
                let height = stacked.size().height;
                (
                    stacked.move_to((0.0, balance_height + STACK_GAP)),
                    balance_height + STACK_GAP + height,
                )
            } else {
                (hidden(), balance_height)
            };
            (vec![balance, hidden(), hidden(), stacked], height)
        }
    };
    layout::Node::with_children(Size::new(width, height), nodes)
}

/// The colour of the 28px rule over the voice line: muted ink in every
/// worldview and appearance. The modes have no accent colours (Desktop V3:
/// Bitcoin Worldview is wording only), so a mode switch changes the line's
/// words and never its colour.
pub fn voice_rule_color(theme: &Theme) -> iced::Color {
    theme.colors.shell.muted
}

/// `.line`: the 28px rule over the italic voice line, both aligned to
/// `align`.
fn voice_line<'a, M: 'a>(line: &'a str, size: f32, max: f32, align: Horizontal) -> Element<'a, M> {
    let rule = Container::new(iced::widget::Space::new())
        .width(Length::Fixed(RULE_WIDTH))
        .height(Length::Fixed(1.0))
        .style(|theme: &Theme| iced::widget::container::Style {
            background: Some(voice_rule_color(theme).into()),
            ..Default::default()
        });
    let text_align = match align {
        Horizontal::Right => TextAlignment::Right,
        Horizontal::Center => TextAlignment::Center,
        Horizontal::Left => TextAlignment::Left,
    };
    Column::new()
        .spacing(RULE_GAP)
        .max_width(max)
        .align_x(align)
        .push(rule)
        .push(
            iced::widget::text(line)
                .shaping(iced::advanced::text::Shaping::Advanced)
                .size(size)
                .line_height(LineHeight::Relative(LINE_HEIGHT))
                .font(font::ITALIC)
                .align_x(text_align)
                .style(theme::tenshu::text::ink),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_row_narrows_then_stacks_as_its_column_shrinks() {
        let (wide, narrow) = (190.0, 150.0);
        assert_eq!(hero_layout(840.0, wide, narrow), HeroLayout::Wide);
        assert_eq!(hero_layout(620.0, wide, narrow), HeroLayout::Narrow);
        assert_eq!(hero_layout(520.0, wide, narrow), HeroLayout::Narrow);
        assert_eq!(hero_layout(500.0, wide, narrow), HeroLayout::Stacked);
        assert_eq!(hero_layout(420.0, wide, narrow), HeroLayout::Stacked);
        // A mode with no line keeps the row at any width it can hold.
        assert_eq!(hero_layout(640.0, 0.0, 0.0), HeroLayout::Wide);
        assert_eq!(hero_layout(520.0, 0.0, 0.0), HeroLayout::Narrow);
    }

    /// Desktop V3 (V7): the rule over the voice line is the same muted ink
    /// in all three worldviews, in both appearances.
    #[test]
    fn the_voice_rule_keeps_one_colour_in_every_worldview() {
        use crate::theme::WorldviewMode;
        for base in [Theme::dark(), Theme::light()] {
            for mode in [
                WorldviewMode::Samurai,
                WorldviewMode::Suitcoiner,
                WorldviewMode::Luke,
            ] {
                assert_eq!(
                    voice_rule_color(&base.with_worldview(mode)),
                    base.colors.shell.muted,
                    "{mode:?}"
                );
            }
        }
    }
}
