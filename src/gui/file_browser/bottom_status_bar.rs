use gpui::prelude::*;
use gpui::{Context, IntoElement, ParentElement, Styled, Window, div, px};
use open;

use crate::gui::styling::colors::*;
use crate::gui::styling::icons::ProductIcon;

pub struct BottomStatusBar
{
    pub version: String,
    pub git_sha: String,
}

impl BottomStatusBar
{
    pub fn new() -> Self
    {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let git_sha = "unknown".to_string();
        Self { version, git_sha }
    }
}

impl Render for BottomStatusBar
{
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .bg(background_darker())
            .border_t_1()
            .border_color(border_separator())
            .text_color(text_secondary())
            .text_size(px(12.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(format!("v{}", self.version))
                    .child("•")
                    .child(format!("git:{}", self.git_sha)),
            )
            .child(
                div().flex().items_center().gap_2().child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded_md()
                        .bg(gpui::Rgba {
                            r: 0.6,
                            g: 0.4,
                            b: 0.9,
                            a: 1.0,
                        })
                        .hover(|style| {
                            style
                                .bg(gpui::Rgba {
                                    r: 0.7,
                                    g: 0.5,
                                    b: 1.0,
                                    a: 1.0,
                                })
                                .shadow(vec![gpui::BoxShadow {
                                    color:         gpui::Rgba {
                                        r: 0.6,
                                        g: 0.4,
                                        b: 0.9,
                                        a: 0.6,
                                    }
                                    .into(),
                                    offset:        gpui::point(px(0.), px(0.)),
                                    blur_radius:   px(8.),
                                    spread_radius: px(0.),
                                }])
                        })
                        .cursor_pointer()
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |_, _ev, _window, _cx| {
                                if let Err(e) = open::that("http://jarl-game.com/discord") {
                                    eprintln!("Failed to open Discord URL: {}", e);
                                }
                            }),
                        )
                        .child(ProductIcon::Discord.to_svg()),
                ),
            )
    }
}
