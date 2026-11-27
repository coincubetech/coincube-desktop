use coincube_ui::component::{
    amount::BitcoinDisplayUnit,
    payment_result::{payment_result, Outcome},
    quote_display,
};
use coincube_ui::illustration::Slot;
use coincube_ui::image;
use coincube_ui::theme::{ArtworkStyle, Theme, WorldviewMode};

use crate::app::view::settings::page::illustrated_flow;
use crate::app::view::shared::landing::capture;
use crate::app::view::spark::{SparkTransactionsStatus, SparkTransactionsView};
use crate::chain::ChainId;
use crate::home::launcher_presenter::{opening_body, project_opening};
use crate::home::restore_frame::done_art;

#[test]
fn worldview_art_fits_existing_views() {
    check_worldview_art(ArtworkStyle::Existing, "existing");
}

#[test]
fn worldview_art_fits_full_scene_views() {
    check_worldview_art(ArtworkStyle::Scene, "scene");
}

fn check_worldview_art(artwork: ArtworkStyle, art_label: &str) {
    let empty = quote_display::image_handle_for_context("empty-wallet");
    let loading_quote = quote_display::random_quote("loading");
    let empty_quote = quote_display::random_quote("empty-wallet");
    let startup_error = crate::loader::Step::Error(Box::new(crate::loader::Error::Unexpected(
        "Fixture startup error".into(),
    )));
    let error_quote = quote_display::random_quote("error");
    let sent_quote = quote_display::random_quote("bitcoin-send");
    let sent_image = quote_display::image_handle_for_context("bitcoin-send");
    let received_quote = quote_display::random_quote("spark-receive");
    let received_image = quote_display::image_handle_for_context("spark-receive");
    for (mode, name) in [
        (WorldviewMode::Samurai, "samurai"),
        (WorldviewMode::Suitcoiner, "suitcoiner"),
        (WorldviewMode::Luke, "luke"),
    ] {
        for (appearance, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
            let theme = theme.with_worldview(mode).with_artwork(artwork);
            let opened = capture::inspect_filled(
                &format!("worldview-{name}-{art_label}-opening"),
                opening_body(
                    project_opening("Everyday", ChainId::Bitcoin),
                    &crate::loader::Step::Connecting,
                    &loading_quote,
                    false,
                ),
                (1024, 768),
                appearance,
                theme,
                (1024, 768),
            );
            assert!(opened.contains("Connecting to daemon"));
            let failed = capture::inspect_filled(
                &format!("worldview-{name}-{art_label}-startup-error"),
                crate::loader::view(&startup_error, &error_quote),
                (1024, 768),
                appearance,
                theme,
                (1024, 768),
            );
            assert!(failed.contains("COINCUBE Desktop failed to start"));
            let history = capture::inspect(
                &format!("worldview-{name}-{art_label}-empty-history"),
                SparkTransactionsView {
                    status: SparkTransactionsStatus::Loaded(vec![]),
                    recent_transactions: &[],
                    fiat_converter: None,
                    bitcoin_unit: BitcoinDisplayUnit::Sats,
                    show_direction_badges: false,
                    empty_state_quote: &empty_quote,
                    empty_state_image_handle: &empty,
                    current_page: 0,
                    is_last_page: true,
                    processing: false,
                    mask_amounts: true,
                }
                .render(),
                (1024, 768),
                appearance,
                theme,
            );
            assert!(history.contains("Your Spark wallet is ready"));

            use coincube_ui::worldview_copy::{self, VoiceLine};
            let sent_line = VoiceLine::PaymentSent.for_mode(mode);
            let received_line = VoiceLine::PaymentReceived.for_mode(mode);
            for (screen, view, heading) in [
                (
                    "sent",
                    payment_result(
                        Outcome::Sent,
                        "50,000 sats",
                        "bitcoin-send",
                        &sent_quote,
                        &sent_image,
                        (),
                    )
                    .voice(sent_line)
                    .into(),
                    "Payment sent",
                ),
                (
                    "received",
                    worldview_copy::render_in(mode, || {
                        payment_result(
                            Outcome::Received,
                            "50,000 sats",
                            "spark-receive",
                            &received_quote,
                            &received_image,
                            (),
                        )
                        .voice(received_line)
                        .into()
                    }),
                    "Payment received",
                ),
                ("restored", done_art(appearance == "light"), ""),
                (
                    "protected",
                    illustrated_flow(
                        "Protect your Cube",
                        "Set up a Recovery Kit.",
                        image::recovery_kit_protected(),
                        Slot::KitProtection,
                    )
                    .into(),
                    "Protect your Cube",
                ),
                (
                    "withdrawn",
                    illustrated_flow(
                        "Remove Recovery Kit",
                        "Confirm removal.",
                        image::recovery_kit_withdrawn(),
                        Slot::KitRemoval,
                    )
                    .into(),
                    "Remove Recovery Kit",
                ),
                (
                    "pin",
                    illustrated_flow(
                        "Unlock your seed phrase",
                        "Enter your Cube PIN.",
                        image::pin_unlock_lock(),
                        Slot::Pin,
                    )
                    .into(),
                    "Unlock your seed phrase",
                ),
            ] {
                let rendered = capture::inspect(
                    &format!("worldview-{name}-{art_label}-{screen}"),
                    view,
                    (1024, 768),
                    appearance,
                    theme,
                );
                check_art_labels(&rendered, screen, heading, sent_line, received_line);
            }
        }
    }
}

fn check_art_labels(
    rendered: &capture::Inspection,
    screen: &str,
    heading: &str,
    sent_line: Option<&str>,
    received_line: Option<&str>,
) {
    if !heading.is_empty() {
        assert!(rendered.contains(heading));
    }
    if let Some(line) = match screen {
        "sent" => sent_line,
        "received" => received_line,
        _ => None,
    } {
        assert!(rendered.contains(line));
    }
}

#[test]
fn spanish_samurai_wallet_actions_and_errors_render_together() {
    use coincube_ui::component::notification;
    use coincube_ui::locale::{render_in, Language};

    let quote = quote_display::random_quote("empty-wallet");
    let image = quote_display::image_handle_for_context("empty-wallet");
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
            empty_state_image_handle: &image,
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

/// The Opening paths draw the line Cube and no artwork: the Launcher's
/// Opening face, the loader (connecting, starting, scanning and its error),
/// and `loader::cube_and_quote`, which the passkey Opening shows. Counted
/// from what the software renderer actually drew, so a picture cannot hide
/// in a view the text probe passes over: no raster image (the ink art was
/// raster), and at least one SVG (the Cube) on every Opening view.
#[test]
fn every_opening_draws_the_cube_and_no_raster_art() {
    let quote = quote_display::random_quote("loading");
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
            for artwork in [ArtworkStyle::Existing, ArtworkStyle::Scene] {
                let theme = theme.with_worldview(mode).with_artwork(artwork);
                let mut views: Vec<(String, capture::Inspection)> = vec![
                    (
                        "launcher-opening".into(),
                        capture::inspect_rendered_filled(
                            &format!("opening-cube-{name}-launcher"),
                            opening_body(
                                project_opening("Everyday", ChainId::Bitcoin),
                                &crate::loader::Step::Connecting,
                                &quote,
                                false,
                            ),
                            (1024, 768),
                            appearance,
                            theme,
                        ),
                    ),
                    (
                        "passkey-opening".into(),
                        capture::inspect_rendered_filled(
                            &format!("opening-cube-{name}-passkey"),
                            crate::loader::cube_and_quote::<()>(&quote),
                            (1024, 768),
                            appearance,
                            theme,
                        ),
                    ),
                ];
                for (step_name, step) in &steps {
                    views.push((
                        format!("loader-{step_name}"),
                        capture::inspect_rendered_filled(
                            &format!("opening-cube-{name}-loader-{step_name}"),
                            crate::loader::view(step, &quote),
                            (1024, 768),
                            appearance,
                            theme,
                        ),
                    ));
                }
                let failed = capture::inspect_rendered_filled(
                    &format!("opening-cube-{name}-loader-error"),
                    crate::loader::view(&error, &quote),
                    (1024, 768),
                    appearance,
                    theme,
                );
                assert!(failed.contains("COINCUBE Desktop failed to start"));
                assert_eq!(
                    failed.drawn_images().raster,
                    0,
                    "{} {}: the startup error draws raster art",
                    name,
                    appearance
                );
                for (view, inspection) in &views {
                    let drawn = inspection.drawn_images();
                    assert_eq!(
                        drawn.raster, 0,
                        "{} {} {}: draws raster art",
                        name, appearance, view
                    );
                    assert!(
                        drawn.vector >= 1,
                        "{} {} {}: the line Cube is missing",
                        name,
                        appearance,
                        view
                    );
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
