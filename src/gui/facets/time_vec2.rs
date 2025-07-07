use gpui::prelude::*;
use gpui::{Context, Entity, SharedString, Window, div, px};

// ====================
// Editor.
// ====================
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::primitives::events::{SliderEvent, TextInputEvent};
use crate::gui::primitives::slider::{SizeVariant as SliderSizeVariant, Slider};
use crate::gui::primitives::text_input::{BorderRadius, ColorVariant, SizeVariant, TextInput};
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::utils::text::ValidationMode;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TimeVec2
{
    pub t: f32,
    pub v: bevy::math::Vec2,
}

impl TimeVec2
{
    pub fn new(t: f32, x: f32, y: f32) -> Self
    {
        Self {
            t,
            v: bevy::math::Vec2::new(x, y),
        }
    }
}

pub struct TimeVec2Facet
{
    time_slider:    Entity<Slider>,
    x_input:        Entity<TextInput>,
    y_input:        Entity<TextInput>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for TimeVec2Facet
{
    type Value = TimeVec2;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let time_slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(initial.t)
                .with_range(0.0, 1.0) // Time range from 0 to 1
                .with_step(0.01)
                .with_size_variant(SliderSizeVariant::Small)
        });

        let x_input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(SharedString::new(initial.v.x.to_string()), cx)
                .with_validation_mode(ValidationMode::Float)
                .with_color_variant(ColorVariant::Subtle)
                .with_border_radius(BorderRadius::Small)
                .with_border_width(px(1.0))
                .with_size_variant(SizeVariant::Small)
                .with_full_width(true)
                .with_text_color_focused(text_primary())
                .with_text_color_unfocused(text_primary())
                .with_padding(px(4.0), px(2.0))
        });

        let y_input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(SharedString::new(initial.v.y.to_string()), cx)
                .with_validation_mode(ValidationMode::Float)
                .with_color_variant(ColorVariant::Subtle)
                .with_border_radius(BorderRadius::Small)
                .with_border_width(px(1.0))
                .with_size_variant(SizeVariant::Small)
                .with_full_width(true)
                .with_text_color_focused(text_primary())
                .with_text_color_unfocused(text_primary())
                .with_padding(px(4.0), px(2.0))
        });

        // Subscribe to all input events
        let mut subscriptions = Vec::new();

        // Subscribe to time slider changes
        let time_subscription = cx.subscribe(
            &time_slider,
            |this, _slider, event: &SliderEvent, cx| match event {
                SliderEvent::ValueChanged(_) => {
                    cx.emit(FacetEvent::Updated {
                        v: this.get_value(cx),
                    });
                }
            },
        );
        subscriptions.push(time_subscription);

        // Subscribe to x input changes
        let x_subscription =
            cx.subscribe(
                &x_input,
                |this, _input, event: &TextInputEvent, cx| match event {
                    TextInputEvent::Edited => {
                        cx.emit(FacetEvent::Updated {
                            v: this.get_value(cx),
                        });
                    }
                    _ => {}
                },
            );
        subscriptions.push(x_subscription);

        // Subscribe to y input changes
        let y_subscription =
            cx.subscribe(
                &y_input,
                |this, _input, event: &TextInputEvent, cx| match event {
                    TextInputEvent::Edited => {
                        cx.emit(FacetEvent::Updated {
                            v: this.get_value(cx),
                        });
                    }
                    _ => {}
                },
            );
        subscriptions.push(y_subscription);

        Self {
            time_slider,
            x_input,
            y_input,
            _subscriptions: subscriptions,
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        let time = self.time_slider.read(cx).get_value().max(0.0);

        let x = self
            .x_input
            .read(cx)
            .content
            .to_string()
            .parse::<f32>()
            .unwrap_or(0.0)
            .max(0.0);

        let y = self
            .y_input
            .read(cx)
            .content
            .to_string()
            .parse::<f32>()
            .unwrap_or(0.0)
            .max(0.0);

        TimeVec2 {
            t: time,
            v: bevy::math::Vec2::new(x, y),
        }
    }
}

impl Render for TimeVec2Facet
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .w(px(240.0))
                    .child(
                        with_default_font(div())
                            .text_xs()
                            .text_color(text_muted())
                            .child("t:"),
                    )
                    .child(div().flex_1().child(self.time_slider.clone())),
            )
            .child(
                div()
                    .w(px(1.0))
                    .h(px(20.0))
                    .bg(border_separator())
                    .opacity(0.5),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        with_default_font(div())
                            .text_xs()
                            .text_color(text_muted())
                            .child("x:"),
                    )
                    .child(div().w(px(60.0)).child(self.x_input.clone())),
            )
            .child(
                div()
                    .w(px(1.0))
                    .h(px(20.0))
                    .bg(border_separator())
                    .opacity(0.5),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        with_default_font(div())
                            .text_xs()
                            .text_color(text_muted())
                            .child("y:"),
                    )
                    .child(div().w(px(60.0)).child(self.y_input.clone())),
            )
    }
}
