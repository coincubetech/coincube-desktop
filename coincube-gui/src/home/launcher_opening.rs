//! Opening on the Cube (canvas boards O-Launcher, O-Glow-B, O-Stall and
//! O-List).
//!
//! After the PIN or passkey the Cube tumbles to its bottom face, the one face
//! nothing used yet, and grows to the 700px loading face. Its edge glows in
//! slowly, then breathes, faster and brighter with each loader stage. The App
//! shows as soon as the Cube is ready: the O-Finale-B rush and wash that used
//! to follow only held a ready Cube back (about 1.5 s), so they are gone.
//!
//! Without the Launcher floor behind it (List view, which reduced motion
//! forces) the same content fills the screen inside a glowing frame, with no
//! tumble; under reduced motion the frame does not flicker or breathe either.
//!
//! This is presentation only. The loader's own `Step` decides the stage, and
//! nothing here starts, stops or retries a load.

use coincube_ui::component::quote_display::{Quote, QuoteProvider};
use coincube_ui::locale::Text as CopyText;
use std::time::{Duration, Instant};

use super::launcher_catalog::LauncherSceneContext;
use super::launcher_gpu::Glow;
use super::launcher_scene::{self, Pose};

/// `.cube { transition: transform 820ms }`: unlock face to loading face.
const TUMBLE: Duration = launcher_scene::TURN;
/// `.fc.load { animation: loadin 700ms var(--spin) 500ms }`.
const CONTENT_DELAY: Duration = Duration::from_millis(500);
const CONTENT_FADE: Duration = Duration::from_millis(700);
/// `.lload { animation: loadin 500ms var(--spin) }`.
const FRAME_FADE: Duration = Duration::from_millis(500);
/// The breath fades in over this long once its delay is up, so the light
/// starts slow and soft rather than with a flicker.
const GLOW_IN: Duration = Duration::from_millis(1600);
/// One breath at the first stage, and at the last before Ready. Each stage
/// in between breathes faster, so the light quickens as the load goes on.
const BREATHE_SLOW: Duration = Duration::from_millis(2600);
const BREATHE_FAST: Duration = Duration::from_millis(1100);
/// `::before { transition: opacity 900ms var(--spin) }`.
const LEVEL_FADE: Duration = Duration::from_millis(900);
/// A stage held this long offers Cancel.
pub(crate) const STALL: Duration = Duration::from_secs(15);
/// Waiting on Spark this long offers to go on without it, so a Spark or
/// Breez outage never keeps a user from the rest of their Cube.
pub(crate) const SPARK_WAIT: Duration = Duration::from_secs(16);

/// `--lvl` for each stage index.
const LEVELS: [f32; 6] = [0.35, 0.5, 0.65, 0.8, 0.9, 1.0];

const EASE_IN_OUT: (f32, f32, f32, f32) = (0.42, 0.0, 0.58, 1.0);

/// The stage line, quoted from the loader's own view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    Connecting,
    StartingDaemon,
    Scanning,
    Syncing,
    AlmostThere,
    /// The App is up and its Spark wallet has not returned a synced
    /// balance yet.
    Spark,
    Ready,
}

impl Stage {
    pub fn text(self) -> &'static str {
        match self {
            Stage::Connecting => CopyText::ConnectingToDaemon.get(),
            Stage::StartingDaemon => CopyText::StartingDaemon.get(),
            Stage::Scanning => CopyText::ScanningTheBlockchain.get(),
            Stage::Syncing => CopyText::SyncingBlockchain.get(),
            Stage::AlmostThere => CopyText::AlmostThere.get(),
            Stage::Spark => CopyText::LoadingYourSparkWallet.get(),
            Stage::Ready => CopyText::Ready.get(),
        }
    }

    fn index(self) -> usize {
        match self {
            Stage::Connecting => 0,
            Stage::StartingDaemon | Stage::Scanning => 1,
            Stage::Syncing => 2,
            Stage::AlmostThere => 3,
            Stage::Spark => 4,
            Stage::Ready => 5,
        }
    }

    fn level(self) -> f32 {
        LEVELS[self.index()]
    }

    /// How far along the breath is, 0 at the first stage to 1 at the last
    /// before Ready: it sets the breath's speed and depth.
    fn intensity(self) -> f32 {
        (self.index() as f32 / 4.0).min(1.0)
    }

    /// How long this stage runs before the face offers a way out.
    fn wait(self) -> Duration {
        if self == Stage::Spark {
            SPARK_WAIT
        } else {
            STALL
        }
    }

    fn breath(self) -> Duration {
        BREATHE_SLOW.mul_f32(1.0 - self.intensity()) + BREATHE_FAST.mul_f32(self.intensity())
    }
}

/// One Cube's opening, from the end of the unlock to the App.
#[derive(Debug, Clone)]
pub(crate) struct Opening {
    /// The Launcher page the Cube was unlocked on. `None` opens full screen.
    floor: Option<LauncherSceneContext>,
    reduced: bool,
    started: Instant,
    stage: Stage,
    stage_since: Instant,
    level_from: f32,
    level_since: Instant,
    intensity_from: f32,
    /// Breaths taken up to `breath_since`, at the previous stages' speeds,
    /// so a quicker stage carries on from the same point in the breath.
    breaths: f32,
    breath_since: Instant,
    /// An error page held the stall wait; the next stage set, even the same
    /// one after a Retry, starts it afresh.
    held: bool,
}

impl Opening {
    pub fn new(
        floor: Option<LauncherSceneContext>,
        reduced: bool,
        stage: Stage,
        now: Instant,
    ) -> Self {
        Self {
            floor,
            reduced,
            started: now,
            stage,
            stage_since: now,
            level_from: 0.0,
            level_since: now,
            intensity_from: stage.intensity(),
            breaths: 0.0,
            breath_since: now,
            held: false,
        }
    }

    pub fn floor(&self) -> Option<&LauncherSceneContext> {
        self.floor.as_ref()
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// The tumble runs only on the floor, with motion allowed.
    fn moving(&self) -> bool {
        self.floor.is_some() && !self.reduced
    }

    pub fn set_stage(&mut self, stage: Stage, now: Instant) {
        let held = std::mem::take(&mut self.held);
        if stage == self.stage {
            // Back from an error page, however long ago its last hold was
            // (a background tab gets no ticks), the wait starts now.
            if held {
                self.stage_since = now;
            }
            return;
        }
        self.level_from = self.level(now);
        self.intensity_from = self.intensity(now);
        self.breaths = self.breaths_at(now);
        self.breath_since = now;
        self.level_since = now;
        self.stage = stage;
        self.stage_since = now;
    }

    /// Restart the stall wait without changing the stage: while the loader
    /// shows an error, nothing is stalled.
    pub fn hold(&mut self, now: Instant) {
        self.stage_since = now;
        self.held = true;
    }

    fn since(&self, at: Instant, now: Instant) -> Duration {
        now.saturating_duration_since(at)
    }

    /// The tumble's eased progress.
    pub fn tumble(&self, now: Instant) -> f32 {
        if !self.moving() {
            return 1.0;
        }
        launcher_scene::ease(self.since(self.started, now).as_secs_f32() / TUMBLE.as_secs_f32())
    }

    /// The loading content's opacity as it fades in.
    pub fn content(&self, now: Instant) -> f32 {
        if self.reduced {
            return 1.0;
        }
        let t = self.since(self.started, now);
        let (delay, fade) = if self.floor.is_some() {
            (CONTENT_DELAY, CONTENT_FADE)
        } else {
            (Duration::ZERO, FRAME_FADE)
        };
        launcher_scene::ease(t.saturating_sub(delay).as_secs_f32() / fade.as_secs_f32())
    }

    /// `::before`: the steady light, easing to each stage's `--lvl`.
    fn level(&self, now: Instant) -> f32 {
        let to = self.stage.level();
        if self.reduced {
            return to;
        }
        let t = launcher_scene::ease(
            self.since(self.level_since, now).as_secs_f32() / LEVEL_FADE.as_secs_f32(),
        );
        self.level_from + (to - self.level_from) * t
    }

    /// The stage's intensity, easing from the last stage's like the level.
    fn intensity(&self, now: Instant) -> f32 {
        let to = self.stage.intensity();
        let t = launcher_scene::ease(
            self.since(self.level_since, now).as_secs_f32() / LEVEL_FADE.as_secs_f32(),
        );
        self.intensity_from + (to - self.intensity_from) * t
    }

    /// When the breath starts: after the tumble has begun on the floor.
    fn breath_start(&self) -> Instant {
        self.started
            + if self.floor.is_some() {
                Duration::from_millis(700)
            } else {
                Duration::from_millis(300)
            }
    }

    /// Breaths taken by `now`, each stage at its own speed.
    fn breaths_at(&self, now: Instant) -> f32 {
        let from = self.breath_since.max(self.breath_start());
        self.breaths + self.since(from, now).as_secs_f32() / self.stage.breath().as_secs_f32()
    }

    /// `::after`: dark until the breath starts, then a slow glow in and a
    /// breath that quickens and deepens with each stage; full on at Ready
    /// (`.peak`). Reduced motion has no animation, so no `::after`.
    fn pulse(&self, now: Instant) -> f32 {
        if self.reduced {
            return 0.0;
        }
        if self.stage == Stage::Ready {
            return 1.0;
        }
        let start = self.breath_start();
        if now <= start {
            return 0.0;
        }
        let glow_in =
            launcher_scene::ease(self.since(start, now).as_secs_f32() / GLOW_IN.as_secs_f32());
        let phase = self.breaths_at(now).fract();
        // Low at the start of each breath, full at its middle, each half
        // ease-in-out.
        let half = launcher_scene::bezier((phase * 2.0).min(2.0 - phase * 2.0), EASE_IN_OUT);
        let intensity = self.intensity(now);
        let low = 0.2 + 0.2 * intensity;
        let high = 0.6 + 0.4 * intensity;
        glow_in * (low + (high - low) * half)
    }

    pub(super) fn glow(&self, color: [u8; 3], now: Instant) -> Glow {
        Glow {
            color,
            level: self.level(now),
            pulse: self.pulse(now),
        }
    }

    /// A way out is offered once one stage has run past its wait: Cancel
    /// after [`STALL`], or going on without Spark after [`SPARK_WAIT`].
    pub fn stalled(&self, now: Instant) -> bool {
        self.stage != Stage::Ready && self.since(self.stage_since, now) >= self.stage.wait()
    }

    /// The opening Cube's pose.
    pub(super) fn pose(&self, now: Instant) -> Pose {
        Pose::opening(self.tumble(now))
    }

    /// The other Cubes on the page, stepping further back.
    pub(super) fn peer_pose(&self, now: Instant) -> Pose {
        Pose::deep(self.tumble(now))
    }

    /// Whether the App, once it exists, is drawn instead of the stage: as
    /// soon as the Cube is ready, with or without motion.
    pub fn shows_app(&self) -> bool {
        self.stage == Stage::Ready
    }

    /// The Cube is ready and the App shows: the opening can be dropped.
    pub fn done(&self) -> bool {
        self.shows_app()
    }

    /// Whether the next frame changes position or size, rather than only
    /// light: the tumble and the content's fade-in need every frame.
    pub fn needs_frames(&self, now: Instant) -> bool {
        self.moving() && self.since(self.started, now) < TUMBLE + CONTENT_DELAY + CONTENT_FADE
    }

    /// The three progress dots under the quote (`P-Opening` `.dots`): each
    /// pulses from 25% to 95% ink over 1.5 s, half a second after the one
    /// before it. Reduced motion holds the first lit and the others dim.
    pub fn dots(&self, now: Instant) -> [f32; 3] {
        if self.reduced {
            return [DOT_LIT, DOT_DIM, DOT_DIM];
        }
        let t = self.since(self.started, now).as_secs_f32();
        std::array::from_fn(|i| {
            let phase = ((t - DOT_DELAY * i as f32) / DOT_PERIOD).rem_euclid(1.0);
            // Keyframes: dim at 0% and 100%, brightest at 33%.
            let rise = if phase < 1.0 / 3.0 {
                phase * 3.0
            } else {
                (1.0 - phase) * 1.5
            };
            DOT_DIM + (DOT_PEAK - DOT_DIM) * launcher_scene::ease(rise)
        })
    }

    /// Whether the light still changes on its own: the flicker and breath.
    pub fn breathes(&self) -> bool {
        !self.reduced
    }
}

/// The progress dots' opacities and timing (`P-Opening` `.dots`).
const DOT_DIM: f32 = 0.25;
const DOT_PEAK: f32 = 0.95;
const DOT_LIT: f32 = 0.9;
const DOT_PERIOD: f32 = 1.5;
const DOT_DELAY: f32 = 0.5;

/// How long each loading quote stays before the next one replaces it.
const QUOTE_HOLD: Duration = Duration::from_secs(8);
/// The outgoing quote fades out while the next one fades in over this long.
const QUOTE_FADE: Duration = Duration::from_millis(1200);

/// The loading face's quotes: a new one every [`QUOTE_HOLD`], crossfading
/// from the last. Under reduced motion they swap without the fade.
#[derive(Debug, Clone)]
pub(crate) struct QuoteCycle {
    current: Quote,
    outgoing: Option<Quote>,
    since: Instant,
    provider: QuoteProvider,
    reduced: bool,
}

impl QuoteCycle {
    pub fn new(first: Quote, reduced: bool, now: Instant) -> Self {
        let mut provider = QuoteProvider::new();
        provider.remember(&first);
        Self {
            current: first,
            outgoing: None,
            since: now,
            provider,
            reduced,
        }
    }

    /// Move on to the next quote once the current one has had its time.
    pub fn advance(&mut self, now: Instant) {
        if now.saturating_duration_since(self.since) < QUOTE_HOLD {
            return;
        }
        let next = self.provider.select("loading");
        let last = std::mem::replace(&mut self.current, next);
        self.outgoing = (!self.reduced).then_some(last);
        self.since = now;
    }

    pub fn current(&self) -> &Quote {
        &self.current
    }

    /// The quote fading out and how far the crossfade has gone (0 to 1),
    /// while one is under way.
    pub fn outgoing(&self, now: Instant) -> Option<(&Quote, f32)> {
        let t = now.saturating_duration_since(self.since).as_secs_f32() / QUOTE_FADE.as_secs_f32();
        let t = t.clamp(0.0, 1.0);
        self.outgoing
            .as_ref()
            .filter(|_| t < 1.0)
            .map(|quote| (quote, t * t * (3.0 - 2.0 * t)))
    }

    /// Whether a crossfade is under way and needs every frame.
    pub fn needs_frames(&self, now: Instant) -> bool {
        self.outgoing(now).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainId;

    fn floor() -> LauncherSceneContext {
        LauncherSceneContext {
            selected: super::super::CubeCatalogKey {
                chain: ChainId::Bitcoin,
                cube_id: "everyday".into(),
            },
            page: 0,
            selected_slot: 0,
            backup: "Status unknown",
            connect: String::new(),
            wallets: "Not yet known".into(),
            peers: Vec::new(),
            pages: 1,
            chrome: Default::default(),
        }
    }

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn loading_quotes_change_every_eight_seconds_with_a_crossfade() {
        use coincube_ui::component::quote_display::random_quote;
        let t0 = Instant::now();
        let first = random_quote("loading");
        let mut quotes = QuoteCycle::new(first.clone(), false, t0);
        quotes.advance(t0 + ms(7_900));
        assert_eq!(quotes.current().id, first.id);
        assert!(quotes.outgoing(t0 + ms(7_900)).is_none());

        let t8 = t0 + QUOTE_HOLD;
        quotes.advance(t8);
        assert_ne!(
            quotes.current().id,
            first.id,
            "never the same quote twice in a row"
        );
        let (outgoing, t) = quotes.outgoing(t8 + ms(600)).expect("crossfading");
        assert_eq!(outgoing.id, first.id);
        assert!(t > 0.0 && t < 1.0);
        assert!(quotes.needs_frames(t8 + ms(600)));
        assert!(quotes.outgoing(t8 + QUOTE_FADE).is_none());
        assert!(!quotes.needs_frames(t8 + QUOTE_FADE));
    }

    #[test]
    fn reduced_motion_swaps_loading_quotes_without_a_fade() {
        use coincube_ui::component::quote_display::random_quote;
        let t0 = Instant::now();
        let first = random_quote("loading");
        let mut quotes = QuoteCycle::new(first.clone(), true, t0);
        quotes.advance(t0 + QUOTE_HOLD);
        assert_ne!(quotes.current().id, first.id);
        assert!(quotes.outgoing(t0 + QUOTE_HOLD).is_none());
    }

    #[test]
    fn the_cube_tumbles_then_its_content_fades_in_and_its_edge_glows_in_slowly() {
        let t0 = Instant::now();
        let opening = Opening::new(Some(floor()), false, Stage::Connecting, t0);
        assert_eq!(opening.tumble(t0), 0.0);
        assert_eq!(opening.content(t0 + ms(500)), 0.0);
        assert!(opening.content(t0 + ms(900)) > 0.0 && opening.content(t0 + ms(900)) < 1.0);
        assert!((opening.tumble(t0 + TUMBLE) - 1.0).abs() < 1e-6);
        assert!((opening.content(t0 + ms(1200)) - 1.0).abs() < 1e-6);
        // `::after` waits 700ms, then glows in with no flicker: it only
        // rises over its first 400ms.
        assert_eq!(opening.pulse(t0 + ms(700)), 0.0);
        let first: Vec<f32> = (1..=40)
            .map(|i| opening.pulse(t0 + ms(700 + i * 10)))
            .collect();
        assert!(
            first.windows(2).all(|pair| pair[1] >= pair[0]) && first[0] < 0.01 && first[39] < 0.25,
            "{:?}",
            first
        );
        // Then it breathes softly at the first stage: never dark, never full.
        let breaths: Vec<f32> = (0..60)
            .map(|i| opening.pulse(t0 + ms(2400 + i * 50)))
            .collect();
        assert!(breaths
            .iter()
            .all(|p| (0.2 - 1e-4..=0.6 + 1e-4).contains(p)));
        assert!(breaths.iter().any(|p| *p > 0.59), "{:?}", breaths);
        // `::before` eases from 0 to the first stage's level.
        assert!((opening.level(t0 + ms(2000)) - 0.35).abs() < 1e-6);
    }

    #[test]
    fn each_stage_brightens_the_edge_and_a_held_stage_offers_cancel() {
        let t0 = Instant::now();
        let mut opening = Opening::new(Some(floor()), false, Stage::Connecting, t0);
        let mut levels = Vec::new();
        for (i, stage) in [
            Stage::StartingDaemon,
            Stage::Syncing,
            Stage::AlmostThere,
            Stage::Ready,
        ]
        .iter()
        .copied()
        .enumerate()
        {
            let at = t0 + ms(2000 * (i as u64 + 1));
            opening.set_stage(stage, at);
            levels.push(opening.level(at + LEVEL_FADE));
        }
        assert_eq!(levels, vec![0.5, 0.65, 0.8, 1.0]);
        assert_eq!(opening.pulse(t0 + ms(9000)), 1.0, "Ready is .peak");

        let mut held = Opening::new(Some(floor()), false, Stage::Connecting, t0);
        held.set_stage(Stage::Syncing, t0 + ms(1000));
        assert!(!held.stalled(t0 + ms(1000) + STALL - ms(1)));
        assert!(held.stalled(t0 + ms(1000) + STALL));
        held.set_stage(Stage::AlmostThere, t0 + ms(20_000));
        assert!(
            !held.stalled(t0 + ms(20_000)),
            "a new stage restarts the wait"
        );
    }

    #[test]
    fn the_app_shows_as_soon_as_the_cube_is_ready_with_motion_or_without() {
        let t0 = Instant::now();
        let ready = t0 + ms(3000);
        for (floor, reduced) in [(Some(floor()), false), (None, false), (None, true)] {
            let mut opening = Opening::new(floor, reduced, Stage::Syncing, t0);
            assert!(!opening.shows_app() && !opening.done());
            opening.set_stage(Stage::Ready, ready);
            assert!(opening.shows_app(), "no hold before the App");
            assert!(opening.done(), "and nothing left to play");
            assert!(!opening.needs_frames(ready));
        }
        // A Cube ready at once (no Vault to load) does not wait either.
        let instant = Opening::new(Some(floor()), false, Stage::Ready, t0);
        assert!(instant.shows_app() && instant.done());
    }

    #[test]
    fn without_the_floor_the_content_opens_full_screen_with_no_tumble() {
        let t0 = Instant::now();
        let mut opening = Opening::new(None, false, Stage::Connecting, t0);
        assert_eq!(opening.tumble(t0), 1.0);
        assert!((opening.content(t0 + FRAME_FADE) - 1.0).abs() < 1e-6);
        assert_eq!(opening.pulse(t0 + ms(300)), 0.0);
        assert!(opening.pulse(t0 + ms(1000)) > 0.0);
        opening.set_stage(Stage::Ready, t0 + ms(4000));
        assert!(opening.shows_app() && opening.done());
        assert!(!opening.needs_frames(t0));

        let mut reduced = Opening::new(None, true, Stage::Connecting, t0);
        assert_eq!(reduced.content(t0), 1.0);
        assert_eq!(reduced.level(t0), 0.35, "no fade under reduced motion");
        assert_eq!(reduced.pulse(t0 + ms(800)), 0.0, "no flicker or breath");
        assert!(!reduced.breathes());
        reduced.set_stage(Stage::Syncing, t0 + ms(10));
        assert_eq!(reduced.level(t0 + ms(10)), 0.65);
    }

    #[test]
    fn a_held_error_restarts_the_stall_wait() {
        let t0 = Instant::now();
        let mut opening = Opening::new(Some(floor()), false, Stage::Connecting, t0);
        // An error page shows from 3s to 40s; Retry comes back to Connecting.
        opening.hold(t0 + ms(40_000));
        opening.set_stage(Stage::Connecting, t0 + ms(40_000));
        assert!(!opening.stalled(t0 + ms(40_000)));
        assert!(opening.stalled(t0 + ms(40_000) + STALL));
        // A last hold long before the Retry (no ticks in the background)
        // still restarts the wait at the Retry itself.
        opening.hold(t0 + ms(60_000));
        opening.set_stage(Stage::Connecting, t0 + ms(200_000));
        assert!(!opening.stalled(t0 + ms(200_000)));
        assert!(opening.stalled(t0 + ms(200_000) + STALL));
        // Without a hold, the same stage keeps its wait.
        opening.set_stage(Stage::Connecting, t0 + ms(210_000));
        assert!(opening.stalled(t0 + ms(200_000) + STALL));
    }

    #[test]
    fn waiting_on_spark_offers_to_go_on_without_it_after_its_own_wait() {
        let t0 = Instant::now();
        let mut opening = Opening::new(Some(floor()), false, Stage::AlmostThere, t0);
        let spark = t0 + ms(2000);
        opening.set_stage(Stage::Spark, spark);
        // Spark gets its own, longer wait than the other stages' Cancel.
        assert!(!opening.stalled(spark + STALL));
        assert!(!opening.stalled(spark + SPARK_WAIT - ms(1)));
        assert!(opening.stalled(spark + SPARK_WAIT));
        assert!(!opening.shows_app(), "the face waits for Spark");
        assert!(
            opening.level(spark + LEVEL_FADE) > Stage::AlmostThere.level(),
            "Spark is the brightest stage before Ready"
        );
        let ready = spark + SPARK_WAIT + ms(1000);
        opening.set_stage(Stage::Ready, ready);
        assert!(!opening.stalled(ready));
    }

    #[test]
    fn a_retry_after_ready_starts_the_stages_over() {
        let t0 = Instant::now();
        let mut opening = Opening::new(Some(floor()), false, Stage::AlmostThere, t0);
        opening.set_stage(Stage::Ready, t0 + ms(100));
        opening.set_stage(Stage::Connecting, t0 + ms(200));
        assert!(!opening.shows_app() && !opening.done());
    }

    /// The time between the first two rises of the breath through its middle
    /// after `from`.
    fn breath_period(opening: &Opening, from: Instant) -> Duration {
        let samples: Vec<(Instant, f32)> = (0..800)
            .map(|i| {
                let at = from + ms(i * 10);
                (at, opening.pulse(at))
            })
            .collect();
        let (low, high) = samples
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), (_, p)| {
                (lo.min(*p), hi.max(*p))
            });
        let middle = (low + high) / 2.0;
        let rises: Vec<Instant> = samples
            .windows(2)
            .filter(|w| w[0].1 < middle && w[1].1 >= middle)
            .map(|w| w[1].0)
            .collect();
        rises[1] - rises[0]
    }

    #[test]
    fn the_breath_quickens_and_deepens_as_the_load_goes_on() {
        let t0 = Instant::now();
        let mut opening = Opening::new(Some(floor()), false, Stage::Connecting, t0);
        let slow = breath_period(&opening, t0 + ms(3000));
        let soft = (0..300)
            .map(|i| opening.pulse(t0 + ms(3000 + i * 10)))
            .fold(0.0_f32, f32::max);

        let at = t0 + ms(12_000);
        let before = opening.pulse(at);
        opening.set_stage(Stage::AlmostThere, at);
        assert!(
            (opening.pulse(at) - before).abs() < 1e-3,
            "the breath carries on from where it was"
        );
        let settled = at + LEVEL_FADE;
        let fast = breath_period(&opening, settled);
        let bright = (0..200)
            .map(|i| opening.pulse(settled + ms(i * 10)))
            .fold(0.0_f32, f32::max);

        assert!(slow > ms(2500) && slow < ms(2700), "{:?}", slow);
        // Three of the four steps toward the fastest breath, Spark's.
        assert!(fast > ms(1400) && fast < ms(1550), "{:?}", fast);
        assert!(bright > soft + 0.25, "{} vs {}", bright, soft);

        opening.set_stage(Stage::Spark, t0 + ms(30_000));
        let fastest = breath_period(&opening, t0 + ms(30_000) + LEVEL_FADE);
        assert!(fastest > ms(1000) && fastest < ms(1200), "{:?}", fastest);
    }

    /// P-Opening's three dots: one after another, half a second apart,
    /// between 25% and 95%; under reduced motion the first stays lit.
    #[test]
    fn progress_dots_pulse_in_turn_and_hold_under_reduced_motion() {
        let t0 = Instant::now();
        let reduced = Opening::new(None, true, Stage::Connecting, t0);
        assert_eq!(reduced.dots(t0 + ms(700)), [0.9, 0.25, 0.25]);

        let opening = Opening::new(None, false, Stage::Connecting, t0);
        let close = |a: f32, b: f32| (a - b).abs() < 0.01;
        let start = opening.dots(t0);
        assert!(close(start[0], 0.25), "{:?}", start);
        let first_peak = opening.dots(t0 + ms(500));
        assert!(close(first_peak[0], 0.95), "{:?}", first_peak);
        let second_peak = opening.dots(t0 + ms(1000));
        assert!(close(second_peak[1], 0.95), "{:?}", second_peak);
        assert!(second_peak[0] < 0.95 && second_peak[2] < 0.95);
        for step in 0..60 {
            for dot in opening.dots(t0 + ms(step * 50)) {
                assert!((0.25..=0.951).contains(&dot), "{}", dot);
            }
        }
    }
}
