//! The style dispatch, its fallbacks, and each style drawn in both themes.

use super::*;
use crate::component::ring_mark::tests::{count_near, render};
use crate::theme::palette::ShellPalette;

/// A base vector of each style's fixture.
const RING_CUBE: &str = "3994693e-a3ae-4e68-9323-451b4a95902a";
const CREST_CUBES: [&str; 5] = [
    "2571891e-a33e-484b-81e1-f894901f7a63",
    "4bf822a7-ad46-4243-a977-10e63e28ea5a",
    "0c3579f9-347f-4f4e-9a36-7e9694a4346e",
    "b76a25e2-31af-451f-88ab-a680259ecd86",
    "7f041680-d251-4426-a660-f0132a322078",
];
const GRID_CUBES: [&str; 5] = [
    "26329c0c-d0e3-4e97-bc1a-9cec432f438e",
    "39abb429-5dde-4519-90ce-ed0798d02723",
    "7388bbbf-9f15-49e4-acef-da87f9d3cb52",
    "1d6511c0-5829-4110-a76f-e0c491b413c9",
    "945ca68c-e79f-4bdd-a246-620ede33d47a",
];

#[test]
fn style_numbers_round_trip_and_unknown_is_rings() {
    for style in CubeMarkStyle::ALL {
        assert_eq!(CubeMarkStyle::from_u8(style.as_u8()), style);
    }
    assert_eq!(CubeMarkStyle::from_u8(0), CubeMarkStyle::Rings);
    assert_eq!(CubeMarkStyle::from_u8(1), CubeMarkStyle::Crest);
    assert_eq!(CubeMarkStyle::from_u8(2), CubeMarkStyle::Grid);
    for unknown in [3, 7, 128, u8::MAX] {
        assert_eq!(
            CubeMarkStyle::from_u8(unknown),
            CubeMarkStyle::Rings,
            "{unknown}"
        );
    }
    assert_eq!(CubeMarkStyle::default(), CubeMarkStyle::Rings);
    assert_eq!(
        CubeMarkStyle::ALL.map(CubeMarkStyle::name),
        ["Rings", "Crest", "Grid"]
    );
}

#[test]
fn svg_dispatches_to_each_styles_own_drawing() {
    let palette = ShellPalette::dark();
    let (ink, accent) = (palette.ink, palette.accent);
    let id = CREST_CUBES[1];
    let seed = ring_mark::seed(id, 0).unwrap();
    assert_eq!(
        svg_body(CubeMarkStyle::Rings, Some(&seed), ink, accent),
        ring_mark::svg_body(Some(&ring_mark::RingMark::from_seed(&seed)), ink, accent)
    );
    assert_eq!(
        svg_body(CubeMarkStyle::Crest, Some(&seed), ink, accent),
        crest_mark::svg_body(&crest_mark::CrestMark::from_seed(&seed), ink)
    );
    assert_eq!(
        svg_body(CubeMarkStyle::Grid, Some(&seed), ink, accent),
        grid_mark::svg_body(&grid_mark::GridMark::from_seed(&seed), ink)
    );
    // Only Rings uses the accent.
    let accent_hex = ring_mark::hex(accent);
    assert!(svg(CubeMarkStyle::Rings, Some(&seed), ink, accent).contains(&accent_hex));
    assert!(!svg(CubeMarkStyle::Crest, Some(&seed), ink, accent).contains(&accent_hex));
    assert!(!svg(CubeMarkStyle::Grid, Some(&seed), ink, accent).contains(&accent_hex));
}

/// No usable seed draws the ring mark's centre dot alone, in every style.
#[test]
fn every_style_falls_back_to_the_dot() {
    let palette = ShellPalette::light();
    let dot = ring_mark::svg(None, palette.ink, palette.accent);
    for style in CubeMarkStyle::ALL {
        for seed in [
            None,
            Some(""),
            Some("42"),
            Some("9FB93534-081C-4253-B1E2-73A87984A8AB"),
        ] {
            assert_eq!(
                svg(style, seed, palette.ink, palette.accent),
                dot,
                "{style:?} {seed:?}"
            );
        }
    }
}

/// The variation number is shared: switching to another style and back
/// draws exactly the earlier mark, and each style changes with the number.
#[test]
fn switching_style_and_back_restores_the_earlier_mark() {
    let palette = ShellPalette::dark();
    let draw = |style, variation| {
        let seed = ring_mark::seed(RING_CUBE, variation);
        svg(style, seed.as_deref(), palette.ink, palette.accent)
    };
    let before = draw(CubeMarkStyle::Rings, 3);
    assert_ne!(draw(CubeMarkStyle::Crest, 3), before);
    assert_ne!(draw(CubeMarkStyle::Grid, 3), before);
    assert_eq!(draw(CubeMarkStyle::Rings, 3), before);
    for style in CubeMarkStyle::ALL {
        assert_ne!(
            draw(style, 3),
            draw(style, 4),
            "{style:?} changes with the number"
        );
    }
}

#[test]
fn widgets_build_in_every_style() {
    let seed = ring_mark::seed(GRID_CUBES[0], 2);
    for style in CubeMarkStyle::ALL {
        let _: Element<'_, ()> = cube_mark(style, seed.as_deref(), 48.0);
        let _: Element<'_, ()> = cube_mark_muted(style, seed.as_deref(), 44.0);
        let _: Element<'_, ()> = cube_mark(style, None, ring_mark::MIN_SIZE);
    }
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "at least 24px")]
fn a_crest_below_the_minimum_asserts_in_debug() {
    let seed = ring_mark::seed(CREST_CUBES[0], 0);
    let _: Element<'_, ()> = cube_mark(CubeMarkStyle::Crest, seed.as_deref(), 20.0);
}

#[test]
fn layers_are_cached_per_style_and_seed() {
    let seed = ring_mark::seed(CREST_CUBES[2], 0).unwrap();
    let crest = layer(CubeMarkStyle::Crest, &seed);
    assert_eq!(crest.id(), layer(CubeMarkStyle::Crest, &seed).id());
    assert_ne!(crest.id(), layer(CubeMarkStyle::Grid, &seed).id());
    let text = match crest.data() {
        iced::advanced::svg::Data::Bytes(bytes) => String::from_utf8(bytes.to_vec()).unwrap(),
        iced::advanced::svg::Data::Path(_) => unreachable!("layers are built from memory"),
    };
    // Drawn in black for the theme's tint; the faint ring keeps its opacity.
    assert!(text.contains("#000000"));
    assert!(text.contains(r#"opacity=".28""#));
}

/// The five base vectors of Crest and Grid at 56 px, the specs' golden size,
/// in both themes: ink and no accent, and the muted variant in the muted
/// colour. Rings is covered by the ring mark's own tests; one Rings Cube
/// here checks `cube_mark` keeps its orange. Set `CUBE_MARK_PREVIEW_DIR` to
/// save the renders as PNGs, for a by-eye check against each fixture's
/// `svg`.
#[test]
fn styles_render_in_the_theme_colours() {
    const SIDE: u32 = 56;
    let preview = std::env::var_os("CUBE_MARK_PREVIEW_DIR");
    for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
        let shell = theme.colors.shell;
        for (style, ids) in [
            (CubeMarkStyle::Crest, CREST_CUBES),
            (CubeMarkStyle::Grid, GRID_CUBES),
        ] {
            for id in ids {
                let seed = ring_mark::seed(id, 0);
                let pixels = render(cube_mark(style, seed.as_deref(), SIDE as f32), &theme, SIDE);
                assert_eq!(
                    count_near(&pixels, shell.accent),
                    0,
                    "{name} {style:?} {id}: no orange"
                );
                assert!(
                    count_near(&pixels, shell.ink) > 20,
                    "{name} {style:?} {id}: ink"
                );
                if let Some(dir) = &preview {
                    image::save_buffer(
                        std::path::Path::new(dir).join(format!("{}-{id}-{name}.png", style.name())),
                        &pixels,
                        SIDE,
                        SIDE,
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                }
                let muted = render(
                    cube_mark_muted(style, seed.as_deref(), SIDE as f32),
                    &theme,
                    SIDE,
                );
                assert!(
                    count_near(&muted, shell.muted) > 20,
                    "{name} {style:?}: muted"
                );
                assert_eq!(
                    count_near(&muted, shell.ink),
                    0,
                    "{name} {style:?}: muted has no ink"
                );
            }
        }
        let rings = ring_mark::seed(RING_CUBE, 0);
        let pixels = render(
            cube_mark(CubeMarkStyle::Rings, rings.as_deref(), SIDE as f32),
            &theme,
            SIDE,
        );
        assert!(
            count_near(&pixels, shell.accent) > 20,
            "{name}: Rings keeps its orange"
        );
    }
}
