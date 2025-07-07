use bevy::math::Vec3;
use gpui::prelude::*;
use gpui::{Context, Entity, SharedString, Window, div, px};

// ====================
// Editor.
// ====================
use crate::gui::primitives::events::{TextInputEvent, Vec3InputEvent};
use crate::gui::primitives::text_input::{BorderRadius, ColorVariant, SizeVariant, TextInput};
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::utils::text::ValidationMode;

pub struct Vec3Input
{
    x_input:        Entity<TextInput>,
    y_input:        Entity<TextInput>,
    z_input:        Entity<TextInput>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Vec3Input
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        Self::with_value(cx, Vec3::ZERO)
    }

    pub fn with_value(cx: &mut Context<Self>, value: Vec3) -> Self
    {
        let x_input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(SharedString::new(value.x.to_string()), cx)
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
                .with_content(SharedString::new(value.y.to_string()), cx)
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

        let z_input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(SharedString::new(value.z.to_string()), cx)
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

        let mut subscriptions = Vec::new();

        // Subscribe to x input changes
        subscriptions.push(
            cx.subscribe(&x_input, |this, _, event: &TextInputEvent, cx| {
                if matches!(event, TextInputEvent::Edited) {
                    let new_value = this.get_value(cx);
                    cx.emit(Vec3InputEvent::Changed(new_value));
                }
            }),
        );

        // Subscribe to y input changes
        subscriptions.push(
            cx.subscribe(&y_input, |this, _, event: &TextInputEvent, cx| {
                if matches!(event, TextInputEvent::Edited) {
                    let new_value = this.get_value(cx);
                    cx.emit(Vec3InputEvent::Changed(new_value));
                }
            }),
        );

        // Subscribe to z input changes
        subscriptions.push(
            cx.subscribe(&z_input, |this, _, event: &TextInputEvent, cx| {
                if matches!(event, TextInputEvent::Edited) {
                    let new_value = this.get_value(cx);
                    cx.emit(Vec3InputEvent::Changed(new_value));
                }
            }),
        );

        Self {
            x_input,
            y_input,
            z_input,
            _subscriptions: subscriptions,
        }
    }

    pub fn get_value<T>(&self, cx: &Context<T>) -> Vec3
    {
        let x = self
            .x_input
            .read(cx)
            .content
            .to_string()
            .parse::<f32>()
            .unwrap_or(0.0);

        let y = self
            .y_input
            .read(cx)
            .content
            .to_string()
            .parse::<f32>()
            .unwrap_or(0.0);

        let z = self
            .z_input
            .read(cx)
            .content
            .to_string()
            .parse::<f32>()
            .unwrap_or(0.0);

        Vec3::new(x, y, z)
    }

    pub fn set_value<T>(&self, value: Vec3, cx: &mut Context<T>)
    {
        self.x_input.update(cx, |input, _cx| {
            input.content = SharedString::new(value.x.to_string());
        });
        self.y_input.update(cx, |input, _cx| {
            input.content = SharedString::new(value.y.to_string());
        });
        self.z_input.update(cx, |input, _cx| {
            input.content = SharedString::new(value.z.to_string());
        });
    }
}

impl Render for Vec3Input
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
                    .child(
                        with_default_font(div())
                            .text_xs()
                            .text_color(text_muted())
                            .child("x:"),
                    )
                    .child(div().flex_1().min_w(px(50.0)).child(self.x_input.clone())),
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
                    .child(div().flex_1().min_w(px(50.0)).child(self.y_input.clone())),
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
                            .child("z:"),
                    )
                    .child(div().flex_1().min_w(px(50.0)).child(self.z_input.clone())),
            )
    }
}
