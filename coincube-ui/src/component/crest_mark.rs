//! Cube crest marks, version 1.
//!
//! The Crest style (`markStyle` 1): three to six identical strokes
//! ("petals") spaced evenly around the centre, inside a faint ring,
//! sometimes with a dot in the middle. Each petal is a straight line, a
//! curve or a bent line, the same shape for every petal of a Cube.
//!
//! The rules are `SPEC-cube-crest-mark-v1.md` in the company brain
//! (`plans/cube-avatars/`). The seed and the generator are the ring mark's
//! ([`super::ring_mark::seed`], [`super::ring_mark::Generator`]), so
//! switching style never changes a Cube's variation number. Version 1 is
//! frozen once shipped. The spec's fixture is copied unchanged into
//! `crest_mark/testdata/` and asserted in the tests.
//!
//! The faint ring is the same ring a person's initials sit in; Robert
//! accepted that on 2026-11-27 (master plan invariant RM7 notes the
//! exception, for Crest only).

use std::f64::consts::PI;
use std::fmt::Write as _;

use iced::Color;

use super::ring_mark::{self, Generator};

/// The spec version this module draws.
pub const VERSION: u32 = 1;

const BOX: f64 = 64.0;
const CENTRE: f64 = 32.0;
const STROKE: f64 = 2.4;
const RING_RADIUS: f64 = 29.0;
const DOT_RADIUS: f64 = 3.0;
/// Each draw is `0..DRAW_RANGE`.
const DRAW_RANGE: u32 = 100_000;

/// What every petal of a Cube is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetalShape {
    /// A straight line from the inner to the outer point.
    Line,
    /// A quadratic curve, its control point at the outer radius.
    Curve,
    /// Two lines: inner point, outer point, then a third point.
    Bent,
}

impl PetalShape {
    /// The spec's number for the shape: 0 line, 1 curve, 2 bent line.
    pub fn index(self) -> u32 {
        match self {
            Self::Line => 0,
            Self::Curve => 1,
            Self::Bent => 2,
        }
    }
}

/// One petal's angles, in radians from the positive x-axis, increasing
/// clockwise on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Petal {
    /// The inner point's angle, `a_i`.
    pub angle: f64,
    /// The outer point's angle, `b_i`.
    pub outer_angle: f64,
    /// The curve's control angle or the bend's third angle, `c_i`; `None`
    /// for a straight line.
    pub third_angle: Option<f64>,
}

/// A Cube's crest (spec section 5).
#[derive(Debug, Clone, PartialEq)]
pub struct CrestMark {
    pub shape: PetalShape,
    pub inner_radius: f64,
    pub outer_radius: f64,
    pub twist: f64,
    pub dot: bool,
    pub petals: Vec<Petal>,
}

impl CrestMark {
    /// The crest for a seed made by [`ring_mark::seed`]: exactly six draws,
    /// in the spec's order, every real value in `f64` computed left to right
    /// as written.
    pub fn from_seed(seed: &str) -> Self {
        let mut generator = Generator::new(seed);
        let k = 3 + (generator.next_draw() * 4) / DRAW_RANGE;
        let shape = match (generator.next_draw() * 3) / DRAW_RANGE {
            0 => PetalShape::Line,
            1 => PetalShape::Curve,
            _ => PetalShape::Bent,
        };
        let fraction = |u: u32| f64::from(u) / f64::from(DRAW_RANGE);
        let inner_radius = 8.0 + fraction(generator.next_draw()) * 6.0;
        let outer_radius = 17.0 + fraction(generator.next_draw()) * 5.0;
        let twist = (fraction(generator.next_draw()) - 0.5) * 1.1;
        // The dot's draw comes last; do not move it.
        let dot = generator.next_draw() > DRAW_RANGE / 2;

        let count = f64::from(k);
        let petals = (0..k)
            .map(|i| {
                let angle = f64::from(i * 2) * PI / count - PI / 2.0;
                let outer_angle = angle + twist;
                let third_angle = match shape {
                    PetalShape::Line => None,
                    PetalShape::Curve => Some(outer_angle + 0.5),
                    PetalShape::Bent => Some(outer_angle + (PI / count) * 0.8),
                };
                Petal {
                    angle,
                    outer_angle,
                    third_angle,
                }
            })
            .collect();
        Self {
            shape,
            inner_radius,
            outer_radius,
            twist,
            dot,
            petals,
        }
    }

    /// The number of petals, 3 to 6.
    pub fn petal_count(&self) -> usize {
        self.petals.len()
    }

    /// Petal `petal` as an SVG path on the 64 box, coordinates to two
    /// decimals, as the study wrote them.
    pub fn petal_path(&self, petal: &Petal) -> String {
        let (x0, y0) = point(self.inner_radius, petal.angle);
        let (x1, y1) = point(self.outer_radius, petal.outer_angle);
        match (self.shape, petal.third_angle) {
            (PetalShape::Curve, Some(third)) => {
                let (cx, cy) = point(self.outer_radius, third);
                format!("M{x0:.2} {y0:.2}Q{cx:.2} {cy:.2} {x1:.2} {y1:.2}")
            }
            (PetalShape::Bent, Some(third)) => {
                let (x2, y2) = point(self.outer_radius, third);
                format!("M{x0:.2} {y0:.2}L{x1:.2} {y1:.2}L{x2:.2} {y2:.2}")
            }
            _ => format!("M{x0:.2} {y0:.2}L{x1:.2} {y1:.2}"),
        }
    }
}

fn point(radius: f64, angle: f64) -> (f64, f64) {
    (CENTRE + radius * angle.cos(), CENTRE + radius * angle.sin())
}

/// The crest's shapes on the 64 box, without the `<svg>` wrapper (spec
/// section 6): the faint ring, the petals, and the dot when it has one. All
/// in `ink`; Crest has no orange.
pub fn svg_body(mark: &CrestMark, ink: Color) -> String {
    let ink = ring_mark::hex(ink);
    let stroke = format!(
        r#"fill="none" stroke="{ink}" stroke-width="{STROKE}" stroke-linecap="round" stroke-linejoin="round""#
    );
    let mut body = format!(
        r#"<circle cx="{CENTRE}" cy="{CENTRE}" r="{RING_RADIUS}" {stroke} opacity=".28"/>"#
    );
    for petal in &mark.petals {
        let _ = write!(body, r#"<path d="{}" {stroke}/>"#, mark.petal_path(petal));
    }
    if mark.dot {
        let _ = write!(
            body,
            r#"<circle cx="{CENTRE}" cy="{CENTRE}" r="{DOT_RADIUS}" fill="{ink}"/>"#
        );
    }
    body
}

/// A complete SVG document for the crest on the 64 box, in `ink`. Callers
/// pass the theme's token (`theme.colors.shell.ink`), never a literal.
pub fn svg(mark: &CrestMark, ink: Color) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {BOX} {BOX}">{}</svg>"#,
        svg_body(mark, ink)
    )
}

#[cfg(test)]
mod tests;
