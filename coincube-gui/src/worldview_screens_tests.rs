//! Desktop V3 Phase 1 (V6): the screens that used to draw Worldview artwork
//! now show text only. Each is mounted in every worldview, both themes and
//! both languages (invariant R8): the quote stays (R7), the worldview's
//! wording stays (R2), and nothing else replaces the picture: every screen
//! is drawn with the software renderer and must queue no image or SVG. The
//! Cube loader (the Opening and the loader) also turns the line Cube above
//! its quote (Robert, 2026-11-27); that Cube is a canvas, which the counter
//! does not see, so its own test finds it by its container.

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
                // stage under it is the Cube loader: the turning Cube (a
                // canvas, see `every_cube_loader_turns_the_cube_above_the_
                // quote_and_status`), the quote and the status, and no image.
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
                    failed.drawn_images(),
                    capture::DrawnImages {
                        raster: 0,
                        vector: 0
                    },
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

/// The Cube loader (Robert, 2026-11-27): "spinning Cube, then the quote below
/// that, then the status text", one centred group. Checked on the Launcher's
/// Opening (the rail over the loader) and on every loader step, the error
/// included, in every worldview, theme and language. The Cube is a canvas,
/// which the drawn-image counter does not see, so it is found and placed by
/// its container (`capture::Inspection::assert_cube_loader`); the counter
/// still shows no raster art and no SVG beyond the Opening rail's own bar
/// icons. The Launcher's opening face is checked in `launcher_presenter` and
/// the passkey unlock in `gui::tab`.
#[test]
fn every_cube_loader_turns_the_cube_above_the_quote_and_status() {
    let quote = fixture_quote("loading");
    let attribution = "\u{2014} Author, Source";
    let error = crate::loader::Step::Error(Box::new(crate::loader::Error::Unexpected(
        "Fixture startup error".into(),
    )));
    // Never connected: the Syncing step only needs a daemon handle to hold.
    let daemon = || -> std::sync::Arc<dyn crate::daemon::Daemon + Sync + Send> {
        std::sync::Arc::new(crate::daemon::client::Coincubed::new(
            crate::daemon::client::jsonrpc::JsonRPCClient::new("/nonexistent/fixture.sock"),
        ))
    };
    let steps = [
        crate::loader::Step::Connecting,
        crate::loader::Step::StartingDaemon { progress: 0.4 },
        crate::loader::Step::FullScan { progress: 0.4 },
        crate::loader::Step::Syncing {
            daemon: daemon(),
            progress: 0.4,
            bitcoind_logs: "Fixture node log".into(),
        },
        crate::loader::Step::Syncing {
            daemon: daemon(),
            progress: 0.99,
            bitcoind_logs: String::new(),
        },
        error,
    ];
    for language in [Language::English, Language::Spanish] {
        for (mode, name) in [
            (WorldviewMode::Samurai, "samurai"),
            (WorldviewMode::Suitcoiner, "suitcoiner"),
            (WorldviewMode::Luke, "luke"),
        ] {
            for (appearance, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
                let theme = theme.with_worldview(mode);
                let scenario = |screen: &str| format!("cube-loader-{language:?}-{name}-{screen}");
                let rail_icons = capture::inspect_rendered_filled(
                    &scenario("rail"),
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
                for (index, step) in steps.iter().enumerate() {
                    let status = render_in(language, || match step {
                        crate::loader::Step::Connecting => {
                            coincube_ui::locale::Text::ConnectingToDaemon.get()
                        }
                        crate::loader::Step::StartingDaemon { .. } => {
                            coincube_ui::locale::Text::StartingDaemon.get()
                        }
                        crate::loader::Step::FullScan { .. } => {
                            coincube_ui::locale::Text::ScanningTheBlockchain.get()
                        }
                        crate::loader::Step::Syncing { progress, .. } if *progress > 0.98 => {
                            coincube_ui::locale::Text::AlmostThere.get()
                        }
                        crate::loader::Step::Syncing { .. } => {
                            coincube_ui::locale::Text::SyncingBlockchain.get()
                        }
                        crate::loader::Step::Error(_) => {
                            coincube_ui::locale::Text::TenshuFailedToStart.get()
                        }
                    });
                    let screens = [
                        (
                            "loader",
                            0,
                            render_in(language, || {
                                coincube_ui::worldview_copy::render_in(mode, || {
                                    crate::loader::view(step, &quote)
                                })
                            }),
                        ),
                        (
                            "launcher-opening",
                            rail_icons,
                            render_in(language, || {
                                coincube_ui::worldview_copy::render_in(mode, || {
                                    opening_body(
                                        project_opening("Everyday", ChainId::Bitcoin),
                                        step,
                                        &quote,
                                        false,
                                    )
                                })
                            }),
                        ),
                    ];
                    for (screen, icons, view) in screens {
                        let at = scenario(&format!("{screen}-{index}-{appearance}"));
                        let shown = capture::inspect_rendered_filled(
                            &at,
                            view,
                            (1024, 900),
                            appearance,
                            theme,
                        );
                        shown.assert_cube_loader(
                            &at,
                            crate::loader::CUBE_LOADER_SIZE,
                            &quote.text,
                            attribution,
                            status,
                        );
                        assert_eq!(
                            shown.drawn_images(),
                            capture::DrawnImages {
                                raster: 0,
                                vector: icons
                            },
                            "{at}: no picture beyond the turning Cube"
                        );
                    }
                }
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
