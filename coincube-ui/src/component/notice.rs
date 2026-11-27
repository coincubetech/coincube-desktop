//! One standing notice in the toast's anatomy (`.nt`, canvas `D4-Notices`,
//! option D, Robert 2026-11-27): a filled row, the tone in a 28px icon tile,
//! a title, one line, and at most one action, an outlined content button.
//!
//! A warning or error washes its panel 8% in its tone and keeps the plain
//! line border; info and success stay on the plain panel. The tone shows in
//! the icon tile and the wash, never as a coloured edge or a solid block
//! (Phase 2 S3). An error's support reference stays on it (S7).

use iced::advanced::text::Shaping;
use iced::widget::{button, container, text, Column, Row};
use iced::{Alignment, Length};

use crate::component::tenshu::text as tenshu_text;
use crate::font;
use crate::icon;
use crate::theme::notification::{self, ToastTone};
use crate::theme::tenshu::button::{action, ActionKind, ActionShape};
use crate::theme::{self, Theme};
use crate::widget::{Container, Element};

/// `.nt .ti { width: 28px }`, and the dismiss control's square.
pub const TILE: f32 = 28.0;
/// `.ic { width: 18px }`.
const GLYPH: f32 = 18.0;
/// `.nt .ds { font-size: 13px }`.
const DETAIL_SIZE: f32 = 13.0;
/// `.nt .ref { font-size: 11.5px }`.
const REFERENCE_SIZE: f32 = 11.5;
/// `.nt .na { height: 32px }`.
const ACTION_HEIGHT: f32 = 32.0;

/// The tone of a notice. Notices share the toasts' tones.
pub type NoticeTone = ToastTone;

/// A notice under construction. Convert it into a [`Container`] or an
/// [`Element`].
pub struct Notice<'a, M> {
    tone: NoticeTone,
    title: String,
    detail: Option<String>,
    reference: Option<String>,
    action: Option<(String, M)>,
    dismiss: Option<M>,
    _lifetime: std::marker::PhantomData<&'a ()>,
}

/// A notice with `title` in `tone`. The title is shown as given: callers
/// localize it.
pub fn notice<'a, M>(tone: NoticeTone, title: impl Into<String>) -> Notice<'a, M> {
    Notice {
        tone,
        title: title.into(),
        detail: None,
        reference: None,
        action: None,
        dismiss: None,
        _lifetime: std::marker::PhantomData,
    }
}

impl<'a, M: Clone + 'a> Notice<'a, M> {
    /// The one line under the title.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        if !detail.is_empty() {
            self.detail = Some(detail);
        }
        self
    }

    /// The support reference, shown as "Ref: …" under the line.
    pub fn reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    /// The notice's one action: an outlined content button.
    pub fn action(mut self, label: impl Into<String>, message: M) -> Self {
        self.action = Some((label.into(), message));
        self
    }

    /// A quiet close × with no box.
    pub fn dismiss(mut self, message: M) -> Self {
        self.dismiss = Some(message);
        self
    }

    pub fn into_container(self) -> Container<'a, M> {
        let tone = self.tone;
        let glyph = glyph(tone).size(GLYPH).style(move |t: &Theme| text::Style {
            color: Some(tone.color(t)),
        });
        let tile = container(glyph)
            .width(TILE)
            .height(TILE)
            .center_x(TILE)
            .center_y(TILE)
            .style(move |t| notification::notice_icon_tile(t, tone));

        // `WordOrGlyph`: an address or an error string with no spaces must
        // still wrap inside the row.
        let title = tenshu_text::label(self.title).wrapping(text::Wrapping::WordOrGlyph);
        let detail = self.detail.map(|detail| {
            tenshu_text::body(detail)
                .size(DETAIL_SIZE)
                .style(theme::tenshu::text::muted)
                .wrapping(text::Wrapping::WordOrGlyph)
        });
        let reference = self.reference.map(|reference| {
            text(crate::localized_format!("Ref: {}", "Ref.: {}", reference))
                .size(REFERENCE_SIZE)
                .font(font::MONO_REGULAR)
                .shaping(Shaping::Advanced)
                .style(theme::tenshu::text::muted)
        });
        let copy = Column::new()
            .spacing(1)
            .width(Length::Fill)
            .push(title)
            .push(detail)
            .push(reference);

        let outlined = self.action.map(|(label, message)| {
            button(
                text(label)
                    .size(DETAIL_SIZE)
                    .font(font::MEDIUM)
                    .shaping(Shaping::Advanced)
                    .wrapping(text::Wrapping::None)
                    .height(Length::Fill)
                    .align_y(Alignment::Center),
            )
            .height(ACTION_HEIGHT)
            .padding([0.0, 14.0])
            .style(theme::button::secondary)
            .on_press(message)
        });
        let dismiss = self.dismiss.map(|message| {
            button(
                icon::cross_icon()
                    .size(16.0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .padding(0)
            .width(TILE)
            .height(TILE)
            .style(|t, status| action(t, status, ActionKind::Quiet, ActionShape::Icon))
            .on_press(message)
        });

        let row = Row::new()
            .spacing(12)
            .align_y(Alignment::Center)
            .push(tile)
            .push(copy)
            .push(outlined)
            .push(dismiss);

        Container::new(row)
            .width(Length::Fill)
            .padding([12.0, 14.0])
            .style(move |t| notification::notice(t, tone))
    }
}

impl<'a, M: Clone + 'a> From<Notice<'a, M>> for Container<'a, M> {
    fn from(notice: Notice<'a, M>) -> Self {
        notice.into_container()
    }
}

impl<'a, M: Clone + 'a> From<Notice<'a, M>> for Element<'a, M> {
    fn from(notice: Notice<'a, M>) -> Self {
        notice.into_container().into()
    }
}

/// The tone's glyph, as on the toasts.
fn glyph<'a>(tone: NoticeTone) -> crate::widget::Text<'a> {
    match tone {
        ToastTone::Info => icon::tooltip_icon(),
        ToastTone::Success => icon::check_circle_icon(),
        ToastTone::Warning => icon::warning_icon(),
        ToastTone::Error => icon::x_circle_icon(),
        ToastTone::Debug => icon::code_slash_icon(),
    }
}
