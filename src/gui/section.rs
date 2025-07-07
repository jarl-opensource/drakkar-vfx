// ====================
// GPUI.
// ====================

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyView,
    BoxShadow,
    Context,
    Entity,
    InteractiveElement,
    IntoElement,
    ParentElement,
    Render,
    Rgba,
    SharedString,
    Styled,
    div,
    px,
};

// ====================
// Editor.
// ====================
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::*;

pub trait IntoElementClonable: IntoElement + Clone {}

/// Block section component for grouping related blocks.
///
pub struct BlockSection
{
    title:       SharedString,
    children:    Vec<AnyView>,
    is_expanded: bool,
}

impl BlockSection
{
    pub fn new(title: impl Into<SharedString>, _cx: &mut Context<Self>) -> Self
    {
        Self {
            title:       title.into(),
            children:    Vec::new(),
            is_expanded: true,
        }
    }

    pub fn with_expanded(mut self, is_expanded: bool) -> Self
    {
        self.is_expanded = is_expanded;
        self
    }

    pub fn is_expanded(&self) -> bool
    {
        self.is_expanded
    }

    pub fn clear(&mut self)
    {
        self.children.clear();
    }

    pub fn try_add_child<T: Render>(&mut self, el: &Option<Entity<T>>)
    {
        if let Some(el) = el {
            self.children.push(AnyView::from(el.clone()));
        }
    }

    fn toggle_expanded(&mut self, cx: &mut Context<Self>)
    {
        self.is_expanded = !self.is_expanded;
        cx.notify();
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .bg(if self.is_expanded {
                Rgba {
                    r: background_primary().r,
                    g: background_primary().g,
                    b: background_primary().b,
                    a: 1.0,
                }
            } else {
                Rgba {
                    r: surface_elevated().r,
                    g: surface_elevated().g,
                    b: surface_elevated().b,
                    a: 0.92,
                }
            })
            .border_1()
            .border_color(if self.is_expanded {
                Rgba {
                    r: border_default().r,
                    g: border_default().g,
                    b: border_default().b,
                    a: 0.85,
                }
            } else {
                Rgba {
                    r: border_subtle().r,
                    g: border_subtle().g,
                    b: border_subtle().b,
                    a: 0.55,
                }
            })
            .rounded_md()
            .when(!self.is_expanded, |el| el.rounded_b_md())
            .cursor_pointer()
            .shadow(vec![BoxShadow {
                color:         if self.is_expanded {
                    shadow_medium()
                } else {
                    shadow_light()
                },
                offset:        gpui::point(px(0.), px(1.)),
                blur_radius:   if self.is_expanded { px(4.) } else { px(2.) },
                spread_radius: px(0.),
            }])
            .hover(|el| {
                el.bg(if self.is_expanded {
                    Rgba {
                        r: background_darker().r,
                        g: background_darker().g,
                        b: background_darker().b,
                        a: 1.0,
                    }
                } else {
                    Rgba {
                        r: hover_overlay().r,
                        g: hover_overlay().g,
                        b: hover_overlay().b,
                        a: 0.85,
                    }
                })
                .border_color(if self.is_expanded {
                    Rgba {
                        r: accent_blue_light().r,
                        g: accent_blue_light().g,
                        b: accent_blue_light().b,
                        a: 0.5,
                    }
                } else {
                    Rgba {
                        r: border_active().r,
                        g: border_active().g,
                        b: border_active().b,
                        a: 0.7,
                    }
                })
            })
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.toggle_expanded(cx);
                }),
            )
            .child(
                div()
                    .size_5()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .bg(if self.is_expanded {
                        Rgba {
                            r: accent_blue_light().r,
                            g: accent_blue_light().g,
                            b: accent_blue_light().b,
                            a: 0.12,
                        }
                    } else {
                        Rgba {
                            r: surface_elevated().r,
                            g: surface_elevated().g,
                            b: surface_elevated().b,
                            a: 0.75,
                        }
                    })
                    .border_1()
                    .border_color(if self.is_expanded {
                        Rgba {
                            r: accent_blue_light().r,
                            g: accent_blue_light().g,
                            b: accent_blue_light().b,
                            a: 0.25,
                        }
                    } else {
                        Rgba {
                            r: border_subtle().r,
                            g: border_subtle().g,
                            b: border_subtle().b,
                            a: 0.45,
                        }
                    })
                    .child(if self.is_expanded {
                        ProductIcon::ChevronUp.to_svg().size_3().text_color(Rgba {
                            r: accent_blue_light().r,
                            g: accent_blue_light().g,
                            b: accent_blue_light().b,
                            a: 0.9,
                        })
                    } else {
                        ProductIcon::ChevronDown.to_svg().size_3().text_color(Rgba {
                            r: text_muted().r,
                            g: text_muted().g,
                            b: text_muted().b,
                            a: 0.8,
                        })
                    }),
            )
            .child(
                with_default_font(div())
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(if self.is_expanded {
                        text_primary()
                    } else {
                        Rgba {
                            r: text_secondary().r,
                            g: text_secondary().g,
                            b: text_secondary().b,
                            a: 0.85,
                        }
                    })
                    .child(self.title.clone()),
            )
    }

    fn render_content(&self) -> impl IntoElement
    {
        div()
            .flex()
            .flex_col()
            .bg(Rgba {
                r: background_darker().r,
                g: background_darker().g,
                b: background_darker().b,
                a: 0.08,
            })
            .border_x_1()
            .border_b_1()
            .border_color(Rgba {
                r: border_subtle().r,
                g: border_subtle().g,
                b: border_subtle().b,
                a: 0.35,
            })
            .rounded_b_md()
            .pt_1()
            .pb_1()
            .shadow(vec![BoxShadow {
                color:         shadow_light(),
                offset:        gpui::point(px(0.), px(1.)),
                blur_radius:   px(4.),
                spread_radius: px(0.),
            }])
            .children(self.children.iter().cloned())
    }
}

impl Render for BlockSection
{
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .my_2()
            .child(self.render_header(cx))
            .when(self.is_expanded, |el| el.child(self.render_content()))
    }
}
