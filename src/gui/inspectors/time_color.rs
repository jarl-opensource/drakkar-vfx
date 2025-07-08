use gpui::prelude::*;
use gpui::{Context, Entity, Window, div, px};

// ====================
// Editor.
// ====================
use crate::gui::inspectors::{Inspector, InspectorEvent};
use crate::gui::models::color::HdrColor;
use crate::gui::primitives::color_picker_input::{
    ColorPicker,
    SizeVariant as ColorPickerSizeVariant,
};
use crate::gui::primitives::events::{ColorPickerEvent, SliderEvent};
use crate::gui::primitives::slider::{SizeVariant as SliderSizeVariant, Slider};
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TimeColor
{
    pub t:     f32,
    pub color: HdrColor,
}

impl TimeColor
{
    pub fn new(t: f32, color: HdrColor) -> Self
    {
        Self { t, color }
    }
}

pub struct TimeColorInspector
{
    time_slider:    Entity<Slider>,
    color_picker:   Entity<ColorPicker>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Inspector for TimeColorInspector
{
    type Value = TimeColor;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let time_slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(initial.t)
                .with_range(0.0, 1.0) // Time range from 0 to 1
                .with_step(0.01)
                .with_size_variant(SliderSizeVariant::Small)
        });

        let color_picker = cx.new(|cx| {
            ColorPicker::new(cx)
                .with_color(initial.color, cx)
                .with_size_variant(ColorPickerSizeVariant::Small)
        });

        // Subscribe to all input events
        let mut subscriptions = Vec::new();

        // Subscribe to time slider changes
        let time_subscription = cx.subscribe(
            &time_slider,
            |this, _slider, event: &SliderEvent, cx| match event {
                SliderEvent::ValueChanged(_) => {
                    cx.emit(InspectorEvent::Updated {
                        v: this.get_value(cx),
                    });
                }
            },
        );
        subscriptions.push(time_subscription);

        // Subscribe to color picker changes
        let color_subscription = cx.subscribe(
            &color_picker,
            |this, _picker, event: &ColorPickerEvent, cx| match event {
                ColorPickerEvent::ColorChanged(_) | ColorPickerEvent::ValuesChanged { .. } => {
                    cx.emit(InspectorEvent::Updated {
                        v: this.get_value(cx),
                    });
                }
                _ => {}
            },
        );
        subscriptions.push(color_subscription);

        Self {
            time_slider,
            color_picker,
            _subscriptions: subscriptions,
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        let time = self.time_slider.read(cx).get_value().max(0.0);
        let color = self.color_picker.read(cx).get_hdr_color();

        TimeColor { t: time, color }
    }
}

impl Render for TimeColorInspector
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
                    .w(px(180.0))
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
            .child(div().flex_1().child(self.color_picker.clone()))
    }
}
