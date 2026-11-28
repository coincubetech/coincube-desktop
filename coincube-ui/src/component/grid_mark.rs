//! Cube grid marks, version 1.
//!
//! The Grid style (`markStyle` 2): a left-right mirrored pattern on a
//! five-by-five grid of dots, each either large and solid or small and
//! faint. No ring and no orange.
//!
//! The rules are `SPEC-cube-grid-mark-v1.md` in the company brain
//! (`plans/cube-avatars/`). The seed and the generator are the ring mark's
//! ([`super::ring_mark::seed`], [`super::ring_mark::Generator`]), so
//! switching style never changes a Cube's variation number. Every step is on
//! integers. Version 1 is frozen once shipped. The spec's fixture is copied
//! unchanged into `grid_mark/testdata/` and asserted in the tests.

use std::fmt::Write as _;

use iced::Color;

use super::ring_mark::{self, Generator};

/// The spec version this module draws.
pub const VERSION: u32 = 1;

/// Rows and columns.
pub const SIDE: usize = 5;

const BOX: f64 = 64.0;
const ON_RADIUS: f64 = 3.2;
const OFF_RADIUS: f64 = 1.1;
/// A draw above this turns a dot on (`next() > 50000`).
const ON_ABOVE: u32 = 50_000;

/// A Cube's grid: `rows[y][x]` is whether the dot at row `y` (from the top)
/// and column `x` (from the left) is large.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridMark {
    pub rows: [[bool; SIDE]; SIDE],
}

impl GridMark {
    /// The grid for a seed made by [`ring_mark::seed`]: exactly fifteen
    /// draws, row by row from the top, columns 0, 1, 2; columns 3 and 4
    /// mirror 1 and 0 (spec section 5).
    pub fn from_seed(seed: &str) -> Self {
        let mut generator = Generator::new(seed);
        let mut rows = [[false; SIDE]; SIDE];
        for row in &mut rows {
            for cell in row.iter_mut().take(3) {
                *cell = generator.next_draw() > ON_ABOVE;
            }
            row[3] = row[1];
            row[4] = row[0];
        }
        Self { rows }
    }

    /// Row `y` as the fixture writes it: five characters, `1` for a large
    /// dot, left to right.
    pub fn row_string(&self, y: usize) -> String {
        self.rows[y]
            .iter()
            .map(|on| if *on { '1' } else { '0' })
            .collect()
    }
}

/// The grid's 25 dots on the 64 box, without the `<svg>` wrapper (spec
/// section 6), all in `ink`. Within a row the dots go in the study's order:
/// each column from the left, followed by its mirror.
pub fn svg_body(mark: &GridMark, ink: Color) -> String {
    let ink = ring_mark::hex(ink);
    let mut body = String::new();
    for (y, row) in mark.rows.iter().enumerate() {
        for x in 0..3 {
            let mirror = SIDE - 1 - x;
            for column in std::iter::once(x).chain((mirror != x).then_some(mirror)) {
                let (radius, opacity) = if row[column] {
                    (ON_RADIUS, "1")
                } else {
                    (OFF_RADIUS, "0.3")
                };
                let _ = write!(
                    body,
                    r#"<circle cx="{}" cy="{}" r="{radius}" fill="{ink}" opacity="{opacity}"/>"#,
                    14 + 9 * column,
                    14 + 9 * y,
                );
            }
        }
    }
    body
}

/// A complete SVG document for the grid on the 64 box, in `ink`. Callers
/// pass the theme's token (`theme.colors.shell.ink`), never a literal.
pub fn svg(mark: &GridMark, ink: Color) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {BOX} {BOX}">{}</svg>"#,
        svg_body(mark, ink)
    )
}

#[cfg(test)]
mod tests;
