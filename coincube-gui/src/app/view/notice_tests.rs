//! Drawn checks for the one notice anatomy (Phase 2 W3, option D): each tone
//! in both themes, through the pixels the software renderer draws.

use coincube_ui::component::notice::{notice, NoticeTone};
use coincube_ui::component::{card, notification};
use coincube_ui::theme::notification::toast_fill;
use coincube_ui::theme::tenshu::{mix, separation::delta_e};
use coincube_ui::theme::Theme;
use coincube_ui::widget::{Container, Element};
use iced::Length;

use super::shared::landing::capture::{render_pixels, Pixels};
use super::Message;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 120;

/// The notices the app draws, one per tone, through the call sites' own
/// functions where a legacy one exists.
fn notices() -> Vec<(NoticeTone, Container<'static, Message>)> {
    vec![
        (
            NoticeTone::Info,
            notice(NoticeTone::Info, "Vault is syncing").into_container(),
        ),
        (
            NoticeTone::Success,
            notice(NoticeTone::Success, "Recovery Kit saved").into_container(),
        ),
        (
            NoticeTone::Warning,
            notification::warning(
                "Spark is reconnecting".into(),
                "Spark is reconnecting".into(),
            ),
        ),
        (
            NoticeTone::Warning,
            card::warning("Couldn't read the install stats".into()),
        ),
        (
            NoticeTone::Error,
            notification::error_card::<Message>(
                "Couldn't load Vault activity",
                "",
                "CC-NET-TIMEOUT",
                None,
            ),
        ),
    ]
}

/// The notice at the top of a page-coloured window.
fn draw(notice: Container<'static, Message>, theme: &Theme) -> Pixels {
    let page: Element<'static, Message> = iced::widget::container(notice)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|t: &Theme| iced::widget::container::Style {
            background: Some(t.colors.shell.background.into()),
            ..Default::default()
        })
        .into();
    render_pixels(page, (WIDTH, HEIGHT), theme, None)
}

/// Warnings and errors wash their panel 8% in the tone; info and success
/// stay on the plain panel. The drawn fill, right of the copy, says which.
#[test]
fn warnings_and_errors_wash_the_panel_and_the_rest_stay_plain() {
    for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
        let panel = theme.colors.shell.panel;
        for (tone, view) in notices() {
            let pixels = draw(view, &theme);
            let drawn = pixels.at(WIDTH - 40, 6);
            let expected = toast_fill(&theme, tone);
            assert!(
                delta_e(drawn, expected) < 1.0,
                "{} {:?}: drawn {:?}, expected {:?}",
                name,
                tone,
                drawn,
                expected
            );
            if tone.needs_attention() {
                assert!(
                    delta_e(drawn, panel) >= 1.0,
                    "{} {:?}: the wash does not show ({:?} on {:?})",
                    name,
                    tone,
                    drawn,
                    panel
                );
            } else {
                assert!(delta_e(drawn, panel) < 1.0, "{} {:?}", name, tone);
            }
        }
    }
}

/// Every notice keeps the 1px line border, never an edge in its tone, and
/// no tone block: the old orange fill and red outline are gone.
#[test]
fn every_notice_has_the_line_border_not_the_tone() {
    for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
        let line = theme.colors.shell.line;
        for (tone, view) in notices() {
            let pixels = draw(view, &theme);
            let edge = pixels.at(WIDTH / 2, 0);
            assert!(
                delta_e(edge, line) < 2.0,
                "{} {:?}: the top edge is {:?}, not the line {:?}",
                name,
                tone,
                edge,
                line
            );
            assert!(
                delta_e(edge, tone.color(&theme)) > 10.0,
                "{} {:?}: the edge reads as the tone",
                name,
                tone
            );
        }
    }
}

/// The tone lives in the 28px icon tile: the tone at 16% over the fill,
/// just above the glyph.
#[test]
fn the_icon_tile_carries_the_tone() {
    for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
        for (tone, view) in notices() {
            let pixels = draw(view, &theme);
            // The 28px tile spans x 14..42 (padding 14) and is centred on
            // the row; walking down its middle column from the top padding,
            // the first pixel off the fill is the tile, above the glyph.
            let fill = toast_fill(&theme, tone);
            let top = (12..HEIGHT - 2)
                .find(|y| delta_e(pixels.at(28, *y), fill) >= 1.0)
                .expect("a tile is drawn");
            // Two pixels in, clear of the edge's anti-aliasing.
            let drawn = pixels.at(28, top + 2);
            let expected = mix(fill, tone.color(&theme), 0.16);
            assert!(
                delta_e(drawn, expected) < 2.0,
                "{} {:?}: the tile is {:?}, expected {:?}",
                name,
                tone,
                drawn,
                expected
            );
        }
    }
}

/// S7: an error notice keeps its support reference, in both languages.
#[tokio::test]
async fn an_error_notice_keeps_its_reference() {
    use coincube_ui::locale::{render_in, Language};
    for (language, expected) in [
        (Language::English, "Ref: CC-NET-TIMEOUT"),
        (Language::Spanish, "Ref.: CC-NET-TIMEOUT"),
    ] {
        let view: Element<'static, Message> = render_in(language, || {
            notification::error_card::<Message>(
                "Couldn't load Vault activity",
                "Check your connection, then try again.",
                "CC-NET-TIMEOUT",
                Some(("Retry", Message::Reload)),
            )
            .into()
        });
        let shown = crate::home::restore_frame::rendered_labels(view).await;
        assert!(
            shown.iter().any(|t| t == expected),
            "{:?}: {:?}",
            language,
            shown
        );
    }
}

/// Opt-in look check: every tone, with and without an action, in both
/// themes. `NOTICE_CAPTURE_DIR=<dir> cargo test -p coincube-gui --lib
/// notice_capture -- --ignored`.
#[test]
#[ignore = "writes PNG frames"]
fn notice_capture() {
    let Some(directory) = std::env::var_os("NOTICE_CAPTURE_DIR") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&directory).expect("capture directory");
    for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
        let column = iced::widget::Column::new()
            .spacing(10)
            .padding(24)
            .push(notification::warning(
                "Spark is reconnecting".into(),
                "Balances may be out of date. Payments resume when the connection returns.".into(),
            ))
            .push(notification::error_card::<Message>(
                "Couldn't load Vault activity",
                "Check your connection, then try again.",
                "CC-NET-TIMEOUT",
                Some(("Retry", Message::Reload)),
            ))
            .push(
                notice(NoticeTone::Info, "Vault is syncing")
                    .detail("History appears as blocks are checked. You can still receive.")
                    .into_container(),
            )
            .push(
                notice(NoticeTone::Success, "Recovery Kit saved")
                    .detail("Your encrypted backup is stored on your Connect account.")
                    .action("View", Message::Reload)
                    .into_container(),
            )
            .push(super::spark_notice_banner(
                &crate::app::cache::SparkNotice::StableBalanceOffWithHolding,
            ));
        let page: Element<'_, Message> = iced::widget::container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|t: &Theme| iced::widget::container::Style {
                background: Some(t.colors.shell.background.into()),
                ..Default::default()
            })
            .into();
        let (w, h) = (900u32, 560u32);
        let pixels = render_pixels(page, (w, h), &theme, None);
        image::save_buffer(
            directory.join(format!("notices-{}.png", name)),
            &pixels.rgba,
            w,
            h,
            image::ColorType::Rgba8,
        )
        .expect("save notice capture");
    }
}
