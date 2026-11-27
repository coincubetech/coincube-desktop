//! The spec's fixture, loaded unchanged.
//!
//! `testdata/grid-mark-v1-vectors.json` is a byte-for-byte copy of
//! `plans/cube-avatars/grid-mark-v1-vectors.json` in the company brain.
//! Never edit it here: a change to the algorithm is a new spec version.

use serde::Deserialize;

use super::*;
use crate::component::ring_mark::seed;
use crate::theme::palette::ShellPalette;

const FIXTURE: &str = include_str!("testdata/grid-mark-v1-vectors.json");

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
    rows: Vec<String>,
    /// The study's own drawing, for comparing by eye; not asserted.
    #[allow(dead_code)]
    svg: String,
}

fn fixture() -> Fixture {
    serde_json::from_str(FIXTURE).expect("the grid mark fixture parses")
}

#[test]
fn fixture_is_version_1_with_every_case() {
    let fixture = fixture();
    assert_eq!(fixture.spec, "SPEC-cube-grid-mark-v1.md");
    assert_eq!(VERSION, 1);
    assert_eq!(fixture.vectors.len(), 8);
    assert_eq!(
        fixture.vectors.iter().filter(|v| v.variation == 0).count(),
        5
    );
}

/// Every field but `svg`, for every vector, exactly: every step is on
/// integers.
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
        assert_eq!(vector.draws.len(), 15, "fifteen draws for {seed}");
        let draws: Vec<u32> = (0..15).map(|_| generator.next_draw()).collect();
        assert_eq!(draws, vector.draws, "draws for {seed}");

        let mark = GridMark::from_seed(&seed);
        let rows: Vec<String> = (0..SIDE).map(|y| mark.row_string(y)).collect();
        assert_eq!(rows, vector.rows, "rows for {seed}");
    }
}

#[test]
fn every_row_is_mirrored() {
    for vector in fixture().vectors {
        let mark = GridMark::from_seed(&vector.seed);
        for row in mark.rows {
            assert_eq!(row[0], row[4]);
            assert_eq!(row[1], row[3]);
        }
    }
}

fn shell_svgs(id: &str) -> (String, String) {
    let mark = GridMark::from_seed(&seed(id, 0).unwrap());
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
        .join("src/component/grid_mark/testdata/snapshots");
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

/// Grid has no ring, no background and no orange: 25 dots in ink, large
/// and solid or small and faint.
#[test]
fn svg_draws_25_dots_and_nothing_else() {
    let palette = ShellPalette::light();
    for vector in fixture().vectors {
        let mark = GridMark::from_seed(&vector.seed);
        let svg = svg(&mark, palette.ink);
        let on = vector
            .rows
            .iter()
            .flat_map(|r| r.chars())
            .filter(|c| *c == '1')
            .count();
        assert_eq!(svg.matches("<circle").count(), 25);
        assert_eq!(svg.matches(r#"r="3.2""#).count(), on, "{}", vector.seed);
        assert_eq!(svg.matches(r#"r="1.1""#).count(), 25 - on);
        assert_eq!(svg.matches(r#"opacity="0.3""#).count(), 25 - on);
        assert!(!svg.contains("<path"));
        assert!(!svg.contains("<rect"));
        assert!(!svg.contains(&ring_mark::hex(palette.accent)));
        // The corners and the centre sit where the spec puts them.
        assert!(svg.contains(r#"cx="14" cy="14""#));
        assert!(svg.contains(r#"cx="50" cy="50""#));
        assert!(svg.contains(r#"cx="32" cy="32""#));
    }
}
