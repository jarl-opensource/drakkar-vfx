use gpui::prelude::*;
use gpui::{AnyView, Context, Entity, EventEmitter, Window, div, px};

// ====================
// Editor.
// ====================
use crate::gui::inspectors::enumeration::EnumInspector;
use crate::gui::inspectors::{Inspector, InspectorEvent};
use crate::gui::models::modifier::{XOrientMode, XOrientModifier, XRenderModifier};
use crate::gui::primitives::dropdown_input::{Dropdown, DropdownItem, DropdownSizeVariant};
use crate::gui::primitives::events::DropdownEvent;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;

pub struct RenderModifierInspector
{
    type_dropdown:    Entity<Dropdown>,
    current_modifier: XRenderModifier,
    orient_mode_enum: Entity<EnumInspector<XOrientMode>>,
    _subscriptions:   Vec<gpui::Subscription>,
}

impl Inspector for RenderModifierInspector
{
    type Value = XRenderModifier;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let type_dropdown = cx.new(|cx| {
            let items = vec![DropdownItem {
                text:   "Orient".to_string().into(),
                icon:   None,
                detail: None,
            }];

            let selected_index = match &initial {
                XRenderModifier::XOrient(_) => Some(0),
            };

            Dropdown::new(cx)
                .with_items(items)
                .with_selected_index(selected_index)
                .with_size_variant(DropdownSizeVariant::Small)
        });

        // Initialize enum fields with values from the initial modifier
        let orient_mode_init = match &initial {
            XRenderModifier::XOrient(m) => m.mode.clone(),
        };

        let orient_mode_enum = cx.new(|cx| EnumInspector::new(cx, orient_mode_init));

        let dropdown_subscription = cx.subscribe(
            &type_dropdown,
            |this, _dropdown, event: &DropdownEvent, cx| {
                let DropdownEvent::SelectionChanged(index) = event;
                this.on_type_changed(*index, cx);
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        );

        let mut subscriptions = vec![dropdown_subscription];

        // Subscribe to enum field changes
        subscriptions.push(cx.subscribe(
            &orient_mode_enum,
            |this, _entity, _event: &InspectorEvent<XOrientMode>, cx| {
                // Update the current modifier when the enum field changes
                let mode = this.orient_mode_enum.read(cx).get_value(cx);
                this.current_modifier = XRenderModifier::XOrient(XOrientModifier { mode });
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        let instance = Self {
            type_dropdown,
            current_modifier: initial,
            orient_mode_enum,
            _subscriptions: subscriptions,
        };

        instance
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        // Construct the current value from the enum field
        match &self.current_modifier {
            XRenderModifier::XOrient(_) => {
                let mode = self.orient_mode_enum.read(cx).get_value(cx);
                XRenderModifier::XOrient(XOrientModifier { mode })
            }
        }
    }
}

impl EventEmitter<InspectorEvent<XRenderModifier>> for RenderModifierInspector {}

impl RenderModifierInspector
{
    fn on_type_changed(&mut self, type_index: usize, cx: &mut Context<Self>)
    {
        match type_index {
            0 => {
                // Orient modifier
                let mode = self.orient_mode_enum.read(cx).get_value(cx);
                self.current_modifier = XRenderModifier::XOrient(XOrientModifier { mode });
            }
            _ => {
                // Default to Orient
                let mode = self.orient_mode_enum.read(cx).get_value(cx);
                self.current_modifier = XRenderModifier::XOrient(XOrientModifier { mode });
            }
        }
    }

    fn render_field_row(&self, label: &str, content: AnyView) -> impl IntoElement
    {
        div()
            .flex()
            .items_center()
            .gap_2()
            .mb_1()
            .child(
                div().w(px(140.0)).flex().justify_end().pr_2().child(
                    with_default_font(div())
                        .text_xs()
                        .text_color(text_muted())
                        .child(format!("{}:", label)),
                ),
            )
            .child(div().flex_1().child(content))
    }
}

impl Render for RenderModifierInspector
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .mb_2()
                    .child(
                        div().w_20().flex().justify_end().pr_2().child(
                            with_default_font(div())
                                .text_xs()
                                .text_color(text_muted())
                                .child("Type:"),
                        ),
                    )
                    .child(div().flex_1().child(self.type_dropdown.clone())),
            )
            .child(match &self.current_modifier {
                XRenderModifier::XOrient(_) => div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.render_field_row("Mode", self.orient_mode_enum.clone().into())),
            })
    }
}
