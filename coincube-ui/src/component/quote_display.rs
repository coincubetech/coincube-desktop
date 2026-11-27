use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;

use iced::{Alignment, Length};
use rand::seq::SliceRandom;
use serde::Deserialize;

use crate::component::text::{CAPTION_SIZE, P2_SIZE};
use crate::widget::{Column, Container};
use crate::{color, font, theme};

// ---------------------------------------------------------------------------
// Quote data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct Quote {
    pub id: String,
    pub text: String,
    pub author: String,
    pub source: String,
    pub category: String,
    pub length: String,
    pub contexts: Vec<String>,
}

#[derive(Deserialize)]
struct QuotesFile {
    quotes: Vec<Quote>,
}

// ---------------------------------------------------------------------------
// Embedded quote data
// ---------------------------------------------------------------------------

const QUOTES_JSON: &str = include_str!("../../static/loading-quotes.json");

static ALL_QUOTES: LazyLock<Vec<Quote>> = LazyLock::new(|| {
    serde_json::from_str::<QuotesFile>(QUOTES_JSON)
        .expect("loading-quotes.json must be valid")
        .quotes
});

static SPANISH_QUOTES: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../static/loading-quotes.es.json"))
        .expect("Spanish quote catalog must be valid")
});

impl Quote {
    /// Resolve text at render time so changing language preserves the selected quote.
    pub fn localized_text(&self) -> &str {
        if crate::locale::language() == crate::locale::Language::Spanish {
            if let Some(text) = SPANISH_QUOTES.get(&self.id) {
                return text;
            }
        }
        &self.text
    }
}

// ---------------------------------------------------------------------------
// QuoteProvider — stateful quote selector with recency tracking
// ---------------------------------------------------------------------------

const RECENCY_WINDOW: usize = 5;

#[derive(Debug, Clone, Default)]
pub struct QuoteProvider {
    recent_ids: VecDeque<String>,
}

/// Pick a single random quote for a context without recency tracking.
/// Use this for one-shot selections (init, celebrations). For repeated
/// selections during a session (e.g. loading screen rotation), use
/// [`QuoteProvider`] to avoid showing the same quote back-to-back.
pub fn random_quote(context: &str) -> Quote {
    QuoteProvider::new().select(context)
}

impl QuoteProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a quote for the given context key.
    ///
    /// Filters by context, excludes recently shown quotes, prefers short
    /// quotes for brief-duration states, then picks uniformly at random.
    pub fn select(&mut self, context: &str) -> Quote {
        let prefer_short = matches!(
            context,
            "transaction-sent"
                | "transaction-received"
                | "balance-update"
                | "error"
                | "lightning-send"
                | "lightning-receive"
                | "note-send"
                | "note-receive"
                | "spark-send"
                | "spark-receive"
                | "bitcoin-send"
                | "bitcoin-receive"
        );

        // For network-specific contexts, fall back to the generic
        // transaction-sent / transaction-received context when no
        // quotes explicitly list the specific context. Exact matches
        // are always preferred — the fallback pool is only consulted
        // when no exact quotes exist, so a small curated set of
        // network-specific quotes isn't diluted by the much larger
        // generic pool.
        let fallback_context = if context.ends_with("-send") {
            Some("transaction-sent")
        } else if context.ends_with("-receive") {
            Some("transaction-received")
        } else {
            None
        };

        let by_context = |ctx: &str| -> Vec<&'static Quote> {
            ALL_QUOTES
                .iter()
                .filter(|q| q.contexts.iter().any(|c| c == ctx))
                .collect()
        };

        let exact_all = by_context(context);
        let exact_fresh: Vec<&Quote> = exact_all
            .iter()
            .copied()
            .filter(|q| !self.recent_ids.contains(&q.id))
            .collect();

        // Preference order:
        //   1. Exact context, excluding recently shown.
        //   2. Exact context, including recently shown (recency
        //      window exhausted).
        //   3. Fallback context, excluding recently shown.
        //   4. Fallback context, including recently shown.
        let pool: Vec<&Quote> = if !exact_fresh.is_empty() {
            exact_fresh
        } else if !exact_all.is_empty() {
            exact_all
        } else if let Some(fb) = fallback_context {
            let fb_all = by_context(fb);
            let fb_fresh: Vec<&Quote> = fb_all
                .iter()
                .copied()
                .filter(|q| !self.recent_ids.contains(&q.id))
                .collect();
            if !fb_fresh.is_empty() {
                fb_fresh
            } else {
                fb_all
            }
        } else {
            Vec::new()
        };

        // Prefer short quotes for brief-duration states
        let final_pool = if prefer_short {
            let short: Vec<_> = pool
                .iter()
                .filter(|q| q.length == "short")
                .copied()
                .collect();
            if short.is_empty() {
                pool
            } else {
                short
            }
        } else {
            pool
        };

        let mut rng = rand::thread_rng();
        let chosen = final_pool
            .choose(&mut rng)
            .copied()
            .unwrap_or_else(|| ALL_QUOTES.first().expect("quotes must not be empty"));

        self.remember(chosen);
        chosen.clone()
    }

    /// Count a quote picked elsewhere (e.g. by [`random_quote`]) as shown,
    /// so the next selection does not repeat it.
    pub fn remember(&mut self, quote: &Quote) {
        self.recent_ids.push_back(quote.id.clone());
        if self.recent_ids.len() > RECENCY_WINDOW {
            self.recent_ids.pop_front();
        }
    }
}

// ---------------------------------------------------------------------------
// QuoteDisplay — stateless display component
// ---------------------------------------------------------------------------

/// Props for the [`display`] component.
pub struct QuoteDisplayProps<'a> {
    pub quote: &'a Quote,
}

impl<'a> QuoteDisplayProps<'a> {
    pub fn new(quote: &'a Quote) -> Self {
        Self { quote }
    }
}

/// Renders a contextual quote and its attribution. Desktop V3 draws no
/// artwork with it: the quote is text (invariant R7 keeps its selection).
///
/// ```text
/// ┌─────────────────────────────┐
/// │  "Quote text here..."       │
/// │  — Author, Source           │
/// └─────────────────────────────┘
/// ```
pub fn display<'a, M: 'a>(props: &QuoteDisplayProps<'a>) -> Container<'a, M> {
    Container::new(quote_block(props.quote))
        .center_x(Length::Fill)
        .max_width(448)
}

/// The quote and its attribution as [`display`] sets them, without the
/// width cap, for a screen that puts the Cube above it.
pub fn quote_block<'a, M: 'a>(quote: &'a Quote) -> Column<'a, M> {
    let quote_text = iced::widget::text(quote.localized_text())
        .size(P2_SIZE)
        .font(font::REGULAR)
        .style(theme::text::secondary)
        .shaping(iced::advanced::text::Shaping::Advanced)
        .align_x(Alignment::Center)
        .width(Length::Fill);

    let mut quote_col = Column::new()
        .align_x(Alignment::Center)
        .spacing(6)
        .padding(24)
        .push(quote_text);

    if let Some(attribution) = format_attribution(quote) {
        let attr_text = iced::widget::text(attribution)
            .size(CAPTION_SIZE)
            .font(font::ITALIC)
            .color(color::GREY_3)
            .shaping(iced::advanced::text::Shaping::Advanced)
            .align_x(Alignment::Center)
            .width(Length::Fill);

        quote_col = quote_col.push(attr_text);
    }

    quote_col
}

/// Format the attribution line for a quote.
///
/// - `scripture`: em-dash + author (which contains the verse reference)
/// - All others: em-dash + author + source
pub(crate) fn format_attribution(quote: &Quote) -> Option<String> {
    match quote.category.as_str() {
        "scripture" => Some(format!("\u{2014} {}", quote.author)),
        _ => Some(format!("\u{2014} {}, {}", quote.author, quote.source)),
    }
}

#[cfg(test)]
mod localization_tests {
    use super::*;

    #[test]
    fn spanish_quotes_cover_every_bundled_id_and_keep_selection() {
        assert_eq!(SPANISH_QUOTES.len(), ALL_QUOTES.len());
        for quote in ALL_QUOTES.iter() {
            let spanish = SPANISH_QUOTES.get(&quote.id).expect("Spanish quote by ID");
            assert!(!spanish.trim().is_empty());
            assert_eq!(quote.localized_text(), quote.text);
            crate::locale::render_in(crate::locale::Language::Spanish, || {
                assert_eq!(quote.localized_text(), spanish);
            });
            assert_eq!(quote.localized_text(), quote.text);
        }
    }
}
