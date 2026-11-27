//! Desktop V3 Phase 1 (V6): the screens that used to draw Worldview artwork
//! now show text only. Each is mounted in every worldview, both themes and
//! both languages (invariant R8): the quote stays (R7), the worldview's
//! wording stays (R2), and nothing else replaces the picture.

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

                let opened = capture::inspect_filled(
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
                    (1024, 768),
                );
                assert!(opened.contains(&loading.text), "{}", scenario("opening"));
                assert!(opened.contains("— Author, Source"));

                let failed = capture::inspect_filled(
                    &scenario("startup-error"),
                    render_in(language, || crate::loader::view(&startup_error, &error)),
                    (1024, 768),
                    appearance,
                    theme,
                    (1024, 768),
                );
                assert!(failed.contains(&error.text));
                assert!(failed.contains("Fixture startup error"));

                let history = capture::inspect(
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
                    (1024, 768),
                    appearance,
                    theme,
                );
                assert!(history.contains(&empty.text));
                let sentence = render_in(language, || {
                    coincube_ui::locale::Text::SparkHistoryEmpty.get()
                });
                assert!(history.contains(sentence), "{}", scenario("empty-history"));

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
                    let page = capture::inspect(
                        &scenario(screen),
                        render_in(language, || {
                            coincube_ui::worldview_copy::render_in(mode, || {
                                payment_result(outcome, "50,000 sats", quote, ())
                                    .voice(line)
                                    .into()
                            })
                        }),
                        (1024, 768),
                        appearance,
                        theme,
                    );
                    assert!(page.contains("50,000"), "{}", scenario(screen));
                    assert!(page.contains(&quote.text), "{}", scenario(screen));
                    if let Some(line) = line {
                        assert!(page.contains(line), "{}: {}", scenario(screen), line);
                    }
                }

                let restored = capture::inspect(
                    &scenario("restored"),
                    done_mark::<()>(),
                    (1024, 768),
                    appearance,
                    theme,
                );
                // The plain check is one glyph, and nothing else is drawn.
                assert_eq!(restored.text.len(), 1, "{}", scenario("restored"));

                let flow = capture::inspect(
                    &scenario("recovery-kit"),
                    flow_intro::<()>("Protect your Cube", "Set up a Recovery Kit.").into(),
                    (1024, 768),
                    appearance,
                    theme,
                );
                assert!(flow.contains("Protect your Cube"));
                assert!(flow.contains("Set up a Recovery Kit."));
                assert_eq!(flow.text.len(), 2, "{}", scenario("recovery-kit"));
            }
        }
    }
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
