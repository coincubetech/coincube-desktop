//! A Cube's mark in the style its owner chose.
//!
//! Every place that shows a Cube calls [`cube_mark`] (or [`cube_mark_muted`]
//! for a Cube on another computer), never one style's widget directly, so
//! the owner's choice shows everywhere (plan invariant D-R3). The three
//! styles share one seed and one generator ([`ring_mark::seed`]), and one
//! fallback: with no usable seed, the ring mark's centre dot alone,
//! whatever the style.
//!
//! The style is `markStyle` on the Cube (master plan decision D8): 0 Rings,
//! the default, 1 Crest, 2 Grid. An app that meets a number it does not
//! know draws Rings ([`CubeMarkStyle::from_u8`]).

use std::cell::RefCell;
use std::collections::HashMap;

use iced::widget::svg as iced_svg;
use iced::{Color, Length};

use super::{crest_mark, grid_mark, ring_mark};
use crate::theme::Theme;
use crate::widget::Element;

/// Which drawing rule a Cube's seed goes through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CubeMarkStyle {
    /// Arcs around a dot, one orange (`SPEC-cube-ring-mark-v1`). The default.
    #[default]
    Rings,
    /// Petals with turning symmetry in a faint ring (`SPEC-cube-crest-mark-v1`).
    Crest,
    /// A mirrored five-by-five grid of dots (`SPEC-cube-grid-mark-v1`).
    Grid,
}

impl CubeMarkStyle {
    /// Every style, in the order the picker shows them.
    pub const ALL: [Self; 3] = [Self::Rings, Self::Crest, Self::Grid];

    /// The style for a `markStyle` number. Anything but 1 or 2 is Rings, so
    /// a style a newer app added draws as the default rather than nothing.
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Crest,
            2 => Self::Grid,
            _ => Self::Rings,
        }
    }

    /// The `markStyle` number for this style.
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Rings => 0,
            Self::Crest => 1,
            Self::Grid => 2,
        }
    }

    /// The style's name as the picker labels it, in English; callers pass
    /// it through the app's localisation.
    pub fn name(self) -> &'static str {
        match self {
            Self::Rings => "Rings",
            Self::Crest => "Crest",
            Self::Grid => "Grid",
        }
    }
}

/// A Cube's mark at `size` px square, in the style given and the theme's
/// colours. `seed` comes from [`ring_mark::seed`]; with `None`, or anything
/// [`ring_mark::is_seed`] refuses, only the centre dot is drawn.
///
/// The mark is decoration beside the Cube's name: always show the name with
/// it (master plan invariant RM6).
pub fn cube_mark<'a, M: 'a>(style: CubeMarkStyle, seed: Option<&str>, size: f32) -> Element<'a, M> {
    draw(style, seed, size, false)
}

/// A Cube's mark with every part in the muted colour, for a Cube that lives
/// on another computer.
pub fn cube_mark_muted<'a, M: 'a>(
    style: CubeMarkStyle,
    seed: Option<&str>,
    size: f32,
) -> Element<'a, M> {
    draw(style, seed, size, true)
}

fn draw<'a, M: 'a>(
    style: CubeMarkStyle,
    seed: Option<&str>,
    size: f32,
    muted: bool,
) -> Element<'a, M> {
    let seed = seed.filter(|seed| ring_mark::is_seed(seed));
    match (style, seed) {
        // Rings, and the shared fallback dot, are the ring mark's own.
        (CubeMarkStyle::Rings, _) | (_, None) => {
            if muted {
                ring_mark::ring_mark_muted(seed, size)
            } else {
                ring_mark::ring_mark(seed, size)
            }
        }
        (style, Some(seed)) => {
            let side = Length::Fixed(ring_mark::mark_size(size));
            iced_svg(layer(style, seed))
                .width(side)
                .height(side)
                .style(move |theme: &Theme, _| iced_svg::Style {
                    color: Some(if muted {
                        theme.colors.shell.muted
                    } else {
                        theme.colors.shell.ink
                    }),
                })
                .into()
        }
    }
}

/// Keeps the one-colour layers for the Crest and Grid marks on screen.
/// Bounded as the ring mark's cache is.
const LAYER_CACHE_LIMIT: usize = 256;

thread_local! {
    static LAYER_CACHE: RefCell<HashMap<(CubeMarkStyle, String), iced_svg::Handle>> =
        RefCell::new(HashMap::new());
}

/// The mark drawn in black, which the theme's tint replaces at draw time;
/// the SVG's own opacities (the crest's ring, the grid's small dots) stay.
/// `seed` has passed [`ring_mark::is_seed`].
fn layer(style: CubeMarkStyle, seed: &str) -> iced_svg::Handle {
    let key = (style, seed.to_owned());
    LAYER_CACHE.with(|cache| {
        if let Some(handle) = cache.borrow().get(&key) {
            return handle.clone();
        }
        let handle = iced_svg::Handle::from_memory(
            svg(style, Some(seed), Color::BLACK, Color::BLACK).into_bytes(),
        );
        let mut cache = cache.borrow_mut();
        if cache.len() >= LAYER_CACHE_LIMIT {
            cache.clear();
        }
        cache.insert(key, handle.clone());
        handle
    })
}

/// The mark's shapes on the 64 box in the style given, without the `<svg>`
/// wrapper, so a caller building a larger SVG (the Launcher's faces) can
/// place it in a `<g>`. `accent` is used by Rings only. With no seed, or one
/// [`ring_mark::is_seed`] refuses, the centre dot alone.
pub fn svg_body(style: CubeMarkStyle, seed: Option<&str>, ink: Color, accent: Color) -> String {
    match (style, seed.filter(|seed| ring_mark::is_seed(seed))) {
        (CubeMarkStyle::Crest, Some(seed)) => {
            crest_mark::svg_body(&crest_mark::CrestMark::from_seed(seed), ink)
        }
        (CubeMarkStyle::Grid, Some(seed)) => {
            grid_mark::svg_body(&grid_mark::GridMark::from_seed(seed), ink)
        }
        // Rings, or no seed in any style: the ring mark (its dot alone when
        // there is no seed).
        (_, seed) => {
            let mark = seed.map(ring_mark::RingMark::from_seed);
            ring_mark::svg_body(mark.as_ref(), ink, accent)
        }
    }
}

/// A complete SVG document for the mark on the 64 box. See [`svg_body`].
pub fn svg(style: CubeMarkStyle, seed: Option<&str>, ink: Color, accent: Color) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">{}</svg>"#,
        svg_body(style, seed, ink, accent)
    )
}

#[cfg(test)]
mod tests;
