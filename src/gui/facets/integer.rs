use gpui::prelude::*;
use gpui::{Context, Entity, SharedString, Window, px};

// ====================
// Editor.
// ====================
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::primitives::events::TextInputEvent;
use crate::gui::primitives::text_input::{BorderRadius, ColorVariant, SizeVariant, TextInput};
use crate::gui::styling::colors::*;
use crate::gui::utils::text::ValidationMode;

/// Integer facet field for models editor.
/// Provides a text input widget for integer-based properties with numeric validation.
///
pub struct IntegerFacet
{
    input:          Entity<TextInput>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for IntegerFacet
{
    type Value = i32;

    /// Create a new integer facet with initial value.
    ///
    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let initial = SharedString::new(initial.to_string());
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(initial, cx)
                .with_validation_mode(ValidationMode::Integer)
                .with_color_variant(ColorVariant::Subtle)
                .with_border_radius(BorderRadius::Small)
                .with_border_width(px(1.0))
                .with_size_variant(SizeVariant::Small)
                .with_full_width(true)
                .with_text_color_focused(text_primary())
                .with_text_color_unfocused(text_primary())
                .with_padding(px(4.0), px(2.0))
        });

        let subscription =
            cx.subscribe(
                &input,
                |this, _input, event: &TextInputEvent, cx| match event {
                    TextInputEvent::Edited => {
                        let content = this.input.read(cx).content.to_string();
                        let value = content.parse::<i32>().unwrap_or(0);
                        cx.emit(FacetEvent::Updated { v: value });
                    }
                    _ => {}
                },
            );

        Self {
            input,
            _subscriptions: vec![subscription],
        }
    }

    /// Get current value from the text input.
    /// Returns 0 if the input cannot be parsed as an integer.
    ///
    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        let content = self.input.read(cx).content.to_string();
        content.parse::<i32>().unwrap_or(0)
    }
}

// ====================
// Rendering.
// ====================

impl Render for IntegerFacet
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        self.input.clone().into_element()
    }
}
