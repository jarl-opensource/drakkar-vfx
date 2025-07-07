use gpui::prelude::*;
use gpui::{Context, FontWeight, IntoElement, ParentElement, Styled, Window, div, px};
use open;

use crate::gui::styling::colors::*;
use crate::gui::styling::icons::ProductIcon;

pub struct AppStatusBar;

impl AppStatusBar
{
    pub fn new() -> Self
    {
        Self
    }
}

impl Render for AppStatusBar
{
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let version = env!("CARGO_PKG_VERSION");
        let git_sha = option_env!("GIT_SHA").unwrap_or("unknown");

        div()
            .id("app-status-bar")
            .w_full()
            .h(px(24.0)) // Smaller height
            .flex()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .pt(px(8.0)) // More top padding
            .pb(px(8.0)) // More bottom padding
            .bg(background_primary())
            .border_b_1()
            .border_color(border_separator())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_color(text_primary())
                            .text_size(px(14.0))
                            .font_weight(FontWeight::BOLD)
                            .child("Drakkar VFX"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_color(text_secondary())
                                    .text_size(px(11.0))
                                    .child(format!("v{}", version)),
                            )
                            .child(
                                div()
                                    .text_color(text_muted())
                                    .text_size(px(10.0))
                                    .child(format!("{}", git_sha)),
                            ),
                    ),
            )
            .child(
                div().flex().items_center().gap_2().child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_color(text_secondary())
                                .text_size(px(11.0))
                                .child("Social"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(16.0))
                                .h(px(16.0))
                                .rounded_md()
                                .bg(gpui::Rgba {
                                    r: 0.2,
                                    g: 0.2,
                                    b: 0.2,
                                    a: 1.0,
                                })
                                .hover(|style| {
                                    style
                                        .bg(gpui::Rgba {
                                            r: 0.3,
                                            g: 0.3,
                                            b: 0.3,
                                            a: 1.0,
                                        })
                                        .shadow(vec![gpui::BoxShadow {
                                            color:         gpui::Rgba {
                                                r: 0.2,
                                                g: 0.2,
                                                b: 0.2,
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
                                        if let Err(e) = open::that(
                                            "https://github.com/jarl-opensource/drakkar-vfx",
                                        ) {
                                            eprintln!("Failed to open GitHub URL: {}", e);
                                        }
                                    }),
                                )
                                .child(ProductIcon::Github.to_svg().size_3()),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(16.0))
                                .h(px(16.0))
                                .rounded_md()
                                .bg(gpui::Rgba {
                                    r: 0.9,
                                    g: 0.2,
                                    b: 0.2,
                                    a: 1.0,
                                })
                                .hover(|style| {
                                    style
                                        .bg(gpui::Rgba {
                                            r: 1.0,
                                            g: 0.3,
                                            b: 0.3,
                                            a: 1.0,
                                        })
                                        .shadow(vec![gpui::BoxShadow {
                                            color:         gpui::Rgba {
                                                r: 0.9,
                                                g: 0.2,
                                                b: 0.2,
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
                                        if let Err(e) =
                                            open::that("https://www.youtube.com/@Jarl-Game-com")
                                        {
                                            eprintln!("Failed to open YouTube URL: {}", e);
                                        }
                                    }),
                                )
                                .child(ProductIcon::Youtube.to_svg().size_3()),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(16.0))
                                .h(px(16.0))
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
                                .child(ProductIcon::Discord.to_svg().size_3()),
                        ),
                ),
            )
    }
}
