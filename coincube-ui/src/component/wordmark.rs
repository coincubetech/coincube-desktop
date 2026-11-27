//! The COINCUBE wordmark: Space Grotesk Bold in one ink, tracked 0.08em
//! (`letter-spacing: .08em`), as Mobile and the canvas draw it.
//!
//! Iced 0.14 has no letter spacing: `text::Span` and the text widget carry no
//! tracking, and the renderer builds its cosmic-text attributes without it.
//! So the word is laid out as one glyph per text widget, spaced by the
//! tracking. Pair kerning between the capitals is lost, which tracked capitals
//! hardly show. To operations (tests, focus and any future accessibility
//! tree) the mark is still one text, "COINCUBE", over its whole bounds.

use std::rc::Rc;

use iced::advanced::{
    layout, mouse, renderer,
    widget::{Operation, Tree},
    Layout, Widget,
};
use iced::{Color, Length, Rectangle, Size};

use crate::font;
use crate::theme::Theme;
use crate::widget::{Element, Renderer, Row};

/// The word.
pub const WORD: &str = "COINCUBE";
/// `letter-spacing: .08em`, in ems of the font size.
pub const TRACKING: f32 = 0.08;

/// The wordmark at `size` in the theme's ink.
pub fn wordmark<'a, M: 'a>(size: f32) -> Wordmark<'a, M> {
    wordmark_in(size, |theme: &Theme| theme.colors.shell.ink)
}

/// The wordmark at `size` in one colour that `color` picks per theme (the
/// bar's brand turns the accent ink on hover).
pub fn wordmark_in<'a, M: 'a>(size: f32, color: impl Fn(&Theme) -> Color + 'a) -> Wordmark<'a, M> {
    let color: Rc<dyn Fn(&Theme) -> Color + 'a> = Rc::new(color);
    let glyphs = WORD.chars().map(|glyph| {
        let color = color.clone();
        iced::widget::text(glyph.to_string())
            .font(font::SPACE_GROTESK_BOLD)
            .size(size)
            .style(move |theme: &Theme| iced::widget::text::Style {
                color: Some(color(theme)),
            })
            .into()
    });
    Wordmark {
        content: Row::with_children(glyphs).spacing(size * TRACKING).into(),
    }
}

pub struct Wordmark<'a, M> {
    content: Element<'a, M>,
}

impl<M> Widget<M, Theme, Renderer> for Wordmark<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let content = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(content.size(), vec![content])
    }

    /// One text for the whole word, not eight letters.
    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.text(None, layout.bounds(), WORD);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let content = layout.children().next().expect("the glyphs");
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            content,
            cursor,
            viewport,
        );
    }
}

impl<'a, M: 'a> From<Wordmark<'a, M>> for Element<'a, M> {
    fn from(wordmark: Wordmark<'a, M>) -> Self {
        Element::new(wordmark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::renderer::Headless;
    use iced::advanced::widget::operation::Outcome;

    fn renderer() -> Renderer {
        for bytes in font::load() {
            iced::advanced::graphics::text::font_system()
                .write()
                .unwrap()
                .load_font(bytes);
        }
        iced::futures::executor::block_on(<Renderer as Headless>::new(
            font::REGULAR,
            14.into(),
            Some("tiny-skia"),
        ))
        .unwrap()
    }

    fn laid(size: f32, renderer: &Renderer) -> (Element<'static, ()>, Tree, layout::Node) {
        let mut element: Element<'static, ()> = wordmark(size).into();
        let mut tree = Tree::new(element.as_widget());
        let node = element.as_widget_mut().layout(
            &mut tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 100.0)),
        );
        (element, tree, node)
    }

    /// The glyphs' natural width, untracked, at `size`.
    fn untracked_width(size: f32, renderer: &Renderer) -> f32 {
        WORD.chars()
            .map(|glyph| {
                let mut element: Element<'static, ()> = iced::widget::text(glyph.to_string())
                    .font(font::SPACE_GROTESK_BOLD)
                    .size(size)
                    .into();
                let mut tree = Tree::new(element.as_widget());
                element
                    .as_widget_mut()
                    .layout(
                        &mut tree,
                        renderer,
                        &layout::Limits::new(Size::ZERO, Size::new(400.0, 100.0)),
                    )
                    .size()
                    .width
            })
            .sum()
    }

    #[test]
    fn the_word_is_tracked_by_eight_hundredths_of_an_em() {
        let renderer = renderer();
        for size in [10.0, 15.0, 20.0, 28.0] {
            let (_, _, node) = laid(size, &renderer);
            let gaps = (WORD.len() - 1) as f32;
            let expected = untracked_width(size, &renderer) + gaps * size * TRACKING;
            assert!(
                (node.size().width - expected).abs() < 0.01,
                "{size}: {} vs {expected}",
                node.size().width
            );
        }
    }

    #[derive(Default)]
    struct Texts(Vec<(String, Rectangle)>);

    impl Operation for Texts {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }

        fn text(
            &mut self,
            _id: Option<&iced::advanced::widget::Id>,
            bounds: Rectangle,
            text: &str,
        ) {
            self.0.push((text.to_owned(), bounds));
        }

        fn finish(&self) -> Outcome<()> {
            Outcome::None
        }
    }

    #[test]
    fn operations_see_one_word_over_the_whole_mark() {
        let renderer = renderer();
        let (mut element, mut tree, node) = laid(15.0, &renderer);
        let mut texts = Texts::default();
        element
            .as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut texts);
        assert_eq!(texts.0, vec![(WORD.to_owned(), node.bounds())]);
    }

    /// Every drawn pixel lies between the background and the ink: one colour,
    /// antialiased, with no second tone anywhere in the word.
    #[test]
    fn the_word_is_one_ink_in_both_themes() {
        for theme in [Theme::dark(), Theme::light()] {
            let mut renderer = renderer();
            let (element, tree, node) = laid(28.0, &renderer);
            let size = node.size();
            let (width, height) = (size.width.ceil() as u32, size.height.ceil() as u32);
            let background = theme.colors.shell.rail;
            let ink = theme.colors.shell.ink;
            element.as_widget().draw(
                &tree,
                &mut renderer,
                &theme,
                &renderer::Style { text_color: ink },
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &Rectangle::with_size(size),
            );
            let pixels =
                Headless::screenshot(&mut renderer, Size::new(width, height), 1.0, background);
            let channel = |c: f32| c * 255.0;
            let (bg, fg) = (
                [
                    channel(background.r),
                    channel(background.g),
                    channel(background.b),
                ],
                [channel(ink.r), channel(ink.g), channel(ink.b)],
            );
            // How far a pixel is along background → ink, from the channel that
            // moves most, and whether it lies on that line at all.
            let (axis, span) = (0..3)
                .map(|i| (i, fg[i] - bg[i]))
                .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
                .unwrap();
            let along = |x: usize, y: usize| -> f32 {
                let at = (y * width as usize + x) * 4;
                let p = [
                    f32::from(pixels[at]),
                    f32::from(pixels[at + 1]),
                    f32::from(pixels[at + 2]),
                ];
                let t = ((p[axis] - bg[axis]) / span).clamp(0.0, 1.0);
                for i in 0..3 {
                    let expected = bg[i] + t * (fg[i] - bg[i]);
                    assert!(
                        (p[i] - expected).abs() <= 6.0,
                        "{:?}: pixel {p:?} is off the background-to-ink line",
                        theme.colors.shell.ink
                    );
                }
                t
            };
            let mut inked = 0;
            for y in 0..height as usize {
                for x in 0..width as usize {
                    if along(x, y) > 0.9 {
                        inked += 1;
                    }
                }
            }
            // Each letter's stems reach the full ink: no letter is a second,
            // lighter or darker tone of the same hue.
            let glyphs = Layout::new(&node).children().next().unwrap();
            for (letter, glyph) in WORD.chars().zip(glyphs.children()) {
                let b = glyph.bounds();
                let mut full = 0.0f32;
                for y in b.y as usize..(b.y + b.height) as usize {
                    for x in b.x.ceil() as usize..((b.x + b.width) as usize).min(width as usize) {
                        full = full.max(along(x, y));
                    }
                }
                assert!(full > 0.95, "{letter} peaks at {full} of the ink");
            }
            assert!(inked > 50, "the word drew ({inked} full-ink pixels)");
        }
    }
}
