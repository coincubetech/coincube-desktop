//! The line Cube, turning: a loading mark drawn from the logo itself
//! (Robert, 2026-11-27, "turn and rest").
//!
//! The Cube is a real box seen from above, turning on its upright axis. Only
//! the edges of faces turned toward the viewer are drawn. The box is
//! [`HALF_HEIGHT`] tall and seen from [`VIEW_SIN`] above, so that at a 45°
//! turn its edges fall exactly on the logo's 24-unit drawing
//! (`M12 3l8 4.5v9L12 21l-8-4.5v-9z`, `M4 7.5l8 4.5 8-4.5M12 12v9`). Every
//! resting frame, and the whole of a reduced-motion run, is the logo.
//!
//! Motion: a quarter turn in [`TURN`], eased, then [`REST`] on the logo pose.
//! The canvas asks for the next frame itself, so the screen that shows it
//! needs no timer.
//!
//! It is the app's one loading mark (Robert, 2026-11-27): the Cube loader
//! (turning Cube, then the quote, then the status) and the smaller waits in
//! panels both use it. Views that do not hold the motion preference build
//! [`loading_cube`] inside [`render_with_motion`], which the GUI shell sets
//! for every tab view.

use std::{cell::Cell, time::Duration};

use iced::{
    advanced::widget::Id,
    mouse,
    widget::canvas::{self, Canvas, Frame, Geometry, LineCap, LineJoin, Path, Stroke},
    window, Length, Point, Rectangle, Renderer,
};

use crate::{
    theme::Theme,
    widget::{Container, Element},
};

thread_local! {
    static MOTION_REDUCED: Cell<bool> = const { Cell::new(false) };
}

/// Build a view with the window's reduce-motion preference, for the
/// [`loading_cube`]s inside it.
pub fn render_with_motion<T>(reduced: bool, render: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            MOTION_REDUCED.set(self.0);
        }
    }

    let _restore = Restore(MOTION_REDUCED.replace(reduced));
    render()
}

/// Whether the view being built reduces motion (see [`render_with_motion`]).
pub fn motion_reduced() -> bool {
    MOTION_REDUCED.get()
}

/// The id of the container every turning Cube is drawn in. A canvas reports
/// nothing to widget operations, so tests find the Cube, and where it sits
/// relative to the text around it, by this id.
pub fn id() -> Id {
    Id::new("turning-cube")
}

/// One quarter turn.
pub const TURN: Duration = Duration::from_millis(700);
/// The pause on the logo pose after each turn.
pub const REST: Duration = Duration::from_millis(500);

/// Sine of the view angle above the box (34.2°).
const VIEW_SIN: f32 = 0.5625;
/// Half the box's height for a side of 1: the logo's proportions.
const HALF_HEIGHT: f32 = 0.481_070_24;
/// Grid units per box side on the 24-unit drawing.
const UNITS_PER_SIDE: f32 = 11.313_708;
/// The logo pose: a 45° turn.
const REST_YAW: f32 = std::f32::consts::FRAC_PI_4;

fn view_cos() -> f32 {
    (1.0 - VIEW_SIN * VIEW_SIN).sqrt()
}

/// The turn, in radians, `elapsed` into the motion. Reduced motion holds the
/// logo pose.
pub fn yaw_at(elapsed: Duration, reduced: bool) -> f32 {
    if reduced {
        return REST_YAW;
    }
    let cycle = (TURN + REST).as_secs_f64();
    let t = elapsed.as_secs_f64();
    let turns = (t / cycle).floor();
    let into = t - turns * cycle;
    let turn = TURN.as_secs_f64();
    let progress = if into < turn { ease(into / turn) } else { 1.0 };
    REST_YAW + ((turns + progress) * std::f64::consts::FRAC_PI_2) as f32
}

/// While the Cube rests on the logo pose `elapsed` into the motion, how long
/// until the next turn starts. `None` while it is turning.
pub fn rest_left(elapsed: Duration) -> Option<Duration> {
    let cycle = (TURN + REST).as_nanos();
    let into = elapsed.as_nanos() % cycle;
    (into >= TURN.as_nanos()).then(|| Duration::from_nanos((cycle - into) as u64))
}

/// Ease in and out (cubic).
fn ease(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// A point of the box after the turn and the view: `(x, y_up, depth)`.
fn project(p: [f32; 3], yaw: f32) -> [f32; 3] {
    let (s, c) = yaw.sin_cos();
    let x = p[0] * c + p[2] * s;
    let z = -p[0] * s + p[2] * c;
    let vc = view_cos();
    [x, p[1] * vc - z * VIEW_SIN, p[1] * VIEW_SIN + z * vc]
}

/// The edges to draw at `yaw`, in the logo's 24-unit grid (y down).
pub fn visible_edges(yaw: f32) -> Vec<(Point, Point)> {
    let corner = |i: usize| -> [f32; 3] {
        [
            if i & 4 != 0 { 0.5 } else { -0.5 },
            if i & 2 != 0 {
                HALF_HEIGHT
            } else {
                -HALF_HEIGHT
            },
            if i & 1 != 0 { 0.5 } else { -0.5 },
        ]
    };
    // Each face: its outward normal and its four corners, in order.
    let faces: [([f32; 3], [usize; 4]); 6] = [
        ([0.0, 1.0, 0.0], [2, 6, 7, 3]),
        ([0.0, -1.0, 0.0], [0, 4, 5, 1]),
        ([1.0, 0.0, 0.0], [4, 6, 7, 5]),
        ([-1.0, 0.0, 0.0], [0, 2, 3, 1]),
        ([0.0, 0.0, 1.0], [1, 5, 7, 3]),
        ([0.0, 0.0, -1.0], [0, 4, 6, 2]),
    ];
    let to_grid = |p: [f32; 3]| {
        let q = project(p, yaw);
        Point::new(12.0 + q[0] * UNITS_PER_SIDE, 12.0 - q[1] * UNITS_PER_SIDE)
    };
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for (normal, corners) in faces {
        if project(normal, yaw)[2] <= 1e-6 {
            continue;
        }
        for i in 0..4 {
            let (a, b) = (corners[i], corners[(i + 1) % 4]);
            let key = (a.min(b), a.max(b));
            if !edges.contains(&key) {
                edges.push(key);
            }
        }
    }
    edges
        .into_iter()
        .map(|(a, b)| (to_grid(corner(a)), to_grid(corner(b))))
        .collect()
}

/// The line weight in grid units for a Cube drawn `size` pixels square
/// (brand brief, section 3).
pub fn line_units(size: f32) -> f32 {
    if size <= 32.0 {
        1.5
    } else if size <= 128.0 {
        1.0
    } else {
        0.75
    }
}

struct TurningCube {
    size: f32,
    reduced: bool,
    opacity: f32,
}

/// When the motion started and the frame being drawn.
#[derive(Default)]
struct Clock {
    start: Option<iced::time::Instant>,
    now: Option<iced::time::Instant>,
}

impl Clock {
    fn elapsed(&self) -> Duration {
        match (self.start, self.now) {
            (Some(start), Some(now)) => now.saturating_duration_since(start),
            _ => Duration::ZERO,
        }
    }
}

impl<M> canvas::Program<M, Theme> for TurningCube {
    type State = Clock;

    fn update(
        &self,
        state: &mut Clock,
        event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<M>> {
        if self.reduced {
            return None;
        }
        if let iced::Event::Window(window::Event::RedrawRequested(now)) = event {
            state.start.get_or_insert(*now);
            state.now = Some(*now);
            // Every frame while turning; while resting, one wake when the next
            // turn starts (the resting frame does not change).
            return Some(match rest_left(state.elapsed()) {
                Some(left) => canvas::Action::request_redraw_at(*now + left),
                None => canvas::Action::request_redraw(),
            });
        }
        None
    }

    fn draw(
        &self,
        state: &Clock,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let unit = bounds.width.min(bounds.height) / 24.0;
        let stroke = Stroke::default()
            .with_color(theme.colors.shell.ink.scale_alpha(self.opacity))
            .with_width(line_units(self.size) * unit)
            .with_line_cap(LineCap::Round)
            .with_line_join(LineJoin::Round);
        for (a, b) in visible_edges(yaw_at(state.elapsed(), self.reduced)) {
            let from = Point::new(a.x * unit, a.y * unit);
            let to = Point::new(b.x * unit, b.y * unit);
            frame.stroke(&Path::line(from, to), stroke);
        }
        vec![frame.into_geometry()]
    }
}

/// The turning line Cube, `size` pixels square, in the theme's ink. With
/// `reduced` set it holds the logo pose.
pub fn turning_cube<'a, M: 'a>(size: f32, reduced: bool) -> Element<'a, M> {
    turning_cube_faded(size, reduced, 1.0)
}

/// [`turning_cube`] at `opacity`, for a screen whose content fades in.
pub fn turning_cube_faded<'a, M: 'a>(size: f32, reduced: bool, opacity: f32) -> Element<'a, M> {
    Container::new(
        Canvas::new(TurningCube {
            size,
            reduced,
            opacity,
        })
        .width(Length::Fixed(size))
        .height(Length::Fixed(size)),
    )
    .id(id())
    .into()
}

/// [`turning_cube`] under the motion preference of the view being built
/// (see [`render_with_motion`]).
pub fn loading_cube<'a, M: 'a>(size: f32) -> Element<'a, M> {
    turning_cube(size, motion_reduced())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Point, b: (f32, f32)) -> bool {
        (a.x - b.0).abs() < 1e-3 && (a.y - b.1).abs() < 1e-3
    }

    /// At rest the Cube is the logo: the six outline edges and three inner
    /// edges of `M12 3l8 4.5v9L12 21l-8-4.5v-9z` and `M4 7.5l8 4.5 8-4.5M12 12v9`.
    #[test]
    fn the_resting_pose_is_the_logo() {
        let logo = [
            ((12.0, 3.0), (20.0, 7.5)),
            ((20.0, 7.5), (20.0, 16.5)),
            ((20.0, 16.5), (12.0, 21.0)),
            ((12.0, 21.0), (4.0, 16.5)),
            ((4.0, 16.5), (4.0, 7.5)),
            ((4.0, 7.5), (12.0, 3.0)),
            ((4.0, 7.5), (12.0, 12.0)),
            ((12.0, 12.0), (20.0, 7.5)),
            ((12.0, 12.0), (12.0, 21.0)),
        ];
        let edges = visible_edges(REST_YAW);
        assert_eq!(edges.len(), logo.len());
        for (a, b) in logo {
            assert!(
                edges
                    .iter()
                    .any(|(p, q)| (close(*p, a) && close(*q, b)) || (close(*p, b) && close(*q, a))),
                "the resting Cube is missing the logo edge {a:?}–{b:?}"
            );
        }
    }

    #[test]
    fn it_rests_on_the_logo_between_turns() {
        let quarter = std::f32::consts::FRAC_PI_2;
        let at = |ms: u64| yaw_at(Duration::from_millis(ms), false);
        assert!((at(0) - REST_YAW).abs() < 1e-6);
        // Mid-turn it has moved; after the turn it rests one quarter on.
        assert!(at(350) > REST_YAW + 0.1 && at(350) < REST_YAW + quarter - 0.1);
        for ms in [700, 900, 1199] {
            assert!((at(ms) - (REST_YAW + quarter)).abs() < 1e-5, "{ms} ms");
        }
        assert!((at(1200 + 700) - (REST_YAW + 2.0 * quarter)).abs() < 1e-5);
    }

    #[test]
    fn reduced_motion_holds_the_logo_pose() {
        for ms in [0, 350, 5000] {
            assert_eq!(yaw_at(Duration::from_millis(ms), true), REST_YAW);
        }
    }

    #[test]
    fn mid_turn_shows_no_hidden_edges() {
        // The top face and at most two sides face the viewer: never more than
        // nine edges, and never the bottom face alone.
        for step in 0..90 {
            let yaw = REST_YAW + (step as f32).to_radians();
            let n = visible_edges(yaw).len();
            assert!((7..=9).contains(&n), "{n} edges at step {step}");
        }
    }

    #[test]
    fn it_idles_through_each_rest() {
        let at = |ms: u64| rest_left(Duration::from_millis(ms));
        assert_eq!(at(0), None);
        assert_eq!(at(699), None);
        assert_eq!(at(700), Some(Duration::from_millis(500)));
        assert_eq!(at(1000), Some(Duration::from_millis(200)));
        assert_eq!(at(1200), None, "the next turn has begun");
    }

    /// The bottom-back corner is behind the box at every turn: no drawn edge
    /// ends there. (At rest it projects onto the logo's inner corner, so the
    /// check starts a little into a turn.) Fails if face culling is inverted.
    #[test]
    fn the_hidden_corner_is_never_drawn() {
        for step in (5..90).step_by(5) {
            let yaw = REST_YAW + (step as f32).to_radians();
            let farthest = [[-0.5, -0.5], [-0.5, 0.5], [0.5, -0.5], [0.5, 0.5]]
                .into_iter()
                .map(|[x, z]| project([x, -HALF_HEIGHT, z], yaw))
                .min_by(|a, b| a[2].total_cmp(&b[2]))
                .unwrap();
            let hidden = Point::new(
                12.0 + farthest[0] * UNITS_PER_SIDE,
                12.0 - farthest[1] * UNITS_PER_SIDE,
            );
            for (a, b) in visible_edges(yaw) {
                assert!(
                    a.distance(hidden) > 1e-3 && b.distance(hidden) > 1e-3,
                    "step {step}: an edge ends at the hidden corner"
                );
            }
        }
    }

    #[test]
    fn the_motion_preference_scopes_to_one_view() {
        assert!(!motion_reduced());
        let inside = render_with_motion(true, || {
            let nested = render_with_motion(false, motion_reduced);
            (motion_reduced(), nested)
        });
        assert_eq!(inside, (true, false));
        assert!(
            !motion_reduced(),
            "the preference is restored after the view"
        );
    }

    #[test]
    fn the_line_follows_the_brief() {
        assert_eq!(line_units(24.0), 1.5);
        assert_eq!(line_units(128.0), 1.0);
        assert_eq!(line_units(256.0), 0.75);
    }
}
