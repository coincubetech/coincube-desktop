//! Offscreen frames of the shared top bar on an open Cube, through the
//! production wgpu renderer, for review against the `P-` boards. Opt-in only:
//! `TOP_BAR_CAPTURE_DIR=<dir> cargo test -p coincube-gui --lib top_bar_capture
//! -- --ignored`. Every Cube is a synthetic fixture.

use std::path::PathBuf;
use std::time::Instant;

use coincube_ui::{font, theme, widget::Element};
use iced::advanced::{
    layout, mouse,
    renderer::Headless,
    widget::{operation, Id, Operation, Tree},
    Layout, Renderer as _, Shell,
};
use iced::{Rectangle, Size};

use crate::app::{
    cache::Cache,
    menu::{CubeSubMenu, Menu},
    state::preferences::{BalancePrivacy, BalancePrivacyChrome},
    ConnectionStatus,
};

pub(crate) const WINDOWS: [(f32, f32); 2] = [(1024.0, 768.0), (1440.0, 900.0)];

pub(crate) fn directory() -> Option<PathBuf> {
    let directory = PathBuf::from(std::env::var("TOP_BAR_CAPTURE_DIR").ok()?);
    std::fs::create_dir_all(&directory).unwrap();
    Some(directory)
}

pub(crate) fn renderer() -> iced::Renderer {
    for bytes in font::load() {
        iced::advanced::graphics::text::font_system()
            .write()
            .unwrap()
            .load_font(bytes);
    }
    iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        font::REGULAR,
        iced::Pixels(14.0),
        Some("wgpu"),
    ))
    .expect("wgpu headless renderer")
}

/// Lays `element` out at `size`, presses Tab and focuses `focus` (so the
/// ring paints), sends the first redraw and saves the frame to `path`. Panics if
/// `focus` names no focusable control, so a frame can't be saved unfocused
/// under a focus name.
pub(crate) fn save<M>(
    renderer: &mut iced::Renderer,
    mut element: Element<'_, M>,
    size: Size,
    theme: &theme::Theme,
    focus: Option<Id>,
    path: PathBuf,
) {
    let mut tree = Tree::new(element.as_widget());
    let node =
        element
            .as_widget_mut()
            .layout(&mut tree, renderer, &layout::Limits::new(Size::ZERO, size));
    if let Some(id) = focus {
        // Arrive as a keyboard user does, after Tab: only keyboard navigation
        // paints the ring (`coincube_ui::widget::action`).
        let tab = iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab);
        let viewport = Rectangle::with_size(size);
        element.as_widget_mut().update(
            &mut tree,
            &iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: tab.clone(),
                modified_key: tab,
                physical_key: iced::keyboard::key::Physical::Unidentified(
                    iced::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced::keyboard::Location::Standard,
                modifiers: iced::keyboard::Modifiers::empty(),
                text: None,
                repeat: false,
            }),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Shell::new(&mut Vec::new()),
            &viewport,
        );
        let mut focus: Box<dyn Operation<()>> =
            Box::new(operation::focusable::focus::<()>(id.clone()));
        loop {
            element.as_widget_mut().operate(
                &mut tree,
                Layout::new(&node),
                renderer,
                &mut operation::black_box(focus.as_mut()),
            );
            match focus.finish() {
                operation::Outcome::Chain(next) => focus = next,
                _ => break,
            }
        }
        let mut focused = operation::focusable::find_focused();
        element.as_widget_mut().operate(
            &mut tree,
            Layout::new(&node),
            renderer,
            &mut operation::black_box(&mut focused),
        );
        match focused.finish() {
            operation::Outcome::Some(found) => assert_eq!(found, id, "{path:?}"),
            _ => panic!("{:?} is not a focusable control here: {:?}", id, path),
        }
    }
    draw_and_save(renderer, element, tree, node, size, theme, path);
}

/// Lays `element` out at `size`, scrolls by `scroll_by` pixels with the pointer
/// at `pointer` (as a mouse wheel does), sends the first redraw and saves the
/// frame to `path`: a page's pinned head over a scrolled body.
pub(crate) fn save_scrolled<M>(
    renderer: &mut iced::Renderer,
    mut element: Element<'_, M>,
    size: Size,
    theme: &theme::Theme,
    pointer: iced::Point,
    scroll_by: f32,
    path: PathBuf,
) {
    let mut tree = Tree::new(element.as_widget());
    let node =
        element
            .as_widget_mut()
            .layout(&mut tree, renderer, &layout::Limits::new(Size::ZERO, size));
    element.as_widget_mut().update(
        &mut tree,
        &iced::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels {
                x: 0.0,
                y: -scroll_by,
            },
        }),
        Layout::new(&node),
        mouse::Cursor::Available(pointer),
        renderer,
        &mut iced::advanced::clipboard::Null,
        &mut Shell::new(&mut Vec::new()),
        &Rectangle::with_size(size),
    );
    draw_and_save(renderer, element, tree, node, size, theme, path);
}

fn draw_and_save<M>(
    renderer: &mut iced::Renderer,
    element: Element<'_, M>,
    tree: Tree,
    node: layout::Node,
    size: Size,
    theme: &theme::Theme,
    path: PathBuf,
) {
    draw_and_save_at(
        renderer,
        element,
        tree,
        node,
        size,
        theme,
        mouse::Cursor::Unavailable,
        path,
    );
}

/// Lays `element` out at `size`, moves the pointer to `pointer` and saves the
/// frame with it there, so hover styles can be checked by eye.
pub(crate) fn save_hovered<M>(
    renderer: &mut iced::Renderer,
    mut element: Element<'_, M>,
    size: Size,
    theme: &theme::Theme,
    pointer: iced::Point,
    path: PathBuf,
) {
    let mut tree = Tree::new(element.as_widget());
    let node =
        element
            .as_widget_mut()
            .layout(&mut tree, renderer, &layout::Limits::new(Size::ZERO, size));
    let viewport = Rectangle::with_size(size);
    element.as_widget_mut().update(
        &mut tree,
        &iced::Event::Mouse(mouse::Event::CursorMoved { position: pointer }),
        Layout::new(&node),
        mouse::Cursor::Available(pointer),
        renderer,
        &mut iced::advanced::clipboard::Null,
        &mut Shell::new(&mut Vec::new()),
        &viewport,
    );
    draw_and_save_at(
        renderer,
        element,
        tree,
        node,
        size,
        theme,
        mouse::Cursor::Available(pointer),
        path,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_and_save_at<M>(
    renderer: &mut iced::Renderer,
    mut element: Element<'_, M>,
    mut tree: Tree,
    node: layout::Node,
    size: Size,
    theme: &theme::Theme,
    cursor: mouse::Cursor,
    path: PathBuf,
) {
    let viewport = Rectangle::with_size(size);
    renderer.reset(viewport);
    element.as_widget_mut().update(
        &mut tree,
        &iced::Event::Window(iced::window::Event::RedrawRequested(Instant::now())),
        Layout::new(&node),
        cursor,
        renderer,
        &mut iced::advanced::clipboard::Null,
        &mut Shell::new(&mut Vec::new()),
        &viewport,
    );
    element.as_widget().draw(
        &tree,
        renderer,
        theme,
        &iced::advanced::renderer::Style {
            text_color: theme.colors.shell.ink,
        },
        Layout::new(&node),
        cursor,
        &viewport,
    );
    let pixels = renderer.screenshot(
        Size::new(size.width as u32, size.height as u32),
        1.0,
        theme.colors.shell.background,
    );
    renderer.reset(viewport);
    image::save_buffer(
        path,
        &pixels,
        size.width as u32,
        size.height as u32,
        image::ColorType::Rgba8,
    )
    .expect("save capture");
}

pub(crate) fn themes() -> [(&'static str, theme::Theme); 2] {
    [
        ("dark", theme::Theme::dark()),
        ("light", theme::Theme::light()),
    ]
}

pub(crate) fn cache(light: bool, privacy: BalancePrivacy) -> Cache {
    Cache {
        cube_name: "Everyday".into(),
        has_vault: true,
        sidebar_connect_signed_in: true,
        connect_email: Some("qa@example.invalid".into()),
        connect_stream_status: ConnectionStatus::Connected,
        balance_privacy: Some(BalancePrivacyChrome {
            privacy,
            busy: false,
            interactive: true,
        }),
        theme_mode: if light {
            coincube_ui::theme::palette::ThemeMode::Light
        } else {
            coincube_ui::theme::palette::ThemeMode::Dark
        },
        ..Default::default()
    }
}

fn dashboard(cache: &Cache) -> Element<'_, crate::app::view::Message> {
    static MENU: Menu = Menu::Cube(CubeSubMenu::Overview);
    crate::app::view::dashboard(
        &MENU,
        cache,
        iced::widget::column![
            iced::widget::text("Total balance").size(24),
            iced::widget::text("Synthetic top bar preview").size(14),
        ],
    )
}

#[test]
#[ignore = "writes PNG frames; needs a GPU adapter"]
fn top_bar_capture_cube() {
    let Some(directory) = directory() else {
        return;
    };
    let mut renderer = renderer();
    for (appearance, palette) in themes() {
        let light = appearance == "light";
        for (privacy, state) in [
            (BalancePrivacy::Shown, "shown"),
            (BalancePrivacy::Hidden, "hidden"),
        ] {
            let cache = cache(light, privacy);
            for (width, height) in WINDOWS {
                save(
                    &mut renderer,
                    dashboard(&cache),
                    Size::new(width, height),
                    &palette,
                    None,
                    directory.join(format!("cube-{appearance}-{state}-{width}x{height}.png")),
                );
            }
        }
        // The same bar in Spanish, so long labels can be checked by eye.
        let cache = cache(light, BalancePrivacy::Shown);
        for (width, height) in WINDOWS {
            coincube_ui::locale::render_in(coincube_ui::locale::Language::Spanish, || {
                save(
                    &mut renderer,
                    dashboard(&cache),
                    Size::new(width, height),
                    &palette,
                    None,
                    directory.join(format!("cube-{appearance}-shown-es-{width}x{height}.png")),
                )
            });
        }
        // Each control's focus ring, between its neighbours and the gutter.
        for (id, name) in [
            (super::LAUNCHER_RETURN_ID, "brand"),
            (super::BALANCE_PRIVACY_ID, "eye"),
            (super::CONNECTION_ID, "connection"),
            (super::APP_PREFERENCES_ID, "preferences"),
        ] {
            save(
                &mut renderer,
                dashboard(&cache),
                Size::new(1024.0, 768.0),
                &palette,
                Some(Id::new(id)),
                directory.join(format!("cube-{appearance}-focus-{name}-1024x768.png")),
            );
        }
    }
}

/// The side panel with both wallets named, each wallet and Settings selected
/// in turn, for review against `P-Overview`, `P-Send` and `P-Settings`
/// (Tenshu v175 L2). Selection alone paints no ring; the `focus-spark` frame
/// arrives by Tab and shows the 2px ring. Synthetic fixture names only.
/// `TOP_BAR_CAPTURE_DIR=<dir> cargo test -p coincube-gui --lib
/// side_panel_capture -- --ignored`.
#[test]
#[ignore = "writes PNG frames; needs a GPU adapter"]
fn side_panel_capture() {
    use crate::app::menu::{CubeSettingsOption, SparkSubMenu};
    let Some(directory) = directory() else {
        return;
    };
    let mut renderer = renderer();
    for (appearance, palette) in themes() {
        let mut cache = cache(appearance == "light", BalancePrivacy::Shown);
        cache.wallet_names = crate::app::cache::WalletNames {
            spark: Some("Daily spending".into()),
            vault: Some("Family savings".into()),
        };
        let pages = [
            ("overview", Menu::Cube(CubeSubMenu::Overview)),
            ("send", Menu::Spark(SparkSubMenu::Send)),
            (
                "settings",
                Menu::Cube(CubeSubMenu::Settings(CubeSettingsOption::General)),
            ),
        ];
        for (page, menu) in &pages {
            for (width, height) in WINDOWS {
                save(
                    &mut renderer,
                    crate::app::view::dashboard(
                        menu,
                        &cache,
                        iced::widget::column![iced::widget::text("Synthetic page").size(24)],
                    ),
                    Size::new(width, height),
                    &palette,
                    None,
                    directory.join(format!(
                        "side-panel-{appearance}-{page}-{width}x{height}.png"
                    )),
                );
            }
        }
        // The side panel in Spanish, at the narrower window, where long
        // labels have least room.
        for (page, menu) in &pages {
            coincube_ui::locale::render_in(coincube_ui::locale::Language::Spanish, || {
                save(
                    &mut renderer,
                    crate::app::view::dashboard(
                        menu,
                        &cache,
                        iced::widget::column![iced::widget::text("Synthetic page").size(24)],
                    ),
                    Size::new(1024.0, 768.0),
                    &palette,
                    None,
                    directory.join(format!("side-panel-{appearance}-{page}-es-1024x768.png")),
                )
            });
        }
        let (_, send) = &pages[1];
        save(
            &mut renderer,
            crate::app::view::dashboard(
                send,
                &cache,
                iced::widget::column![iced::widget::text("Synthetic page").size(24)],
            ),
            Size::new(1024.0, 768.0),
            &palette,
            Some(Id::new("tenshu.shell.nav.Spark")),
            directory.join(format!("side-panel-{appearance}-focus-spark-1024x768.png")),
        );
    }
}

/// Saves `page` for the page-head review (Tenshu v175 L3): at each window,
/// once at the top and once scrolled, so the pinned head can be checked
/// over a body that has moved under it. Files are
/// `page-head-{name}-{appearance}-{width}x{height}-{top|scrolled}.png`.
pub(crate) fn save_page_head_frames<'a>(
    renderer: &mut iced::Renderer,
    directory: &std::path::Path,
    name: &str,
    appearance: &str,
    palette: &theme::Theme,
    mut page: impl FnMut() -> Element<'a, crate::app::view::Message>,
) {
    for (width, height) in WINDOWS {
        let size = Size::new(width, height);
        let file = |state: &str| {
            directory.join(format!(
                "page-head-{name}-{appearance}-{width}x{height}-{state}.png"
            ))
        };
        save(renderer, page(), size, palette, None, file("top"));
        // The pointer over the body, right of the side panel and below the head.
        let pointer = iced::Point::new(width * 0.6, height * 0.7);
        save_scrolled(
            renderer,
            page(),
            size,
            palette,
            pointer,
            600.0,
            file("scrolled"),
        );
    }
}

/// The colour the quiet region rule (`shell.rule`, ink at 10%) paints over
/// `under`, as 8-bit RGB.
pub(crate) fn quiet_rule_over(theme: &theme::Theme, under: iced::Color) -> [u8; 3] {
    let rule = theme.colors.shell.rule;
    [(rule.r, under.r), (rule.g, under.g), (rule.b, under.b)]
        .map(|(top, under)| ((top * rule.a + under * (1.0 - rule.a)) * 255.0).round() as u8)
}

/// Asserts the pixel at (`x`, `y`) of an RGBA frame `width` wide is the quiet
/// rule over `under`, within rounding.
pub(crate) fn assert_quiet_rule(
    pixels: &[u8],
    width: u32,
    (x, y): (u32, u32),
    theme: &theme::Theme,
    under: iced::Color,
    what: &str,
) {
    let i = ((y * width + x) * 4) as usize;
    let actual = [pixels[i], pixels[i + 1], pixels[i + 2]];
    let expected = quiet_rule_over(theme, under);
    assert!(
        actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
        "{} at ({}, {}) is {:?}, not the quiet rule {:?}",
        what,
        x,
        y,
        actual,
        expected
    );
}

/// Hover states by eye, for the Desktop V3 button rule: the bar buttons (top
/// bar, side panel, backup strip) take a light fill with no outline, and the
/// content buttons (the Receive/Send pair, legacy secondary and primary) keep
/// their outline or fill and add the hover. Pointer positions are fixed for
/// the 1440×900 layout. `TOP_BAR_CAPTURE_DIR=<dir> cargo test -p coincube-gui
/// --lib hover_capture -- --ignored`.
#[test]
#[ignore = "writes PNG frames; needs a GPU adapter"]
fn hover_capture() {
    use crate::app::view::Message;
    let Some(directory) = directory() else {
        return;
    };
    let mut renderer = renderer();
    let size = Size::new(1440.0, 900.0);
    let go = || Message::Menu(Menu::Cube(CubeSubMenu::Overview));
    for (appearance, palette) in themes() {
        let cache = cache(appearance == "light", BalancePrivacy::Shown);
        for (name, x, y) in [
            ("eye", 1076.0, 31.0),
            ("connect", 1240.0, 31.0),
            ("preferences", 1400.0, 31.0),
            ("nav-activity", 70.0, 183.0),
            ("backup", 100.0, 822.0),
        ] {
            save_hovered(
                &mut renderer,
                dashboard(&cache),
                size,
                &palette,
                iced::Point::new(x, y),
                directory.join(format!("hover-{appearance}-bar-{name}.png")),
            );
        }
        type Button = fn(fn() -> Message) -> Element<'static, Message>;
        let pages: [(&str, Button); 4] = [
            ("receive", |go| {
                coincube_ui::component::tenshu::action::receive_action(
                    "capture.receive",
                    Message::FocusAction,
                    Some(go()),
                )
                .into()
            }),
            ("send", |go| {
                coincube_ui::component::tenshu::action::send_action(
                    "capture.send",
                    Message::FocusAction,
                    Some(go()),
                )
                .into()
            }),
            ("legacy-secondary", |go| {
                coincube_ui::component::button::secondary(None, "Cancel")
                    .on_press(go())
                    .into()
            }),
            ("legacy-primary", |go| {
                coincube_ui::component::button::primary(None, "Continue")
                    .on_press(go())
                    .into()
            }),
        ];
        for (name, button) in pages {
            for (state, pointer) in [
                ("rest", None),
                ("hover", Some(iced::Point::new(300.0, 106.0))),
            ] {
                let page = crate::app::view::dashboard(
                    &Menu::Cube(CubeSubMenu::Overview),
                    &cache,
                    iced::widget::column![button(go)],
                );
                let path = directory.join(format!("hover-{appearance}-content-{name}-{state}.png"));
                match pointer {
                    Some(point) => save_hovered(&mut renderer, page, size, &palette, point, path),
                    None => save(&mut renderer, page, size, &palette, None, path),
                }
            }
        }
    }
}
