use gpui::prelude::*;
use gpui::{AnyView, Context, Entity, SharedString, Window, div, px};

// ====================
// Editor.
// ====================
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::models::color::HdrColor;
use crate::gui::models::key_value::{KeyValue, KeyValueEntry, ValueType};
use crate::gui::primitives::color_picker_input::ColorPicker;
use crate::gui::primitives::dropdown_input::{Dropdown, DropdownSizeVariant};
use crate::gui::primitives::events::{
    ColorPickerEvent,
    DropdownEvent,
    TextInputEvent,
    Vec2InputEvent,
    Vec3InputEvent,
};
use crate::gui::primitives::text_input::{BorderRadius, ColorVariant, SizeVariant, TextInput};
use crate::gui::primitives::vec2_input::Vec2Input;
use crate::gui::primitives::vec3_input::Vec3Input;
use crate::gui::styling::colors::*;
use crate::gui::utils::text::ValidationMode;

pub struct KeyValueFacet
{
    key_input: Entity<TextInput>,

    // Possible values.
    type_dropdown:  Entity<Dropdown>,
    value_type:     ValueType,
    float_input:    Entity<TextInput>,
    integer_input:  Entity<TextInput>,
    vec2_input:     Entity<Vec2Input>,
    vec3_input:     Entity<Vec3Input>,
    color_input:    Entity<ColorPicker>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for KeyValueFacet
{
    type Value = KeyValueEntry;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let key_input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(initial.key.clone(), cx)
                .with_color_variant(ColorVariant::Subtle)
                .with_border_radius(BorderRadius::Small)
                .with_border_width(px(1.0))
                .with_size_variant(SizeVariant::Small)
                .with_full_width(true)
                .with_text_color_focused(text_primary())
                .with_text_color_unfocused(text_primary())
                .with_padding(px(4.0), px(2.0))
        });

        let type_dropdown = cx.new(|cx| {
            Dropdown::new(cx)
                .with_items(ValueType::variants())
                .with_selected_index(Some(initial.value.get_type().to_index()))
                .with_size_variant(DropdownSizeVariant::Small)
        });

        let float_input = cx.new(|cx| {
            let value = match &initial.value {
                KeyValue::Float(v) => *v,
                _ => 0.0,
            };
            TextInput::new(cx)
                .with_content(SharedString::new(value.to_string()), cx)
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

        let integer_input = cx.new(|cx| {
            let value = match &initial.value {
                KeyValue::Integer(v) => *v,
                _ => 0,
            };
            TextInput::new(cx)
                .with_content(SharedString::new(value.to_string()), cx)
                .with_validation_mode(ValidationMode::Numeric)
                .with_color_variant(ColorVariant::Subtle)
                .with_border_radius(BorderRadius::Small)
                .with_border_width(px(1.0))
                .with_size_variant(SizeVariant::Small)
                .with_full_width(true)
                .with_text_color_focused(text_primary())
                .with_text_color_unfocused(text_primary())
                .with_padding(px(4.0), px(2.0))
        });

        let vec2_input = cx.new(|cx| {
            let value = match &initial.value {
                KeyValue::Vec2(v) => *v,
                _ => bevy::math::Vec2::ZERO,
            };
            Vec2Input::with_value(cx, value)
        });

        let vec3_input = cx.new(|cx| {
            let value = match &initial.value {
                KeyValue::Vec3(v) => *v,
                _ => bevy::math::Vec3::ZERO,
            };
            Vec3Input::with_value(cx, value)
        });

        let color_input = cx.new(|cx| {
            let value = match &initial.value {
                KeyValue::Color(v) => *v,
                _ => HdrColor::default(),
            };
            ColorPicker::new(cx).with_color(value, cx)
        });

        // Subscribe to all relevant events
        let mut subscriptions = Vec::new();

        // Subscribe to type dropdown changes
        let type_subscription = cx.subscribe(
            &type_dropdown,
            |this, _dropdown, event: &DropdownEvent, cx| match event {
                DropdownEvent::SelectionChanged(index) => {
                    this.on_type_changed(*index, cx);
                    cx.emit(FacetEvent::Updated {
                        v: this.get_value(cx),
                    });
                }
            },
        );
        subscriptions.push(type_subscription);

        // Subscribe to key input changes
        let key_subscription = cx.subscribe(
            &key_input,
            |this, _input, event: &TextInputEvent, cx| match event {
                TextInputEvent::Edited => {
                    cx.emit(FacetEvent::Updated {
                        v: this.get_value(cx),
                    });
                }
                _ => {}
            },
        );
        subscriptions.push(key_subscription);

        // Subscribe to float input changes
        let float_subscription = cx.subscribe(
            &float_input,
            |this, _input, event: &TextInputEvent, cx| match event {
                TextInputEvent::Edited => {
                    if this.value_type == ValueType::Float {
                        cx.emit(FacetEvent::Updated {
                            v: this.get_value(cx),
                        });
                    }
                }
                _ => {}
            },
        );
        subscriptions.push(float_subscription);

        // Subscribe to integer input changes
        let integer_subscription = cx.subscribe(
            &integer_input,
            |this, _input, event: &TextInputEvent, cx| match event {
                TextInputEvent::Edited => {
                    if this.value_type == ValueType::Integer {
                        cx.emit(FacetEvent::Updated {
                            v: this.get_value(cx),
                        });
                    }
                }
                _ => {}
            },
        );
        subscriptions.push(integer_subscription);

        // Subscribe to vec2 input changes
        let vec2_subscription = cx.subscribe(
            &vec2_input,
            |this, _input, event: &Vec2InputEvent, cx| match event {
                Vec2InputEvent::Changed(_) => {
                    if this.value_type == ValueType::Vec2 {
                        cx.emit(FacetEvent::Updated {
                            v: this.get_value(cx),
                        });
                    }
                }
            },
        );
        subscriptions.push(vec2_subscription);

        // Subscribe to vec3 input changes
        let vec3_subscription = cx.subscribe(
            &vec3_input,
            |this, _input, event: &Vec3InputEvent, cx| match event {
                Vec3InputEvent::Changed(_) => {
                    if this.value_type == ValueType::Vec3 {
                        cx.emit(FacetEvent::Updated {
                            v: this.get_value(cx),
                        });
                    }
                }
            },
        );
        subscriptions.push(vec3_subscription);

        // Subscribe to color input changes
        let color_subscription = cx.subscribe(
            &color_input,
            |this, _input, event: &ColorPickerEvent, cx| match event {
                ColorPickerEvent::ColorChanged(_) | ColorPickerEvent::ValuesChanged { .. } => {
                    if this.value_type == ValueType::Color {
                        cx.emit(FacetEvent::Updated {
                            v: this.get_value(cx),
                        });
                    }
                }
                _ => {}
            },
        );
        subscriptions.push(color_subscription);

        Self {
            key_input,
            type_dropdown,
            value_type: initial.value.get_type(),
            float_input,
            integer_input,
            vec2_input,
            vec3_input,
            color_input,
            _subscriptions: subscriptions,
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        let key = self.key_input.read(cx).content.clone();

        let value = match self.value_type {
            ValueType::Float => {
                let content = self.float_input.read(cx).content.to_string();
                let val = content.parse::<f32>().unwrap_or(0.0);
                KeyValue::Float(val)
            }
            ValueType::Integer => {
                let content = self.integer_input.read(cx).content.to_string();
                let val = content.parse::<i32>().unwrap_or(0);
                KeyValue::Integer(val)
            }
            ValueType::Vec2 => {
                let val = self.vec2_input.read(cx).get_value(cx);
                KeyValue::Vec2(val)
            }
            ValueType::Vec3 => {
                let val = self.vec3_input.read(cx).get_value(cx);
                KeyValue::Vec3(val)
            }
            ValueType::Color => {
                let val = self.color_input.read(cx).get_hdr_color();
                KeyValue::Color(val)
            }
        };

        KeyValueEntry { key, value }
    }
}

impl KeyValueFacet
{
    fn on_type_changed(&mut self, type_index: usize, cx: &mut Context<Self>)
    {
        let new_type = ValueType::from_index(type_index);
        if new_type != self.value_type {
            self.value_type = new_type;
            cx.notify();
        }
    }
}

impl Render for KeyValueFacet
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        let elem: AnyView = match self.value_type {
            ValueType::Float => AnyView::from(self.float_input.clone().into_element()),
            ValueType::Integer => AnyView::from(self.integer_input.clone().into_element()),
            ValueType::Vec2 => AnyView::from(self.vec2_input.clone().into_element()),
            ValueType::Vec3 => AnyView::from(self.vec3_input.clone().into_element()),
            ValueType::Color => AnyView::from(self.color_input.clone().into_element()),
        };

        div()
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .child(div().w(px(120.0)).child(self.key_input.clone()))
            .child(
                div()
                    .w(px(1.0))
                    .h(px(20.0))
                    .bg(border_separator())
                    .opacity(0.5),
            )
            .child(div().w(px(120.0)).child(self.type_dropdown.clone()))
            .child(
                div()
                    .w(px(1.0))
                    .h(px(20.0))
                    .bg(border_separator())
                    .opacity(0.5),
            )
            .child(div().flex_1().min_w(px(120.0)).child(elem))
    }
}
