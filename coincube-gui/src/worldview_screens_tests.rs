//! Desktop V3 Phase 1 (V6): the screens that used to draw Worldview artwork
//! now show text only. Each is mounted in every worldview, both themes and
//! both languages (invariant R8): the quote stays (R7), the worldview's
//! wording stays (R2), and nothing else replaces the picture: every screen
//! is drawn with the software renderer and must queue no image or SVG. The
//! Opening paths are the exception: they draw the line Cube (an SVG) above
//! the quote, per the brand brief of 2026-11-26, and still no raster art.

use coincube_ui::component::{
    amount::BitcoinDisplayUnit,
    payment_result::{payment_result, Outcome},
    quote_display::{self, Quote},
};
use coincube_ui::locale::{render_in, Language};
use coincube_ui::theme::{Theme, WorldviewMode};

use crate::app::view::settings::page::flow_intro;
use crate::app::view::shared::landing::capture;
use crate::app::view::spark::{SparkTransactionsStatus, SparkTransactionsView};
use crate::chain::ChainId;
use crate::home::launcher_presenter::{opening_body, project_opening};
use crate::home::restore_frame::done_mark;

/// Deliberately tall: iced's `Column` skips drawing children outside the
/// viewport, so a picture below a 1024×768 window's fold would go uncounted.
/// At 3000 px high (the window less the 100 px the helper keeps for chrome)
/// each whole screen is drawn, so `draws_no_image` covers all of it.
const TALL: (u32, u32) = (1024, 3100);

fn fixture_quote(id: &str) -> Quote {
    Quote {
        id: id.into(),
        text: format!("Fixture quote {id}"),
        author: "Author".into(),
        source: "Source".into(),
        category: "fixture".into(),
        length: "short".into(),
        contexts: Vec::new(),
    }
}

#[test]
fn quote_screens_show_text_only_in_every_worldview_theme_and_language() {
    let loading = fixture_quote("loading");
    let empty = fixture_quote("empty");
    let error = fixture_quote("error");
    let sent = fixture_quote("sent");
    let received = fixture_quote("received");
    let startup_error = crate::loader::Step::Error(Box::new(crate::loader::Error::Unexpected(
        "Fixture startup error".into(),
    )));
    for language in [Language::English, Language::Spanish] {
        for (mode, name) in [
            (WorldviewMode::Samurai, "samurai"),
            (WorldviewMode::Suitcoiner, "suitcoiner"),
            (WorldviewMode::Luke, "luke"),
        ] {
            for (appearance, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
                let theme = theme.with_worldview(mode);
                let scenario =
                    |screen: &str| format!("v6-{language:?}-{name}-{appearance}-{screen}");

                let opened = capture::inspect_rendered_filled(
                    &scenario("opening"),
                    render_in(language, || {
                        coincube_ui::worldview_copy::render_in(mode, || {
                            opening_body(
                                project_opening("Everyday", ChainId::Bitcoin),
                                &crate::loader::Step::Connecting,
                                &loading,
                                false,
                            )
                        })
                    }),
                    (1024, 768),
                    appearance,
                    theme,
                );
                assert!(opened.contains(&loading.text), "{}", scenario("opening"));
                assert!(opened.contains("— Author, Source"));
                // The Opening rail above the stage draws the brand mark and
                // the network coin, which are not artwork (R9); the loading
                // stage under it is the Cube loader: the quote and no picture,
                // not even the line Cube (Robert, 2026-11-27).
                let stage = capture::inspect_rendered_filled(
                    &scenario("opening-stage"),
                    render_in(language, || {
                        coincube_ui::worldview_copy::render_in(mode, || {
                            crate::loader::view(&crate::loader::Step::Connecting, &loading)
                        })
                    }),
                    (1024, 768),
                    appearance,
                    theme,
                );
                assert!(
                    stage.contains(&loading.text),
                    "{}",
                    scenario("opening-stage")
                );
                assert_eq!(
                    stage.drawn_images(),
                    capture::DrawnImages {
                        raster: 0,
                        vector: 0
                    },
                    "{}",
                    scenario("opening-stage")
                );

                let failed = capture::inspect_rendered_filled(
                    &scenario("startup-error"),
                    render_in(language, || crate::loader::view(&startup_error, &error)),
                    (1024, 768),
                    appearance,
                    theme,
                );
                let headline = render_in(language, || {
                    coincube_ui::locale::Text::TenshuFailedToStart.get()
                });
                assert!(failed.contains(headline), "{}", scenario("startup-error"));
                assert!(failed.contains(&error.text));
                assert!(failed.contains("Fixture startup error"));
                assert_eq!(
                    failed.drawn_images().raster,
                    0,
                    "{}",
                    scenario("startup-error")
                );

                let history = capture::inspect_rendered_full_window(
                    &scenario("empty-history"),
                    render_in(language, || {
                        SparkTransactionsView {
                            status: SparkTransactionsStatus::Loaded(vec![]),
                            recent_transactions: &[],
                            fiat_converter: None,
                            bitcoin_unit: BitcoinDisplayUnit::Sats,
                            show_direction_badges: false,
                            empty_state_quote: &empty,
                            current_page: 0,
                            is_last_page: true,
                            processing: false,
                            mask_amounts: true,
                        }
                        .render()
                    }),
                    TALL,
                    appearance,
                    theme,
                );
                assert!(history.contains(&empty.text));
                let sentence = render_in(language, || {
                    coincube_ui::locale::Text::SparkHistoryEmpty.get()
                });
                assert!(history.contains(sentence), "{}", scenario("empty-history"));
                assert!(history.draws_no_image(), "{}", scenario("empty-history"));

                use coincube_ui::worldview_copy::VoiceLine;
                for (screen, outcome, quote, voice) in [
                    ("sent", Outcome::Sent, &sent, VoiceLine::PaymentSent),
                    (
                        "received",
                        Outcome::Received,
                        &received,
                        VoiceLine::PaymentReceived,
                    ),
                ] {
                    let line = render_in(language, || {
                        coincube_ui::worldview_copy::render_in(mode, || voice.get())
                    });
                    let page = capture::inspect_rendered_full_window(
                        &scenario(screen),
                        render_in(language, || {
                            coincube_ui::worldview_copy::render_in(mode, || {
                                payment_result(outcome, "50,000 sats", quote, ())
                                    .voice(line)
                                    .into()
                            })
                        }),
                        TALL,
                        appearance,
                        theme,
                    );
                    assert!(page.contains("50,000"), "{}", scenario(screen));
                    assert!(page.contains(&quote.text), "{}", scenario(screen));
                    assert!(page.draws_no_image(), "{}", scenario(screen));
                    if let Some(line) = line {
                        assert!(page.contains(line), "{}: {}", scenario(screen), line);
                    }
                }

                let restored = capture::inspect_rendered_full_window(
                    &scenario("restored"),
                    done_mark::<()>(),
                    TALL,
                    appearance,
                    theme,
                );
                // The plain check is one glyph, and nothing else is drawn.
                assert_eq!(restored.text.len(), 1, "{}", scenario("restored"));
                assert!(restored.draws_no_image(), "{}", scenario("restored"));

                let flow = capture::inspect_rendered_full_window(
                    &scenario("recovery-kit"),
                    flow_intro::<()>("Protect your Cube", "Set up a Recovery Kit.").into(),
                    TALL,
                    appearance,
                    theme,
                );
                assert!(flow.contains("Protect your Cube"));
                assert!(flow.contains("Set up a Recovery Kit."));
                assert_eq!(flow.text.len(), 2, "{}", scenario("recovery-kit"));
                assert!(flow.draws_no_image(), "{}", scenario("recovery-kit"));
            }
        }
    }
}

/// The probe behind `draws_no_image` sees a picture when there is one, so the
/// text-only test above cannot pass by not looking.
#[test]
fn the_image_probe_counts_a_drawn_picture() {
    let picture = iced::widget::image(iced::widget::image::Handle::from_rgba(2, 2, vec![255; 16]))
        .width(iced::Length::Fixed(40.0))
        .height(iced::Length::Fixed(40.0));
    let drawn = capture::inspect_rendered_full_window::<()>(
        "image-probe",
        coincube_ui::widget::Container::new(picture).into(),
        (1024, 768),
        "dark",
        Theme::dark(),
    );
    assert_eq!(
        drawn.images,
        Some(capture::DrawnImages {
            raster: 1,
            vector: 0
        })
    );
    assert!(!drawn.draws_no_image());

    // Below a 768 px window's fold the picture is skipped, which is why the
    // screens above are drawn `TALL`; there it is counted.
    let below_fold = || -> coincube_ui::widget::Element<'static, ()> {
        coincube_ui::widget::Column::new()
            .push(iced::widget::Space::new().height(iced::Length::Fixed(2000.0)))
            .push(
                iced::widget::image(iced::widget::image::Handle::from_rgba(2, 2, vec![255; 16]))
                    .width(iced::Length::Fixed(40.0))
                    .height(iced::Length::Fixed(40.0)),
            )
            .into()
    };
    let short = capture::inspect_rendered_full_window(
        "image-probe-short",
        below_fold(),
        (1024, 768),
        "dark",
        Theme::dark(),
    );
    assert!(short.draws_no_image());
    let tall = capture::inspect_rendered_full_window(
        "image-probe-tall",
        below_fold(),
        TALL,
        "dark",
        Theme::dark(),
    );
    assert_eq!(tall.drawn_images().raster, 1);
}

#[test]
fn spanish_samurai_wallet_actions_and_errors_render_together() {
    use coincube_ui::component::notification;

    let quote = quote_display::random_quote("empty-wallet");
    let error = crate::user_error::UserError::new(
        "That fee rate isn't valid",
        "Enter a whole number of sats per vByte greater than zero.",
        "CC-XFER-INPUT",
        false,
    );
    let view = render_in(Language::Spanish, || {
        SparkTransactionsView {
            status: SparkTransactionsStatus::Loaded(vec![]),
            recent_transactions: &[],
            fiat_converter: None,
            bitcoin_unit: BitcoinDisplayUnit::Sats,
            show_direction_badges: false,
            empty_state_quote: &quote,
            current_page: 0,
            is_last_page: true,
            processing: false,
            mask_amounts: true,
        }
        .render()
    });
    let activity = capture::inspect("spark-history-es", view, (1024, 768), "dark", Theme::dark());
    assert!(activity.contains("Tu cartera Spark está lista"));
    assert!(activity.contains("Enviar"));
    assert!(activity.contains("Recibir sats"));

    let error_card: coincube_ui::widget::Element<'_, ()> = render_in(Language::Spanish, || {
        notification::error_card(error.title, error.guidance, error.reference, None).into()
    });
    let rendered = capture::inspect(
        "transfer-error-es",
        error_card,
        (1024, 768),
        "dark",
        Theme::dark(),
    );
    assert!(rendered.contains("La tasa de comisión no es válida"));
    assert!(rendered.contains("Ingresa un número entero de sats"));
    assert!(rendered.contains("CC-XFER-INPUT"));
}

/// Desktop V3 Phase 1 (V7): a mode switch changes the network names and the
/// voice line, and no colour. The accent, its ink, the whole palette and the
/// rule over the Overview's voice line are the same in all three worldviews,
/// in both themes.
#[test]
fn a_mode_switch_changes_network_names_and_voice_lines_and_no_colour() {
    use coincube_ui::component::tenshu::worldview_hero::voice_rule_color;
    use coincube_ui::worldview_copy::VoiceLine;
    use std::collections::BTreeSet;

    let modes = [
        WorldviewMode::Samurai,
        WorldviewMode::Suitcoiner,
        WorldviewMode::Luke,
    ];
    for base in [Theme::dark(), Theme::light()] {
        let mut names = BTreeSet::new();
        let mut lines = BTreeSet::new();
        for mode in modes {
            let theme = base.with_worldview(mode);
            assert_eq!(theme.colors.shell.accent, base.colors.shell.accent);
            assert_eq!(theme.colors.shell.accent_ink, base.colors.shell.accent_ink);
            assert_eq!(theme.colors, base.colors, "{mode:?}: no colour changes");
            assert_eq!(voice_rule_color(&theme), voice_rule_color(&base));

            names.insert((
                crate::chain::worldview_chain_label(ChainId::Bitcoin, mode),
                crate::chain::worldview_chain_label(ChainId::BitcoinBlake2b, mode),
            ));
            lines.insert(VoiceLine::Overview.for_mode(mode));
        }
        assert_eq!(
            names.len(),
            modes.len(),
            "each mode names the networks its way"
        );
        assert_eq!(lines.len(), modes.len(), "each mode has its own voice line");
    }
}

/// The Cube loader shows the quote and no picture (Robert, 2026-11-27): the
/// Launcher's Opening (the rail over the loader), the loader on its own
/// (connecting, starting, scanning and its error) and `loader::opening_quote`.
/// No raster image (the ink art was raster) and no SVG beyond the Opening
/// rail's own bar icons, so the line Cube is gone too. The passkey Opening
/// draws the turning Cube on a canvas and has its own tests in
/// `passkey_unlock`. Counted from what the software renderer actually drew,
/// so a picture cannot hide in a view the text probe passes over.
#[test]
fn every_opening_shows_the_quote_and_no_picture() {
    let quote = quote_display::random_quote("loading");
    let words = quote.localized_text().to_owned();
    let error = crate::loader::Step::Error(Box::new(crate::loader::Error::Unexpected(
        "Fixture startup error".into(),
    )));
    let steps = [
        ("connecting", crate::loader::Step::Connecting),
        (
            "starting",
            crate::loader::Step::StartingDaemon { progress: 0.4 },
        ),
        ("scanning", crate::loader::Step::FullScan { progress: 0.4 }),
    ];
    for (mode, name) in [
        (WorldviewMode::Samurai, "samurai"),
        (WorldviewMode::Suitcoiner, "suitcoiner"),
        (WorldviewMode::Luke, "luke"),
    ] {
        for (appearance, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
            let theme = theme.with_worldview(mode);
            let rail_icons = capture::inspect_rendered_filled(
                &format!("opening-quote-{name}-rail"),
                crate::home::launcher_presenter::opening_rail::<()>(&project_opening(
                    "Everyday",
                    ChainId::Bitcoin,
                )),
                (1024, 768),
                appearance,
                theme,
            )
            .drawn_images()
            .vector;
            let launcher = capture::inspect_rendered_filled(
                &format!("opening-quote-{name}-launcher"),
                opening_body(
                    project_opening("Everyday", ChainId::Bitcoin),
                    &crate::loader::Step::Connecting,
                    &quote,
                    false,
                ),
                (1024, 768),
                appearance,
                theme,
            );
            assert!(
                launcher.contains(&words),
                "{} {}: no quote",
                name,
                appearance
            );
            let drawn = launcher.drawn_images();
            assert_eq!(
                drawn.raster, 0,
                "{} {} launcher: raster art",
                name, appearance
            );
            assert_eq!(
                drawn.vector, rail_icons,
                "{} {} launcher: a picture beyond the rail's icons",
                name, appearance
            );
            let mut views: Vec<(String, capture::Inspection)> = vec![(
                "opening-quote".into(),
                capture::inspect_rendered_filled(
                    &format!("opening-quote-{name}-quote"),
                    crate::loader::opening_quote::<()>(&quote),
                    (1024, 768),
                    appearance,
                    theme,
                ),
            )];
            for (step_name, step) in &steps {
                views.push((
                    format!("loader-{step_name}"),
                    capture::inspect_rendered_filled(
                        &format!("opening-quote-{name}-loader-{step_name}"),
                        crate::loader::view(step, &quote),
                        (1024, 768),
                        appearance,
                        theme,
                    ),
                ));
            }
            let failed = capture::inspect_rendered_filled(
                &format!("opening-quote-{name}-loader-error"),
                crate::loader::view(&error, &quote),
                (1024, 768),
                appearance,
                theme,
            );
            assert!(failed.contains("COINCUBE Desktop failed to start"));
            views.push(("loader-error".into(), failed));
            for (view, inspection) in &views {
                assert!(
                    inspection.contains(&words),
                    "{} {} {}: no quote",
                    name,
                    appearance,
                    view
                );
                let drawn = inspection.drawn_images();
                assert_eq!(
                    drawn.raster, 0,
                    "{} {} {}: raster art",
                    name, appearance, view
                );
                assert_eq!(
                    drawn.vector, 0,
                    "{} {} {}: an SVG picture",
                    name, appearance, view
                );
            }
        }
    }
}

/// The counter behind the Opening test sees a drawn raster image and a drawn
/// SVG, so that test cannot pass because nothing is counted.
#[test]
fn the_drawn_image_counter_sees_raster_and_svg() {
    let pixels = vec![255u8; 4 * 4 * 4];
    let picture: coincube_ui::widget::Element<'_, ()> = iced::widget::column![
        iced::widget::image(iced::widget::image::Handle::from_rgba(4, 4, pixels))
            .width(40)
            .height(40),
        coincube_ui::component::tenshu::top_bar::display_cube(64.0, 1.0),
    ]
    .into();
    let drawn = capture::inspect_rendered_filled(
        "drawn-image-counter-probe",
        picture,
        (1024, 768),
        "dark",
        Theme::dark(),
    )
    .drawn_images();
    assert_eq!(drawn.raster, 1);
    assert_eq!(drawn.vector, 1);
}
