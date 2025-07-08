use gpui::prelude::*;
use gpui::{Context, Entity, Window, div};

// ====================
// Editor.
// ====================
use crate::gui::inspectors::{Inspector, InspectorEvent};
use crate::gui::primitives::events::SliderEvent;
use crate::gui::primitives::increment_buttons::{IncrementButtons, IncrementEvent};
use crate::gui::primitives::slider::{SizeVariant, Slider};

pub struct SliderInspector
{
    slider:            Entity<Slider>,
    increment_buttons: Entity<IncrementButtons>,
    _subscriptions:    Vec<gpui::Subscription>,
}

impl Inspector for SliderInspector
{
    type Value = f32;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(initial)
                .with_range(0.0, 10.0)
                .with_step(0.1)
                .with_size_variant(SizeVariant::Small)
        });

        let increment_buttons = cx.new(|cx| IncrementButtons::new(cx));

        let slider_subscription = cx.subscribe(
            &slider,
            |_this, _slider, event: &SliderEvent, cx| match event {
                SliderEvent::ValueChanged(value) => {
                    cx.emit(InspectorEvent::Updated { v: *value });
                }
            },
        );

        let increment_subscription = cx.subscribe(
            &increment_buttons,
            move |this, _buttons, event: &IncrementEvent, cx| match event {
                IncrementEvent::Increment => {
                    let current_value = this.get_value(cx);
                    let step = this.slider.read(cx).step;
                    let max = this.slider.read(cx).max;
                    let new_value = (current_value + step).min(max);
                    this.update_value(new_value, cx);
                }
                IncrementEvent::Decrement => {
                    let current_value = this.get_value(cx);
                    let step = this.slider.read(cx).step;
                    let min = this.slider.read(cx).min;
                    let new_value = (current_value - step).max(min);
                    this.update_value(new_value, cx);
                }
            },
        );

        Self {
            slider,
            increment_buttons,
            _subscriptions: vec![slider_subscription, increment_subscription],
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        self.slider.read(cx).get_value()
    }
}

impl SliderInspector
{
    /// Update the slider value and emit change event
    ///
    fn update_value(&mut self, value: f32, cx: &mut Context<Self>)
    {
        self.slider.update(cx, |slider, cx| {
            slider.value = value.clamp(slider.min, slider.max);
            cx.emit(SliderEvent::ValueChanged(slider.value));
            cx.notify();
        });
        cx.emit(InspectorEvent::Updated { v: value });
    }

    pub fn with_range(cx: &mut Context<Self>, initial: f32, min: f32, max: f32) -> Self
    {
        let slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(initial)
                .with_range(min, max)
                .with_step((max - min) / 100.0)
                .with_size_variant(SizeVariant::Small)
        });

        let increment_buttons = cx.new(|cx| IncrementButtons::new(cx));

        let slider_subscription = cx.subscribe(
            &slider,
            |_this, _slider, event: &SliderEvent, cx| match event {
                SliderEvent::ValueChanged(value) => {
                    cx.emit(InspectorEvent::Updated { v: *value });
                }
            },
        );

        let increment_subscription = cx.subscribe(
            &increment_buttons,
            move |this, _buttons, event: &IncrementEvent, cx| match event {
                IncrementEvent::Increment => {
                    let current_value = this.get_value(cx);
                    let step = (max - min) / 100.0;
                    let new_value = (current_value + step).min(max);
                    this.update_value(new_value, cx);
                }
                IncrementEvent::Decrement => {
                    let current_value = this.get_value(cx);
                    let step = (max - min) / 100.0;
                    let new_value = (current_value - step).max(min);
                    this.update_value(new_value, cx);
                }
            },
        );

        Self {
            slider,
            increment_buttons,
            _subscriptions: vec![slider_subscription, increment_subscription],
        }
    }

    pub fn with_step(cx: &mut Context<Self>, initial: f32, min: f32, max: f32, step: f32) -> Self
    {
        let slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(initial)
                .with_range(min, max)
                .with_step(step)
                .with_size_variant(SizeVariant::Small)
        });

        let increment_buttons = cx.new(|cx| IncrementButtons::new(cx));

        let slider_subscription = cx.subscribe(
            &slider,
            |_this, _slider, event: &SliderEvent, cx| match event {
                SliderEvent::ValueChanged(value) => {
                    cx.emit(InspectorEvent::Updated { v: *value });
                }
            },
        );

        let increment_subscription = cx.subscribe(
            &increment_buttons,
            move |this, _buttons, event: &IncrementEvent, cx| match event {
                IncrementEvent::Increment => {
                    let current_value = this.get_value(cx);
                    let new_value = (current_value + step).min(max);
                    this.update_value(new_value, cx);
                }
                IncrementEvent::Decrement => {
                    let current_value = this.get_value(cx);
                    let new_value = (current_value - step).max(min);
                    this.update_value(new_value, cx);
                }
            },
        );

        Self {
            slider,
            increment_buttons,
            _subscriptions: vec![slider_subscription, increment_subscription],
        }
    }

    pub fn with_label(cx: &mut Context<Self>, initial: f32, label: impl Into<String>) -> Self
    {
        let slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(initial)
                .with_range(0.0, 10.0)
                .with_step(0.1)
                .with_label(label.into())
                .with_size_variant(SizeVariant::Small)
        });

        let increment_buttons = cx.new(|cx| IncrementButtons::new(cx));

        let slider_subscription = cx.subscribe(
            &slider,
            |_this, _slider, event: &SliderEvent, cx| match event {
                SliderEvent::ValueChanged(value) => {
                    cx.emit(InspectorEvent::Updated { v: *value });
                }
            },
        );

        let increment_subscription = cx.subscribe(
            &increment_buttons,
            |this, _buttons, event: &IncrementEvent, cx| match event {
                IncrementEvent::Increment => {
                    let current_value = this.get_value(cx);
                    let step = this.slider.read(cx).step;
                    let max = this.slider.read(cx).max;
                    let new_value = (current_value + step).min(max);
                    this.update_value(new_value, cx);
                }
                IncrementEvent::Decrement => {
                    let current_value = this.get_value(cx);
                    let step = this.slider.read(cx).step;
                    let min = this.slider.read(cx).min;
                    let new_value = (current_value - step).max(min);
                    this.update_value(new_value, cx);
                }
            },
        );

        Self {
            slider,
            increment_buttons,
            _subscriptions: vec![slider_subscription, increment_subscription],
        }
    }
}

impl Render for SliderInspector
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(div().flex_1().w_full().child(self.slider.clone()))
            .child(self.increment_buttons.clone())
    }
}
