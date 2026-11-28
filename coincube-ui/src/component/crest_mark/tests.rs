//! The spec's fixture, loaded unchanged.
//!
//! `testdata/crest-mark-v1-vectors.json` is a byte-for-byte copy of
//! `plans/cube-avatars/crest-mark-v1-vectors.json` in the company brain.
//! Never edit it here: a change to the algorithm is a new spec version.

use serde::Deserialize;

use super::*;
use crate::component::ring_mark::seed;
use crate::theme::palette::ShellPalette;

const FIXTURE: &str = include_str!("testdata/crest-mark-v1-vectors.json");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fixture {
    spec: String,
    #[allow(dead_code)]
    note: String,
    vectors: Vec<Vector>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Vector {
    cube_id: String,
    variation: u32,
    seed: String,
    fnv_state: String,
    draws: Vec<u32>,
    petal_count: usize,
    shape: u32,
    inner_radius: f64,
    outer_radius: f64,
    twist: f64,
    dot: bool,
    petals: Vec<FixturePetal>,
    /// The study's own drawing, for comparing by eye; not asserted (spec
    /// section 6: its coordinates come from `cos` and `sin`).
    #[allow(dead_code)]
    svg: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FixturePetal {
    angle: f64,
    outer_angle: f64,
    third_angle: Option<f64>,
}

fn fixture() -> Fixture {
    serde_json::from_str(FIXTURE).expect("the crest mark fixture parses")
}

fn close(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "{what}: {actual} vs {expected}"
    );
}

#[test]
fn fixture_is_version_1_with_every_case() {
    let fixture = fixture();
    assert_eq!(fixture.spec, "SPEC-cube-crest-mark-v1.md");
    assert_eq!(VERSION, 1);
    assert_eq!(fixture.vectors.len(), 8);
    // The five base Cubes cover every petal count, shape and dot state.
    let base: Vec<_> = fixture
        .vectors
        .iter()
        .filter(|v| v.variation == 0)
        .collect();
    assert_eq!(base.len(), 5);
    for k in 3..=6 {
        assert!(base.iter().any(|v| v.petal_count == k), "{k} petals");
    }
    for shape in 0..3 {
        assert!(base.iter().any(|v| v.shape == shape), "shape {shape}");
    }
}

/// Every field but `svg`, for every vector: integers exactly, real values
/// within 1e-6.
#[test]
fn every_vector_matches() {
    for vector in fixture().vectors {
        let seed = seed(&vector.cube_id, vector.variation).expect("fixture ids are UUIDs");
        assert_eq!(seed, vector.seed, "seed for {}", vector.cube_id);

        let mut generator = Generator::new(&seed);
        assert_eq!(
            format!("{:#010x}", generator.state()),
            vector.fnv_state,
            "fnvState for {seed}"
        );
        assert_eq!(vector.draws.len(), 6, "six draws for {seed}");
        let draws: Vec<u32> = (0..6).map(|_| generator.next_draw()).collect();
        assert_eq!(draws, vector.draws, "draws for {seed}");

        let mark = CrestMark::from_seed(&seed);
        assert_eq!(
            mark.petal_count(),
            vector.petal_count,
            "petalCount for {seed}"
        );
        assert_eq!(mark.shape.index(), vector.shape, "shape for {seed}");
        close(
            mark.inner_radius,
            vector.inner_radius,
            &format!("innerRadius for {seed}"),
        );
        close(
            mark.outer_radius,
            vector.outer_radius,
            &format!("outerRadius for {seed}"),
        );
        close(mark.twist, vector.twist, &format!("twist for {seed}"));
        assert_eq!(mark.dot, vector.dot, "dot for {seed}");
        assert_eq!(mark.petals.len(), vector.petals.len(), "petals for {seed}");
        for (i, (petal, expected)) in mark.petals.iter().zip(&vector.petals).enumerate() {
            close(
                petal.angle,
                expected.angle,
                &format!("angle {i} for {seed}"),
            );
            close(
                petal.outer_angle,
                expected.outer_angle,
                &format!("outerAngle {i} for {seed}"),
            );
            match (petal.third_angle, expected.third_angle) {
                (Some(actual), Some(expected)) => {
                    close(actual, expected, &format!("thirdAngle {i} for {seed}"))
                }
                (None, None) => {}
                (actual, expected) => {
                    panic!("thirdAngle {i} for {seed}: {actual:?} vs {expected:?}")
                }
            }
        }
    }
}

fn shell_svgs(id: &str) -> (String, String) {
    let mark = CrestMark::from_seed(&seed(id, 0).unwrap());
    (
        svg(&mark, ShellPalette::dark().ink),
        svg(&mark, ShellPalette::light().ink),
    )
}

/// The SVG for the five base vectors, in both themes. Set
/// `CUBE_MARK_UPDATE_SNAPSHOTS=1` to rewrite the snapshot files, then compare
/// the result by eye with the fixture's `svg` before committing.
#[test]
fn base_vectors_match_their_svg_snapshots() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/component/crest_mark/testdata/snapshots");
    let update = std::env::var_os("CUBE_MARK_UPDATE_SNAPSHOTS").is_some();
    for vector in fixture().vectors.iter().filter(|v| v.variation == 0) {
        let (dark, light) = shell_svgs(&vector.cube_id);
        for (theme, svg) in [("dark", dark), ("light", light)] {
            let path = dir.join(format!("{}-{theme}.svg", vector.cube_id));
            if update {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(&path, format!("{svg}\n")).unwrap();
            }
            let expected = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("snapshot {}: {e}", path.display()));
            assert_eq!(svg, expected.trim_end(), "snapshot {}", path.display());
        }
    }
}

#[test]
fn svg_draws_the_faint_ring_the_petals_and_no_orange() {
    let palette = ShellPalette::dark();
    for vector in fixture().vectors {
        let mark = CrestMark::from_seed(&vector.seed);
        let svg = svg(&mark, palette.ink);
        assert!(svg.contains(r#"<circle cx="32" cy="32" r="29" fill="none""#));
        assert!(svg.contains(r#"opacity=".28""#));
        assert_eq!(svg.matches("<path").count(), vector.petal_count);
        assert_eq!(
            svg.contains(r#"r="3" fill="#),
            vector.dot,
            "{}",
            vector.seed
        );
        assert!(!svg.contains(&ring_mark::hex(palette.accent)));
        assert!(!svg.contains("<rect"));
    }
}

#[test]
fn each_shape_draws_its_own_path() {
    let line = CrestMark::from_seed("2571891e-a33e-484b-81e1-f894901f7a63");
    let path = |mark: &CrestMark| mark.petal_path(&mark.petals[0]);
    for (mark, letters) in [
        (line.clone(), "ML"),
        (
            CrestMark {
                shape: PetalShape::Curve,
                ..line.clone()
            },
            "MQ",
        ),
        (
            CrestMark {
                shape: PetalShape::Bent,
                ..line.clone()
            },
            "MLL",
        ),
    ] {
        let mut mark = mark;
        if mark.shape != PetalShape::Line {
            mark.petals[0].third_angle = Some(0.0);
        }
        let commands: String = path(&mark)
            .chars()
            .filter(char::is_ascii_alphabetic)
            .collect();
        assert_eq!(commands, letters, "{:?}", mark.shape);
    }
}
