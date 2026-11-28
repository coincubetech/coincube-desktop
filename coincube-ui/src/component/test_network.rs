//! The test-network tag (`.tnet`, canvas `D4-Notices`, `MV-Notices-Proposed`):
//! a quiet chip beside the wordmark that names a test network in words, in
//! place of the full-width blue bar. It is how a user always knows the coins
//! in front of them have no value (Phase 2 invariant S1), so every network
//! that is not mainnet gets one (S2): an unknown network fails safe to a
//! generic name rather than to no tag.

use bitcoin::Network;
use iced::advanced::text::Shaping;
use iced::widget::{container, row, text};
use iced::{Alignment, Background, Border, Length};

use crate::font;
use crate::locale::copy as localized_copy;
use crate::theme::{tenshu, Theme};
use crate::widget::Element;

/// `.tnet { height: 24px }`.
pub const HEIGHT: f32 = 24.0;
/// `.tnet { border-radius: 6px }`.
const RADIUS: f32 = 6.0;
/// `.tnet { background: color-mix(in srgb, var(--ink) 8%, transparent) }`.
pub const FILL_ALPHA: f32 = 0.08;
/// `.tnet b { font-size: 11px }`, the network's name.
const NAME_SIZE: f32 = 11.0;
/// `.tnet { font-size: 12.5px }`, the line after the name.
const LINE_SIZE: f32 = 12.5;

/// What the tag says after the name.
pub const NO_VALUE: &str = "Test coins, no value";

/// The upper-case name a test network's tag shows, or `None` on mainnet.
///
/// Every `bitcoin::Network` is handled: a network added to the enum later
/// still gets a tag, with a generic name, because only mainnet goes without.
pub fn test_network_name(network: Network) -> Option<&'static str> {
    match network {
        Network::Bitcoin => None,
        Network::Testnet => Some("TESTNET"),
        Network::Testnet4 => Some("TESTNET4"),
        Network::Signet => Some("SIGNET"),
        Network::Regtest => Some("REGTEST"),
        #[allow(unreachable_patterns)]
        _ => Some(localized_copy("TEST NETWORK")),
    }
}

/// `.tnet`: the network's name in mono capitals, then "Test coins, no value"
/// in the muted ink, on an 8% ink fill. It does not wrap or shrink, so it
/// always reads whole.
pub fn test_network_tag<'a, M: 'a>(name: &'a str) -> Element<'a, M> {
    let name = text(name)
        .size(NAME_SIZE)
        .font(font::MONO_MEDIUM)
        .shaping(Shaping::Advanced)
        .wrapping(text::Wrapping::None)
        .style(tenshu::text::ink);
    let line = text(localized_copy(NO_VALUE))
        .size(LINE_SIZE)
        .font(font::REGULAR)
        .shaping(Shaping::Advanced)
        .wrapping(text::Wrapping::None)
        .style(tenshu::text::muted);
    container(row![name, line].spacing(8).align_y(Alignment::Center))
        .height(Length::Fixed(HEIGHT))
        .center_y(Length::Fixed(HEIGHT))
        .padding(iced::Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 8.0,
        })
        .style(style)
        .into()
}

/// The tag's fill: the ink at 8%, with no outline.
pub fn style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(tenshu::faded(
            theme.colors.shell.ink,
            FILL_ALPHA,
        ))),
        border: Border {
            radius: RADIUS.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mainnet alone has no tag; every test network is named in words.
    #[test]
    fn every_network_but_mainnet_is_named() {
        assert_eq!(test_network_name(Network::Bitcoin), None);
        for (network, name) in [
            (Network::Testnet, "TESTNET"),
            (Network::Testnet4, "TESTNET4"),
            (Network::Signet, "SIGNET"),
            (Network::Regtest, "REGTEST"),
        ] {
            assert_eq!(test_network_name(network), Some(name));
        }
    }

    /// The tag sits on the bar's rail: its fill must read there.
    #[test]
    fn the_fill_is_ink_at_eight_percent_with_no_outline() {
        for theme in [Theme::dark(), Theme::light()] {
            let s = style(&theme);
            assert_eq!(s.border.width, 0.0);
            let Some(Background::Color(fill)) = s.background else {
                panic!("no flat fill");
            };
            assert_eq!(fill, tenshu::faded(theme.colors.shell.ink, FILL_ALPHA));
        }
    }
}
