use std::marker::PhantomData;

use gpui::prelude::*;
use gpui::{Context, Entity, SharedString, Window};
use strum::IntoEnumIterator;

// ====================
// Editor.
// ====================
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::primitives::dropdown_input::{Dropdown, DropdownItem, DropdownSizeVariant};
use crate::gui::primitives::events::DropdownEvent;

pub struct EnumFacet<E>
where
    E: 'static + IntoEnumIterator + Clone + Default + std::fmt::Debug + std::fmt::Display,
{
    dropdown:       Entity<Dropdown>,
    _subscriptions: Vec<gpui::Subscription>,
    _p:             PhantomData<E>,
}

impl<E> Facet for EnumFacet<E>
where
    E: 'static
        + IntoEnumIterator
        + Clone
        + Default
        + std::fmt::Debug
        + std::fmt::Display
        + std::str::FromStr,
{
    type Value = E;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        // Collect all enum values
        let enum_values: Vec<DropdownItem> = E::iter()
            .map(|v| DropdownItem {
                text:   SharedString::from(v.to_string()),
                icon:   None,
                detail: None,
            })
            .collect();

        let selected_index = E::iter().position(|v| v.to_string() == initial.to_string());

        let dropdown = cx.new(|cx| {
            Dropdown::new(cx)
                .with_size_variant(DropdownSizeVariant::Small)
                .with_placeholder(SharedString::from("Select..."))
                .with_items(enum_values)
                .with_selected_index(selected_index)
        });

        let subscription = cx.subscribe(
            &dropdown,
            |_this, _dropdown, event: &DropdownEvent, cx| match event {
                DropdownEvent::SelectionChanged(index) => {
                    if let Some(enum_value) = E::iter().nth(*index) {
                        cx.emit(FacetEvent::Updated { v: enum_value });
                    }
                }
            },
        );

        Self {
            dropdown,
            _subscriptions: vec![subscription],
            _p: PhantomData::default(),
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        if let Some(val) = self.dropdown.read(cx).get_selected() {
            let val = E::from_str(&val[..]);
            return val.unwrap_or_default();
        }

        Self::Value::default()
    }
}

impl<E> EnumFacet<E>
where
    E: 'static
        + IntoEnumIterator
        + Clone
        + Default
        + std::fmt::Debug
        + std::fmt::Display
        + std::str::FromStr,
{
    pub fn set_value(&mut self, value: E, cx: &mut Context<Self>)
    {
        let selected_index = E::iter().position(|v| v.to_string() == value.to_string());

        self.dropdown.update(cx, |dropdown, cx| {
            dropdown.selected_index = selected_index;
            cx.notify();
        });
    }
}

impl<E> Render for EnumFacet<E>
where
    E: 'static
        + IntoEnumIterator
        + Clone
        + Default
        + std::fmt::Debug
        + std::str::FromStr
        + std::fmt::Display,
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        self.dropdown.clone().into_element()
    }
}
