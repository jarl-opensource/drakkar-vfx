use gpui::prelude::*;
use gpui::{Context, Entity, Window};

// ====================
// Editor.
// ====================
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::models::color::HdrColor;
use crate::gui::primitives::color_picker_input::{ColorPicker, SizeVariant};
use crate::gui::primitives::events::ColorPickerEvent;

pub struct ColorFacet
{
    color_picker:   Entity<ColorPicker>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for ColorFacet
{
    type Value = HdrColor;

    /// Create a new color facet with initial value.
    ///
    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let color_picker = cx.new(|cx| {
            ColorPicker::new(cx)
                .with_color(initial, cx)
                .with_size_variant(SizeVariant::Small)
        });

        let subscription = cx.subscribe(
            &color_picker,
            |_this, _color_picker, event: &ColorPickerEvent, cx| match event {
                ColorPickerEvent::ColorChanged(color) => {
                    let hdr_color = HdrColor::from_gpui_default(*color);
                    cx.emit(FacetEvent::Updated { v: hdr_color });
                }
                ColorPickerEvent::ValuesChanged {
                    color, intensity, ..
                } => {
                    let hdr_color = HdrColor::from_gpui(*color, *intensity);
                    cx.emit(FacetEvent::Updated { v: hdr_color });
                }
                _ => {}
            },
        );

        Self {
            color_picker,
            _subscriptions: vec![subscription],
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        self.color_picker.read(cx).get_hdr_color()
    }
}

// ====================
// Rendering.
// ====================

impl Render for ColorFacet
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        self.color_picker.clone().into_element()
    }
}
