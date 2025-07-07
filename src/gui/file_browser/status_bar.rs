use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{BoxShadow, Context, EventEmitter, IntoElement, Window, div, point, px};

use crate::gui::file_browser::events::StatusBarEvent;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::*;

pub struct StatusBar
{
    pub assets_root: PathBuf,
    pub is_git_repo: bool,
}

impl StatusBar
{
    pub fn new(assets_root: PathBuf, is_git_repo: bool) -> Self
    {
        Self {
            assets_root,
            is_git_repo,
        }
    }

    pub fn update_git_repo(&mut self, is_git_repo: bool)
    {
        self.is_git_repo = is_git_repo;
    }
}

impl Render for StatusBar
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
            .shadow(vec![BoxShadow {
                color:         shadow_light(),
                offset:        point(px(0.), px(-1.)),
                blur_radius:   px(2.),
                spread_radius: px(0.),
            }])
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(render_scm_status(self.is_git_repo))
                    .child(render_assets_root(&self.assets_root)),
            )
            .child(
                div()
                    .w(px(24.))
                    .h(px(24.))
                    .rounded_md()
                    .bg(button_secondary())
                    .border_1()
                    .border_color(border_subtle())
                    .shadow(vec![BoxShadow {
                        color:         shadow_light(),
                        offset:        point(px(0.), px(1.)),
                        blur_radius:   px(1.),
                        spread_radius: px(0.),
                    }])
                    .cursor_pointer()
                    .hover(|el| {
                        el.bg(button_primary())
                            .border_color(accent_blue_light())
                            .shadow(vec![
                                BoxShadow {
                                    color:         shadow_light(),
                                    offset:        point(px(0.), px(1.)),
                                    blur_radius:   px(1.),
                                    spread_radius: px(0.),
                                },
                                BoxShadow {
                                    color:         accent_blue_light().into(),
                                    offset:        point(px(0.), px(0.)),
                                    blur_radius:   px(4.),
                                    spread_radius: px(0.),
                                },
                            ])
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|_this, _event, _window, cx| {
                            cx.emit(StatusBarEvent::RefreshRequested);
                        }),
                    )
                    .child(
                        ProductIcon::FolderSync
                            .to_svg()
                            .size_3()
                            .text_color(text_primary()),
                    ),
            )
    }
}

impl EventEmitter<StatusBarEvent> for StatusBar {}

fn render_scm_status(is_detected: bool) -> impl IntoElement
{
    let (icon, text, color) = if is_detected {
        (
            ProductIcon::GitCommitHorizontal,
            "GIT: connected",
            text_success(),
        )
    } else {
        (
            ProductIcon::OctagonAlert,
            "GIT: not detected",
            text_warning(),
        )
    };

    div()
        .flex()
        .items_center()
        .gap_2()
        .child(icon.to_svg().size_3().text_color(color))
        .child(
            with_default_font(div())
                .text_xs()
                .text_color(color)
                .child(text),
        )
}

fn render_assets_root(root: &PathBuf) -> impl IntoElement
{
    div().flex().items_center().gap_2().child(
        with_default_font(div())
            .text_xs()
            .text_color(text_muted())
            .child(root.display().to_string()),
    )
}
