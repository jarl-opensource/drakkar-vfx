use gpui::prelude::*;
use gpui::{Context, Entity, SharedString, Window, div, px};

// ====================
// Editor.
// ====================
use crate::gui::inspectors::{Inspector, InspectorEvent};
use crate::gui::primitives::events::TextInputEvent;
use crate::gui::primitives::increment_buttons::{IncrementButtons, IncrementEvent};
use crate::gui::primitives::text_input::{BorderRadius, ColorVariant, SizeVariant, TextInput};
use crate::gui::styling::colors::*;
use crate::gui::utils::text::ValidationMode;

/// Float inspector field for models editor.
/// Provides a text input widget for floating-point properties with numeric validation and increment buttons.
///
pub struct FloatInspector
{
    input:             Entity<TextInput>,
    increment_buttons: Entity<IncrementButtons>,
    _subscriptions:    Vec<gpui::Subscription>,
}

impl Inspector for FloatInspector
{
    type Value = f32;

    /// Create a new float inspector with initial value.
    ///
    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let initial = SharedString::new(Self::format_float(initial));
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(initial, cx)
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

        let increment_buttons = cx.new(|cx| IncrementButtons::new(cx));

        let input_subscription = cx.subscribe(
            &input,
            |this, _input, event: &TextInputEvent, cx| match event {
                TextInputEvent::Edited => {
                    let content = this.input.read(cx).content.to_string();
                    let value = Self::parse_float(&content);
                    cx.emit(InspectorEvent::Updated { v: value });
                }
                _ => {}
            },
        );

        let increment_subscription = cx.subscribe(
            &increment_buttons,
            |this, _buttons, event: &IncrementEvent, cx| match event {
                IncrementEvent::Increment => {
                    let current_value = this.get_value(cx);
                    let new_value = current_value + 0.1; // Default step
                    this.update_value(new_value, cx);
                }
                IncrementEvent::Decrement => {
                    let current_value = this.get_value(cx);
                    let new_value = (current_value - 0.1).max(0.0); // Default step, clamp to 0
                    this.update_value(new_value, cx);
                }
            },
        );

        Self {
            input,
            increment_buttons,
            _subscriptions: vec![input_subscription, increment_subscription],
        }
    }

    /// Get current value from the text input.
    /// Returns 0.0 if the input cannot be parsed as a float.
    ///
    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        let content = self.input.read(cx).content.to_string();
        Self::parse_float(&content)
    }
}

impl FloatInspector
{
    /// Format a f32 value to string, handling special values consistently.
    ///
    fn format_float(value: f32) -> String
    {
        if value.is_infinite() {
            if value.is_sign_positive() {
                f32::MAX.to_string()
            } else {
                f32::MIN.to_string()
            }
        } else if value.is_nan() {
            return "1.0".to_string();
        } else {
            value.to_string()
        }
    }

    /// Parse a string to f32, handling special values like "inf", "-inf", and "nan" case-insensitively.
    /// Returns 0.0 if the input cannot be parsed as a float.
    ///
    fn parse_float(content: &str) -> f32
    {
        let trimmed = content.trim();
        match trimmed.to_lowercase().as_str() {
            "inf" | "infinity" | "+inf" | "+infinity" => f32::INFINITY,
            "-inf" | "-infinity" => f32::NEG_INFINITY,
            "nan" => f32::NAN,
            _ => trimmed.parse::<f32>().unwrap_or(0.0),
        }
    }

    /// Update the input value and emit change event
    ///
    fn update_value(&mut self, value: f32, cx: &mut Context<Self>)
    {
        let formatted = Self::format_float(value);
        self.input.update(cx, |input, cx| {
            input.content = SharedString::new(formatted);
            cx.notify();
        });
        cx.emit(InspectorEvent::Updated { v: value });
    }
}

// ====================
// Rendering.
// ====================

impl Render for FloatInspector
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(div().flex_1().child(self.input.clone()))
            .child(self.increment_buttons.clone())
    }
}
