use gpui::prelude::*;
use gpui::{Context, Entity, Window};

// ====================
// Editor.
// ====================
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::primitives::checkbox_input::{Checkbox, SizeVariant};
use crate::gui::primitives::events::CheckboxEvent;

pub struct BoolFacet
{
    checkbox:       Entity<Checkbox>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for BoolFacet
{
    type Value = bool;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let checkbox = cx.new(|cx| {
            Checkbox::new(cx)
                .with_checked(initial)
                .with_size_variant(SizeVariant::Small)
        });

        let subscription = cx.subscribe(
            &checkbox,
            |_this, _checkbox, event: &CheckboxEvent, cx| match event {
                CheckboxEvent::Changed(value) => {
                    cx.emit(FacetEvent::Updated { v: *value });
                }
            },
        );

        Self {
            checkbox,
            _subscriptions: vec![subscription],
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        self.checkbox.read(cx).is_checked()
    }
}

impl Render for BoolFacet
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        self.checkbox.clone().into_element()
    }
}
