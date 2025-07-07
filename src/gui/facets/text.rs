use gpui::prelude::*;
use gpui::{Context, Entity, SharedString, Window, px};

// ====================
// Editor.
// ====================
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::primitives::events::TextInputEvent;
use crate::gui::primitives::text_input::{BorderRadius, ColorVariant, SizeVariant, TextInput};
use crate::gui::styling::colors::*;

/// Text facet field for models editor.
/// Provides a text input widget for string-based properties.
///
pub struct TextFacet
{
    input:          Entity<TextInput>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for TextFacet
{
    type Value = String;

    /// Create a new text facet with initial value.
    ///
    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let initial = SharedString::new(initial.as_str());
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(initial, cx)
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
                        cx.emit(FacetEvent::Updated { v: content });
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
    ///
    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        self.input.read(cx).content.to_string()
    }
}

// ====================
// Rendering.
// ====================

impl Render for TextFacet
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        self.input.clone().into_element()
    }
}
