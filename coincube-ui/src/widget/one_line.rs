//! One line of text that ends in "…" where it would run past its box
//! (`white-space: nowrap; overflow: hidden; text-overflow: ellipsis`).
//!
//! Iced's text either wraps or draws past its bounds. A name in a fixed
//! region, such as the Cube switcher in a sidebar that is 248 or 208 px
//! wide, needs the browser's cut instead: as much of the name as fits, then
//! an ellipsis. The cut is measured with the renderer's own shaping at the
//! width the layout gives, and never splits a grapheme.

use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::text::{self, paragraph, Paragraph as _};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::widget::{Operation, Widget};
use iced::advanced::Layout;
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{alignment, mouse, Color, Font, Length, Pixels, Rectangle, Size};
use unicode_segmentation::UnicodeSegmentation;

use crate::theme::Theme;
use crate::widget::{Element, Renderer};

type Paragraph = <Renderer as text::Renderer>::Paragraph;

const ELLIPSIS: &str = "…";

/// A text colour drawn from the theme.
type ColorFn<'a> = Box<dyn Fn(&Theme) -> Color + 'a>;

/// `content` on one line, cut with an ellipsis to the width it is given.
pub fn one_line<'a>(content: impl Into<String>) -> OneLine<'a> {
    OneLine {
        content: content.into(),
        size: None,
        font: None,
        line_height: LineHeight::default(),
        color: None,
    }
}

pub struct OneLine<'a> {
    content: String,
    size: Option<Pixels>,
    font: Option<Font>,
    line_height: LineHeight,
    color: Option<ColorFn<'a>>,
}

impl<'a> OneLine<'a> {
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// The text colour; without one it takes the surrounding text colour.
    pub fn color(mut self, color: impl Fn(&Theme) -> Color + 'a) -> Self {
        self.color = Some(Box::new(color));
        self
    }
}

#[derive(Default)]
struct State {
    paragraph: paragraph::Plain<Paragraph>,
    /// The text drawn: `content`, or a prefix of it and the ellipsis.
    shown: String,
    /// What `shown` was cut for: the content, the width, the size and font.
    cut_for: Option<(String, u32, u32, Font)>,
}

impl OneLine<'_> {
    fn text<'b>(&self, content: &'b str, size: Pixels, font: Font) -> text::Text<&'b str, Font> {
        text::Text {
            content,
            bounds: Size::INFINITE,
            size,
            line_height: self.line_height,
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::None,
        }
    }

    fn width_of(&self, content: &str, size: Pixels, font: Font) -> f32 {
        Paragraph::with_text(self.text(content, size, font))
            .min_bounds()
            .width
    }

    /// The longest grapheme prefix that fits in `width` with the ellipsis
    /// after it, or the whole content when it fits as it is.
    fn cut(&self, width: f32, size: Pixels, font: Font) -> String {
        if self.width_of(&self.content, size, font) <= width {
            return self.content.clone();
        }
        let ends: Vec<usize> = self
            .content
            .grapheme_indices(true)
            .map(|(start, grapheme)| start + grapheme.len())
            .collect();
        let with_ellipsis = |count: usize| {
            let prefix = if count == 0 {
                ""
            } else {
                &self.content[..ends[count - 1]]
            };
            format!("{}{ELLIPSIS}", prefix.trim_end())
        };
        // Binary search the number of graphemes kept; widths only grow.
        let (mut fits, mut too_long) = (0usize, ends.len());
        while too_long - fits > 1 {
            let middle = (fits + too_long) / 2;
            if self.width_of(&with_ellipsis(middle), size, font) <= width {
                fits = middle;
            } else {
                too_long = middle;
            }
        }
        with_ellipsis(fits)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for OneLine<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self
            .size
            .unwrap_or_else(|| text::Renderer::default_size(renderer));
        let font = self
            .font
            .unwrap_or_else(|| text::Renderer::default_font(renderer));
        let width = limits.max().width;
        let key = (
            self.content.clone(),
            width.to_bits(),
            size.0.to_bits(),
            font,
        );
        let state = tree.state.downcast_mut::<State>();
        if state.cut_for.as_ref() != Some(&key) {
            state.shown = self.cut(width, size, font);
            state.cut_for = Some(key);
        }
        let shown = std::mem::take(&mut state.shown);
        let _ = state.paragraph.update(self.text(&shown, size, font));
        state.shown = shown;
        let natural = state.paragraph.min_bounds();
        layout::Node::new(limits.resolve(Length::Fill, Length::Shrink, natural))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let color = self
            .color
            .as_ref()
            .map_or(style.text_color, |color| color(theme));
        text::Renderer::fill_paragraph(
            renderer,
            state.paragraph.raw(),
            layout.bounds().position(),
            color,
            *viewport,
        );
    }

    /// Reports the text as drawn, so a probe reads what the user sees.
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_ref::<State>();
        operation.text(None, layout.bounds(), &state.shown);
    }
}

impl<'a, Message: 'a> From<OneLine<'a>> for Element<'a, Message> {
    fn from(one_line: OneLine<'a>) -> Self {
        Element::new(one_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::renderer::Headless;

    fn renderer() -> Renderer {
        for bytes in crate::font::load() {
            iced::advanced::graphics::text::font_system()
                .write()
                .unwrap()
                .load_font(bytes);
        }
        iced::futures::executor::block_on(<Renderer as Headless>::new(
            crate::font::REGULAR,
            Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    /// Lays `line` out `width` wide; returns what it draws and its size.
    fn shown(renderer: &Renderer, line: OneLine<'_>, width: f32) -> (String, Size) {
        let mut element: Element<'_, ()> = line.into();
        let mut tree = Tree::new(element.as_widget());
        let node = element.as_widget_mut().layout(
            &mut tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, 100.0)),
        );
        let state = tree.state.downcast_ref::<State>();
        let drawn = state.paragraph.min_bounds();
        assert!(
            drawn.width <= width,
            "{:?} draws {} wide in {}",
            state.shown,
            drawn.width,
            width
        );
        (state.shown.clone(), node.size())
    }

    #[test]
    fn a_name_that_fits_is_shown_whole() {
        let renderer = renderer();
        let (text, size) = shown(&renderer, one_line("Everyday").size(15), 200.0);
        assert_eq!(text, "Everyday");
        assert_eq!(size.width, 200.0, "it fills its slot");
    }

    #[test]
    fn a_long_name_is_cut_with_an_ellipsis_on_one_line() {
        let renderer = renderer();
        let name = "A very long Cube name that keeps going to test wrapping behaviour";
        let (one, one_size) = shown(&renderer, one_line("Everyday").size(15), 100.0);
        assert_eq!(one, "Everyday");
        for width in [60.0, 100.0, 140.0] {
            let (text, size) = shown(&renderer, one_line(name).size(15), width);
            assert!(text.ends_with('…'), "{}: {:?}", width, text);
            let kept = text.trim_end_matches('…');
            assert!(!kept.is_empty() && name.starts_with(kept), "{:?}", text);
            assert!(!kept.ends_with(' '), "no space before the ellipsis");
            assert_eq!(
                size.height, one_size.height,
                "one line, as tall as a short name"
            );
        }
        // A wider slot keeps more of the name.
        let narrow = shown(&renderer, one_line(name).size(15), 100.0).0;
        let wide = shown(&renderer, one_line(name).size(15), 140.0).0;
        assert!(wide.len() > narrow.len(), "{:?} vs {:?}", narrow, wide);
    }

    #[test]
    fn the_cut_never_splits_a_grapheme() {
        let renderer = renderer();
        // "e" + combining acute, repeated: a cut between them would leave
        // a bare accent or a bare "e".
        let name = "e\u{301}".repeat(40);
        let (text, _) = shown(&renderer, one_line(name.clone()).size(15), 80.0);
        let kept = text.trim_end_matches('…');
        assert_eq!(kept.chars().count() % 2, 0, "{:?}", text);
        assert!(name.starts_with(kept));
    }
}
