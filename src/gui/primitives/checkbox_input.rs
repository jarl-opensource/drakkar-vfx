use std::sync::Arc;

use gpui::prelude::*;
use gpui::{
    BoxShadow,
    Context,
    FocusHandle,
    Focusable,
    IntoElement,
    ParentElement,
    Styled,
    Window,
    div,
    img,
    point,
    px,
};

// ====================
// Editor.
// ====================
use crate::gui::primitives::events::*;
use crate::gui::styling::colors::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SizeVariant
{
    Small,
    #[default]
    Medium,
    Large,
}

pub struct Checkbox
{
    focus_handle: FocusHandle,
    checked:      bool,
    disabled:     bool,
    size_variant: SizeVariant,
}

impl Checkbox
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        Self {
            focus_handle: cx.focus_handle(),
            checked:      false,
            disabled:     false,
            size_variant: SizeVariant::default(),
        }
    }

    pub fn with_checked(mut self, checked: bool) -> Self
    {
        self.checked = checked;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self
    {
        self.disabled = disabled;
        self
    }

    pub fn with_size_variant(mut self, size_variant: SizeVariant) -> Self
    {
        self.size_variant = size_variant;
        self
    }

    pub fn is_checked(&self) -> bool
    {
        self.checked
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>)
    {
        if !self.disabled {
            self.checked = !self.checked;
            cx.emit(CheckboxEvent::Changed(self.checked));
            cx.notify();
        }
    }

    fn on_click(&mut self, _: &gpui::ClickEvent, _window: &mut Window, cx: &mut Context<Self>)
    {
        self.toggle(cx);
    }
}

impl Render for Checkbox
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_focused = self.focus_handle.is_focused(window);
        let is_disabled = self.disabled;
        let is_checked = self.checked;

        let size = match self.size_variant {
            SizeVariant::Small => px(16.),
            SizeVariant::Medium => px(20.),
            SizeVariant::Large => px(24.),
        };

        let icon_size = match self.size_variant {
            SizeVariant::Small => px(12.),
            SizeVariant::Medium => px(14.),
            SizeVariant::Large => px(18.),
        };

        let bg_color = if is_disabled {
            surface_muted()
        } else if is_checked {
            accent_blue()
        } else {
            surface_elevated()
        };

        let border_color = if is_focused && !is_disabled {
            border_focus()
        } else if is_disabled {
            border_muted()
        } else if is_checked {
            accent_blue()
        } else {
            border_default()
        };

        div()
            .id("checkbox")
            .size(size)
            .flex()
            .items_center()
            .justify_center()
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .rounded_sm()
            .cursor_pointer()
            .when(!is_disabled, |el| {
                el.hover(|el| {
                    el.bg(if is_checked {
                        accent_blue()
                    } else {
                        hover_overlay()
                    })
                    .border_color(if is_checked {
                        accent_blue()
                    } else {
                        border_active()
                    })
                })
                .active(|el| {
                    el.bg(if is_checked {
                        accent_blue()
                    } else {
                        hover_overlay()
                    })
                })
            })
            .when(is_focused && !is_disabled, |el| {
                el.shadow(vec![BoxShadow {
                    color:         shadow_medium(),
                    offset:        point(px(0.), px(0.)),
                    blur_radius:   px(0.),
                    spread_radius: px(2.),
                }])
            })
            .on_click(cx.listener(Self::on_click))
            .key_context("Checkbox")
            .track_focus(&self.focus_handle)
            .when(is_checked, |el| {
                el.child(
                    img(gpui::ImageSource::Image(Arc::new(gpui::Image::from_bytes(
                        gpui::ImageFormat::Svg,
                        include_bytes!("../../../assets/check.svg").to_vec(),
                    ))))
                    .size(icon_size)
                    .text_color(if is_disabled {
                        text_muted()
                    } else {
                        text_on_primary()
                    }),
                )
            })
    }
}

impl Focusable for Checkbox
{
    fn focus_handle(&self, _cx: &gpui::App) -> gpui::FocusHandle
    {
        self.focus_handle.clone()
    }
}
