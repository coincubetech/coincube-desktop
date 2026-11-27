//! How far apart two colours look, for checks that one box still reads
//! against another once boxes have no outline (Desktop V3 Phase 1, R4).
//!
//! A `!=` on colours passes for a one-unit difference nobody can see, so the
//! checks use the CIE 1976 colour difference (ΔE*ab) between opaque sRGB
//! colours, and WCAG contrast where an edge (a border) carries the
//! separation instead of a fill.

use iced::Color;

/// The smallest ΔE*ab treated as a visible change between two fills: 2.3,
/// the commonly cited just-noticeable difference for CIELAB (Sharma, *Digital
/// Color Imaging Handbook*, 2003). Two fills closer than this read as one.
pub const JUST_NOTICEABLE_DELTA_E: f32 = 2.3;

/// The WCAG 2.1 non-text contrast (SC 1.4.11) for a component's boundary.
pub const NON_TEXT_CONTRAST: f32 = 3.0;

fn linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of an opaque colour.
pub fn relative_luminance(color: Color) -> f32 {
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

/// WCAG contrast ratio, from 1 (identical) to 21.
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn lab(color: Color) -> [f32; 3] {
    let (r, g, b) = (linear(color.r), linear(color.g), linear(color.b));
    // sRGB to XYZ, D65, normalised by the white point.
    let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.950_47;
    let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.088_83;
    let f = |t: f32| {
        if t > 0.008_856 {
            t.cbrt()
        } else {
            7.787 * t + 16.0 / 116.0
        }
    };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// CIE 1976 colour difference ΔE*ab between two opaque colours.
pub fn delta_e(a: Color, b: Color) -> f32 {
    let (a, b) = (lab(a), lab(b));
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_e_and_contrast_match_known_values() {
        let white = iced::color!(0xffffff);
        let black = iced::color!(0x000000);
        assert!((delta_e(white, black) - 100.0).abs() < 0.1);
        assert!((contrast_ratio(white, black) - 21.0).abs() < 0.01);
        assert_eq!(delta_e(white, white), 0.0);
        // One unit apart is far below anything visible.
        assert!(delta_e(white, iced::color!(0xfefefe)) < 1.0);
        // Light theme's panel on its page: the closest pair in the app.
        let separation = delta_e(white, iced::color!(0xfaf9f6));
        assert!((2.5..2.7).contains(&separation), "{}", separation);
    }
}
