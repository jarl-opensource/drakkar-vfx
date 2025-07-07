use gpui::prelude::*;
use gpui::{
    BoxShadow,
    Context,
    Entity,
    FocusHandle,
    Focusable,
    IntoElement,
    MouseDownEvent,
    ParentElement,
    Rgba,
    SharedString,
    Styled,
    Window,
    deferred,
    div,
    linear_color_stop,
    linear_gradient,
    px,
    rgba,
};

// ====================
// Editor.
// ====================
use crate::gui::models::color::HdrColor;
use crate::gui::primitives::events::{ColorPickerEvent, SliderEvent};
use crate::gui::primitives::slider::{SizeVariant as SliderSizeVariant, Slider};
use crate::gui::styling::colors::*;

pub mod actions
{
    use gpui::actions;
    actions!(color_picker, [Escape, Enter]);
}

use actions::{Enter, Escape};

fn render_color_ramp(colors: &[Rgba]) -> Vec<impl IntoElement>
{
    if colors.len() < 2 {
        return vec![];
    }

    colors
        .windows(2)
        .map(|pair| {
            div().flex_1().h_full().bg(linear_gradient(
                90.,
                linear_color_stop(pair[0], 0.0),
                linear_color_stop(pair[1], 1.0),
            ))
        })
        .collect()
}

pub struct ColorPicker
{
    pub focus_handle: FocusHandle,
    pub hdr_color:    HdrColor,
    pub is_open:      bool,
    pub size_variant: SizeVariant,
    alpha_slider:     Entity<Slider>,
    intensity_slider: Entity<Slider>,
    featured_colors:  Vec<Rgba>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SizeVariant
{
    Small,
    #[default]
    Medium,
    Large,
}

fn color_palettes() -> Vec<Vec<Rgba>>
{
    vec![
        // Grays
        vec![
            rgba(0xffffffff),
            rgba(0xe5e5e5ff),
            rgba(0xccccccff),
            rgba(0xb3b3b3ff),
            rgba(0x999999ff),
            rgba(0x808080ff),
            rgba(0x666666ff),
            rgba(0x4d4d4dff),
            rgba(0x333333ff),
            rgba(0x1a1a1aff),
            rgba(0x000000ff),
        ],
        // Reds
        vec![
            rgba(0xffebebff),
            rgba(0xffc9c9ff),
            rgba(0xff9999ff),
            rgba(0xff6666ff),
            rgba(0xff3333ff),
            rgba(0xff0000ff),
            rgba(0xcc0000ff),
            rgba(0x990000ff),
            rgba(0x660000ff),
            rgba(0x330000ff),
        ],
        // Oranges
        vec![
            rgba(0xfff0e6ff),
            rgba(0xffd9b3ff),
            rgba(0xffbf80ff),
            rgba(0xffa64dff),
            rgba(0xff8c1aff),
            rgba(0xff6600ff),
            rgba(0xcc5200ff),
            rgba(0x993d00ff),
            rgba(0x662900ff),
            rgba(0x331400ff),
        ],
        // Yellows
        vec![
            rgba(0xfffce6ff),
            rgba(0xfff7b3ff),
            rgba(0xfff280ff),
            rgba(0xffed4dff),
            rgba(0xffe81aff),
            rgba(0xffe300ff),
            rgba(0xccb600ff),
            rgba(0x998a00ff),
            rgba(0x665d00ff),
            rgba(0x332f00ff),
        ],
        // Greens
        vec![
            rgba(0xe6ffe6ff),
            rgba(0xb3ffb3ff),
            rgba(0x80ff80ff),
            rgba(0x4dff4dff),
            rgba(0x1aff1aff),
            rgba(0x00ff00ff),
            rgba(0x00cc00ff),
            rgba(0x009900ff),
            rgba(0x006600ff),
            rgba(0x003300ff),
        ],
        // Cyans
        vec![
            rgba(0xe6ffffff),
            rgba(0xb3ffffff),
            rgba(0x80ffffff),
            rgba(0x4dffffff),
            rgba(0x1affffff),
            rgba(0x00ffffff),
            rgba(0x00ccccff),
            rgba(0x009999ff),
            rgba(0x006666ff),
            rgba(0x003333ff),
        ],
        // Blues
        vec![
            rgba(0xe6e6ffff),
            rgba(0xb3b3ffff),
            rgba(0x8080ffff),
            rgba(0x4d4dffff),
            rgba(0x1a1affff),
            rgba(0x0000ffff),
            rgba(0x0000ccff),
            rgba(0x000099ff),
            rgba(0x000066ff),
            rgba(0x000033ff),
        ],
        // Purples
        vec![
            rgba(0xffe6ffff),
            rgba(0xffb3ffff),
            rgba(0xff80ffff),
            rgba(0xff4dffff),
            rgba(0xff1affff),
            rgba(0xff00ffff),
            rgba(0xcc00ccff),
            rgba(0x990099ff),
            rgba(0x660066ff),
            rgba(0x330033ff),
        ],
    ]
}

impl ColorPicker
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        let default_color = HdrColor::default();

        let alpha_slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(default_color.a())
                .with_range(0.0, 1.0)
                .with_step(0.01)
                .with_size_variant(SliderSizeVariant::Small)
        });

        let intensity_slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(default_color.i())
                .with_range(0.0, 10.0)
                .with_step(0.01)
                .with_size_variant(SliderSizeVariant::Small)
        });

        // Subscribe to alpha slider changes
        cx.subscribe(&alpha_slider, |this, _slider, _event: &SliderEvent, cx| {
            this.update_color_from_sliders(cx);
            cx.emit(ColorPickerEvent::ValuesChanged {
                color:       this.hdr_color.with_alpha(),
                intensity:   this.hdr_color.i(),
                final_color: this.hdr_color.gpui(),
            });
            cx.notify();
        })
        .detach();

        // Subscribe to intensity slider changes
        cx.subscribe(
            &intensity_slider,
            |this, _slider, _event: &SliderEvent, cx| {
                this.update_color_from_sliders(cx);
                cx.emit(ColorPickerEvent::ValuesChanged {
                    color:       this.hdr_color.with_alpha(),
                    intensity:   this.hdr_color.i(),
                    final_color: this.hdr_color.gpui(),
                });
                cx.notify();
            },
        )
        .detach();

        Self {
            focus_handle: cx.focus_handle(),
            hdr_color: default_color,
            is_open: false,
            size_variant: SizeVariant::default(),
            alpha_slider,
            intensity_slider,
            featured_colors: vec![
                rgba(0x000000ff), // Black
                rgba(0x666666ff), // Gray
                rgba(0xccccccff), // Light Gray
                rgba(0xffffffff), // White
                rgba(0xff0000ff), // Red
                rgba(0xff8c00ff), // Orange
                rgba(0xffff00ff), // Yellow
                rgba(0x00ff00ff), // Green
                rgba(0x0000ffff), // Blue
                rgba(0x4b0082ff), // Indigo
                rgba(0x9400d3ff), // Purple
            ],
        }
    }

    pub fn with_color(mut self, color: HdrColor, cx: &mut Context<Self>) -> Self
    {
        self.hdr_color = color;
        self.update_inputs_from_color(cx);
        self
    }

    pub fn with_size_variant(mut self, size_variant: SizeVariant) -> Self
    {
        self.size_variant = size_variant;
        self
    }

    pub fn get_hdr_color(&self) -> HdrColor
    {
        self.hdr_color
    }

    pub fn get_color(&self) -> Rgba
    {
        self.hdr_color.with_alpha()
    }

    pub fn get_intensity(&self) -> f32
    {
        self.hdr_color.i()
    }

    pub fn get_final_color(&self) -> Rgba
    {
        self.hdr_color.gpui()
    }

    pub fn get_color_with_alpha(&self) -> Rgba
    {
        self.hdr_color.with_alpha()
    }

    pub fn get_base_color(&self) -> Rgba
    {
        self.hdr_color.base_color()
    }

    fn update_inputs_from_color(&mut self, cx: &mut Context<Self>)
    {
        // Update alpha slider with current color alpha
        self.alpha_slider.update(cx, |slider, cx| {
            slider.value = self.hdr_color.a();
            cx.notify();
        });

        // Update intensity slider with current intensity
        self.intensity_slider.update(cx, |slider, cx| {
            slider.value = self.hdr_color.i();
            cx.notify();
        });
    }

    fn update_color_from_sliders(&mut self, cx: &Context<Self>)
    {
        let a = self.alpha_slider.read(cx).get_value();
        let intensity = self.intensity_slider.read(cx).get_value();

        self.hdr_color.set_a(a);
        self.hdr_color.set_i(intensity);
    }

    fn on_click(&mut self, _ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>)
    {
        if self.is_open {
            window.blur();
            self.is_open = false;
        } else {
            window.focus(&mut self.focus_handle);
            self.is_open = true;
        }
        cx.notify();
    }

    fn on_escape(&mut self, _: &Escape, window: &mut Window, cx: &mut Context<Self>)
    {
        if self.is_open {
            self.is_open = false;
            window.blur();
            cx.notify();
        } else {
            // Propagate the event if the panel is not open
            cx.propagate();
        }
    }

    fn on_enter(&mut self, _action: &Enter, _window: &mut Window, cx: &mut Context<Self>)
    {
        self.is_open = false;
        cx.notify();
    }

    fn on_mouse_down_out(
        &mut self,
        _ev: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    )
    {
        if self.is_open {
            self.is_open = false;
            window.blur();
            cx.notify();
        }
    }

    fn on_hue_click(&mut self, color: Rgba, _window: &mut Window, cx: &mut Context<Self>)
    {
        self.hdr_color.set_r(color.r);
        self.hdr_color.set_g(color.g);
        self.hdr_color.set_b(color.b);

        self.update_inputs_from_color(cx);
        cx.emit(ColorPickerEvent::ValuesChanged {
            color:       self.hdr_color.with_alpha(),
            intensity:   self.hdr_color.i(),
            final_color: self.hdr_color.gpui(),
        });
        cx.notify();
    }

    fn on_color_click(&mut self, color: Rgba, window: &mut Window, cx: &mut Context<Self>)
    {
        self.hdr_color.set_r(color.r);
        self.hdr_color.set_g(color.g);
        self.hdr_color.set_b(color.b);
        // Keep existing alpha and intensity

        self.update_inputs_from_color(cx);
        cx.emit(ColorPickerEvent::ValuesChanged {
            color:       self.hdr_color.with_alpha(),
            intensity:   self.hdr_color.i(),
            final_color: self.hdr_color.gpui(),
        });

        self.is_open = false;
        window.blur();
        cx.notify();
    }
}

impl Render for ColorPicker
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_open = self.is_open;
        let is_focused = self.focus_handle.is_focused(window);

        let (size, text_size) = match self.size_variant {
            SizeVariant::Small => (px(24.), px(12.)),
            SizeVariant::Medium => (px(32.), px(14.)),
            SizeVariant::Large => (px(40.), px(16.)),
        };

        let border_color = if is_focused && is_open {
            border_focus()
        } else if is_open {
            border_focus()
        } else if is_focused {
            border_active()
        } else {
            border_default()
        };

        let mut element = div()
            .relative()
            .key_context("ColorPicker")
            .track_focus(&self.focus_handle);

        // Always attach keyboard handlers when panel is open
        if is_open {
            element = element
                .on_action(cx.listener(Self::on_escape))
                .on_action(cx.listener(Self::on_enter));
        }

        element
            .when(is_open, |el| {
                el.on_mouse_down_out(cx.listener(Self::on_mouse_down_out))
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .id("color-preview")
                            .focusable()
                            .w_full()
                            .h(size)
                            .border_2()
                            .border_color(border_color)
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|el| el.border_color(border_active()))
                            .flex()
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_click))
                            .child(
                                div()
                                    .flex_1()
                                    .h_full()
                                    .bg(self.get_base_color())
                                    .rounded_l_md()
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .h_full()
                                    .bg(self.get_color_with_alpha())
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .h_full()
                                    .bg(self.get_final_color())
                                    .rounded_r_md()
                                    .when(self.hdr_color.i() > 1.0, |el| {
                                        // Calculate shadow intensity based on how much above 1.0 we are
                                        // Range: 1.0 to 5.0 maps to 0% to 100% shadow opacity
                                        let shadow_intensity = ((self.hdr_color.i() - 1.0) / 4.0).min(1.0);
                                        let final_color = self.get_base_color();
                                        let shadow_color = Rgba {
                                            r: final_color.r,
                                            g: final_color.g,
                                            b: final_color.b,
                                            a: shadow_intensity * 0.8,
                                        };

                                        el.shadow(vec![
                                            BoxShadow {
                                                color: shadow_color.into(),
                                                offset: gpui::point(px(0.), px(0.)),
                                                blur_radius: px(20. * shadow_intensity),
                                                spread_radius: px(2. * shadow_intensity),
                                            }
                                        ])
                                    })
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(text_size)
                                    .text_color(text_muted())
                                    .child("Alpha:")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.alpha_slider.clone())
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(text_size)
                                    .text_color(text_muted())
                                    .child("HDR Intensity:")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.intensity_slider.clone())
                            ),
                    )
            )
            .when(is_open, |el| {
                el.child(
                    deferred(
                        div()
                            .occlude()
                            .on_mouse_down_out(cx.listener(Self::on_mouse_down_out))
                            .id("color-picker-panel")
                            .absolute()
                            .bottom(size)
                            .left_0()
                            .mb_1()
                            .w(px(280.))
                            .bg(surface_elevated())
                            .border_1()
                            .border_color(border_focus())
                            .rounded_md()
                            .shadow(vec![BoxShadow {
                                color:         shadow_medium().into(),
                                offset:        gpui::point(px(0.), px(2.)),
                                blur_radius:   px(8.),
                                spread_radius: px(0.),
                            }])
                            .p_3()
                            .flex()
                            .flex_col()
                            .gap_3()
                            // Featured colors section
                            .child(
                                div()
                                            .flex()
                                            .gap_1()
                                            .flex_wrap()
                                            .children(
                                                self.featured_colors.iter().enumerate().map(|(i, &color)| {
                                                    div()
                                                        .id(SharedString::from(format!("featured-color-{}", i)))
                                                        .w(px(20.))
                                                        .h(px(20.))
                                                        .bg(color)
                                                        .border_1()
                                                        .border_color(border_subtle())
                                                        .rounded_sm()
                                                        .cursor_pointer()
                                                        .hover(|el| {
                                                            el.border_color(border_active())
                                                                .shadow(vec![BoxShadow {
                                                                    color: shadow_medium().into(),
                                                                    offset: gpui::point(px(0.), px(1.)),
                                                                    blur_radius: px(2.),
                                                                    spread_radius: px(0.),
                                                                }])
                                                        })
                                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _ev: &MouseDownEvent, window, cx| {
                                                            this.on_color_click(color, window, cx);
                                                        }))
                                                })
                                            )
                            )
                            // Color palettes
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .children(
                                        color_palettes().iter().enumerate().map(|(i, palette)| {
                                            div()
                                                .flex()
                                                .gap_1()
                                                .children(
                                                    palette.iter().enumerate().map(|(j, &color)| {
                                                        div()
                                                            .id(SharedString::from(format!("palette-color-{}-{}", i, j)))
                                                            .w(px(20.))
                                                            .h(px(20.))
                                                            .bg(color)
                                                            .border_1()
                                                            .border_color(border_subtle())
                                                            .rounded_sm()
                                                            .cursor_pointer()
                                                            .hover(|el| {
                                                                el.border_color(border_active())
                                                                    .shadow(vec![BoxShadow {
                                                                        color: shadow_medium().into(),
                                                                        offset: gpui::point(px(0.), px(1.)),
                                                                        blur_radius: px(2.),
                                                                        spread_radius: px(0.),
                                                                    }])
                                                            })
                                                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _ev: &MouseDownEvent, window, cx| {
                                                                this.on_color_click(color, window, cx);
                                                            }))
                                                    })
                                                )
                                        })
                                    )
                            )
                            // Color gradient bar
                            .child(
                                div()
                                    .w_full()
                                    .h(px(80.))
                                    .rounded_md()
                                    .overflow_hidden()
                                    .flex()
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _ev: &MouseDownEvent, window, cx| {
                                            let p = autopilot::mouse::location();
                                            if let Ok(c) = autopilot::screen::get_color(p) {
                                                let color = Rgba {
                                                    r: c[0] as f32 / 255.0,
                                                    g: c[1] as f32 / 255.0,
                                                    b: c[2] as f32 / 255.0,
                                                    a: 1.0,
                                                };

                                                this.on_hue_click(color, window, cx);
                                            }
                                        }),
                                    )
                                    .children(render_color_ramp(&[
                                        Rgba {
                                            r: 1.0,
                                            g: 0.0,
                                            b: 0.0,
                                            a: 1.0,
                                        }, // Red
                                        Rgba {
                                            r: 1.0,
                                            g: 1.0,
                                            b: 0.0,
                                            a: 1.0,
                                        }, // Yellow
                                        Rgba {
                                            r: 0.0,
                                            g: 1.0,
                                            b: 0.0,
                                            a: 1.0,
                                        }, // Green
                                        Rgba {
                                            r: 0.0,
                                            g: 1.0,
                                            b: 1.0,
                                            a: 1.0,
                                        }, // Cyan
                                        Rgba {
                                            r: 0.0,
                                            g: 0.0,
                                            b: 1.0,
                                            a: 1.0,
                                        }, // Blue
                                        Rgba {
                                            r: 1.0,
                                            g: 0.0,
                                            b: 1.0,
                                            a: 1.0,
                                        }, // Magenta
                                        Rgba {
                                            r: 1.0,
                                            g: 0.0,
                                            b: 0.0,
                                            a: 1.0,
                                        }, // Red (complete the circle)
                                    ])),
                            ),
                    )
                    .with_priority(1),
                )
            })
    }
}

impl Focusable for ColorPicker
{
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle
    {
        self.focus_handle.clone()
    }
}
