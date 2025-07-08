// ====================
// GPUI.
// ====================

use gpui::prelude::*;
use gpui::{
    BoxShadow,
    Context,
    Entity,
    FontWeight,
    Rgba,
    SharedString,
    Subscription,
    Window,
    div,
    px,
};

// ====================
// Editor.
// ====================
use crate::gui::inspectors::events::{KeyValueBlockEvent, ScalarBlockEvent, SequenceBlockEvent};
use crate::gui::inspectors::key_value::KeyValueInspector;
use crate::gui::inspectors::{Inspector, InspectorEvent};
use crate::gui::models::key_value::{KeyValue, KeyValueEntry};
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::*;

/// Generic scalar value block.
///
/// This component provides a labeled input field for editing
/// models properties with a consistent layout.
///
pub struct ScalarBlock<F: Inspector + Render>
{
    label:         SharedString,
    prop:          Entity<F>,
    index:         usize,
    _subscription: Subscription,
}

impl<P: Inspector> ScalarBlock<P>
{
    /// Create a new inspector block with a label and initial value.
    ///
    pub fn new(label: impl Into<SharedString>, val: P::Value, cx: &mut Context<Self>) -> Self
    where
        P::Value: Clone + std::fmt::Debug + Default,
        P: gpui::EventEmitter<InspectorEvent<P::Value>>,
    {
        let label = label.into();
        let prop = cx.new(move |cx| P::new(cx, val));

        // Subscribe to the prop's events and forward them
        let subscription = cx.subscribe(
            &prop,
            |_this, _prop, event: &InspectorEvent<P::Value>, cx| match event {
                InspectorEvent::Updated { v } => {
                    cx.emit(ScalarBlockEvent::Changed { v: v.clone() });
                    cx.emit(InspectorEvent::Updated { v: v.clone() });
                }
            },
        );

        Self {
            label,
            prop,
            index: 0,
            _subscription: subscription,
        }
    }

    pub fn with_index(mut self, index: usize) -> Self
    {
        self.index = index;
        self
    }

    // ====================
    // Value management.
    // ====================

    /// Apply the current inspector value to a target.
    ///
    pub fn apply(&self, target: &mut P::Value, cx: &Context<Self>)
    {
        *target = self.prop.read(cx).get_value(cx).clone();
    }
}

// ====================
// Rendering.
// ====================

impl<T: Inspector + Render> Render for ScalarBlock<T>
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_even = self.index % 2 == 0;
        let mut container = div().flex().items_center().gap_2().px_3().py(px(4.0));
        if self.label.is_empty() {
            container = container.pt(px(8.0));
        }

        let mut result = container
            .bg(if is_even {
                Rgba {
                    r: background_darker().r,
                    g: background_darker().g,
                    b: background_darker().b,
                    a: 0.5,
                }
            } else {
                Rgba {
                    r: background_primary().r,
                    g: background_primary().g,
                    b: background_primary().b,
                    a: 0.3,
                }
            })
            .border_1()
            .border_color(Rgba {
                r: border_separator().r,
                g: border_separator().g,
                b: border_separator().b,
                a: 0.3,
            })
            .hover(|el| {
                el.bg(hover_overlay())
                    .border_color(border_default())
                    .shadow(vec![BoxShadow {
                        color:         shadow_light(),
                        offset:        gpui::point(px(0.), px(1.)),
                        blur_radius:   px(3.),
                        spread_radius: px(0.),
                    }])
            });

        if !self.label.is_empty() {
            result = result
                .child(
                    with_default_font(div())
                        .w(px(160.0))
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_muted())
                        .pr_2()
                        .child(self.label.clone()),
                )
                .child(
                    div()
                        .w(px(1.0))
                        .h_full()
                        .bg(border_separator())
                        .opacity(0.5),
                );
        }

        result.child(
            div()
                .flex_1()
                .min_w(px(200.0))
                .w_full()
                .child(self.prop.clone()),
        )
    }
}

pub enum NewValueStrategy<P: Inspector + Render>
{
    MostRecentOr(P::Value),
    Value(P::Value),
}

/// Sequence inspector block for managing sequences of uniform properties.
///
/// This component provides a labeled editor for Vec<Inspector>
/// with add/remove functionality.
///
pub struct SequenceBlock<F: Inspector + Render>
{
    label:          SharedString,
    items:          Vec<Entity<F>>,
    new_val:        NewValueStrategy<F>,
    _subscriptions: Vec<Subscription>,
}

impl<P> SequenceBlock<P>
where
    P: Inspector,
{
    /// Create a new sequence block with a label and initial values.
    ///
    pub fn new(
        label: impl Into<SharedString>,
        initial: Vec<P::Value>,
        cx: &mut Context<Self>,
    ) -> Self
    where
        P::Value: Clone + std::fmt::Debug + Default,
        P: gpui::EventEmitter<InspectorEvent<P::Value>>,
    {
        let label = label.into();
        let items: Vec<Entity<P>> = initial
            .into_iter()
            .map(|val| cx.new(move |cx| P::new(cx, val)))
            .collect();

        // Subscribe to each item's events and forward them
        let subscriptions: Vec<Subscription> = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                cx.subscribe(
                    item,
                    move |this, _item, event: &InspectorEvent<P::Value>, cx| {
                        match event {
                            InspectorEvent::Updated { v } => {
                                cx.emit(SequenceBlockEvent::ItemChanged {
                                    index,
                                    v: v.clone(),
                                });

                                // Emit the entire sequence as a generic InspectorEvent
                                let all_values: Vec<P::Value> = this
                                    .items
                                    .iter()
                                    .map(|item| item.read(cx).get_value(cx).clone())
                                    .collect();
                                cx.emit(InspectorEvent::Updated { v: all_values });
                            }
                        }
                    },
                )
            })
            .collect();

        Self {
            label,
            items,
            new_val: NewValueStrategy::MostRecentOr(P::Value::default()),
            _subscriptions: subscriptions,
        }
    }

    pub fn with_new_val(mut self, new_val: NewValueStrategy<P>) -> Self
    {
        self.new_val = new_val;
        self
    }

    // ====================
    // Event handlers.
    // ====================

    /// Add a new item with default value.
    ///
    fn add_item(&mut self, cx: &mut Context<Self>)
    where
        P::Value: Clone + std::fmt::Debug + Default,
        P: gpui::EventEmitter<InspectorEvent<P::Value>>,
    {
        let val = match &self.new_val {
            NewValueStrategy::MostRecentOr(val) => {
                if let Some(most_recent) = self.items.last() {
                    most_recent.read(cx).get_value(cx).clone()
                } else {
                    val.clone()
                }
            }
            NewValueStrategy::Value(val) => val.clone(),
        };
        let item = cx.new(|cx| P::new(cx, val.clone()));
        let index = self.items.len();

        // Subscribe to the new item's events and forward them
        let subscription = cx.subscribe(
            &item,
            move |this, _item, event: &InspectorEvent<P::Value>, cx| {
                match event {
                    InspectorEvent::Updated { v } => {
                        cx.emit(SequenceBlockEvent::ItemChanged {
                            index,
                            v: v.clone(),
                        });

                        // Emit the entire sequence as a generic InspectorEvent
                        let all_values: Vec<P::Value> = this
                            .items
                            .iter()
                            .map(|item| item.read(cx).get_value(cx).clone())
                            .collect();
                        cx.emit(InspectorEvent::Updated { v: all_values });
                    }
                }
            },
        );

        self.items.push(item);
        self._subscriptions.push(subscription);

        // Emit events for the added item
        cx.emit(SequenceBlockEvent::ItemAdded {
            index,
            v: val.clone(),
        });

        // Emit the entire sequence as a generic InspectorEvent
        let all_values: Vec<P::Value> = self
            .items
            .iter()
            .map(|item| item.read(cx).get_value(cx).clone())
            .collect();
        cx.emit(InspectorEvent::Updated { v: all_values });

        cx.notify();
    }

    /// Remove item at index.
    ///
    fn remove_item(&mut self, index: usize, cx: &mut Context<Self>)
    where
        P::Value: Clone + std::fmt::Debug + Default,
    {
        if index < self.items.len() {
            self.items.remove(index);
            let _ = self._subscriptions.remove(index);

            // Emit events for the removed item
            cx.emit(SequenceBlockEvent::ItemRemoved { index });

            // Emit the entire sequence as a generic InspectorEvent
            let all_values: Vec<P::Value> = self
                .items
                .iter()
                .map(|item| item.read(cx).get_value(cx).clone())
                .collect();
            cx.emit(InspectorEvent::Updated { v: all_values });

            cx.notify();
        }
    }

    /// Move item up in the sequence.
    ///
    fn move_item_up(&mut self, index: usize, cx: &mut Context<Self>)
    where
        P::Value: Clone + std::fmt::Debug + Default,
    {
        if index > 0 && index < self.items.len() {
            // Swap items
            self.items.swap(index, index - 1);

            // Swap subscriptions
            self._subscriptions.swap(index, index - 1);

            // Emit events for the moved item
            cx.emit(SequenceBlockEvent::ItemMoved {
                from_index: index,
                to_index:   index - 1,
            });

            // Emit the entire sequence as a generic InspectorEvent
            let all_values: Vec<P::Value> = self
                .items
                .iter()
                .map(|item| item.read(cx).get_value(cx).clone())
                .collect();
            cx.emit(InspectorEvent::Updated { v: all_values });

            cx.notify();
        }
    }

    /// Move item down in the sequence.
    ///
    fn move_item_down(&mut self, index: usize, cx: &mut Context<Self>)
    where
        P::Value: Clone + std::fmt::Debug + Default,
    {
        if index < self.items.len() - 1 {
            // Swap items
            self.items.swap(index, index + 1);

            // Swap subscriptions
            self._subscriptions.swap(index, index + 1);

            // Emit events for the moved item
            cx.emit(SequenceBlockEvent::ItemMoved {
                from_index: index,
                to_index:   index + 1,
            });

            // Emit the entire sequence as a generic InspectorEvent
            let all_values: Vec<P::Value> = self
                .items
                .iter()
                .map(|item| item.read(cx).get_value(cx).clone())
                .collect();
            cx.emit(InspectorEvent::Updated { v: all_values });

            cx.notify();
        }
    }

    // ====================
    // Value management.
    // ====================

    /// Apply the current values to a target vector.
    ///
    pub fn apply(&self, target: &mut Vec<P::Value>, cx: &Context<Self>)
    {
        target.clear();
        for item in &self.items {
            target.push(item.read(cx).get_value(cx).clone());
        }
    }
}

// ====================
// Rendering.
// ====================

impl<T: Inspector + Render> Render for SequenceBlock<T>
where
    T::Value: Clone + std::fmt::Debug + Default,
    T: gpui::EventEmitter<InspectorEvent<T::Value>>,
{
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let mut container = div().flex().flex_col().gap_1();
        container = container.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_1()
                .child(if !self.label.is_empty() {
                    div().child(
                        with_default_font(div())
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_default())
                            .child(self.label.clone()),
                    )
                } else {
                    div()
                })
                .child(
                    div()
                        .w(px(26.0))
                        .h(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .border_1()
                        .border_color(border_subtle())
                        .bg(button_secondary())
                        .cursor_pointer()
                        .hover(|el| {
                            el.bg(button_primary())
                                .border_color(accent_blue_light())
                                .shadow(vec![
                                    gpui::BoxShadow {
                                        color:         gpui::Rgba {
                                            r: accent_blue_light().r,
                                            g: accent_blue_light().g,
                                            b: accent_blue_light().b,
                                            a: 0.4,
                                        }
                                        .into(),
                                        offset:        gpui::point(px(0.), px(0.)),
                                        blur_radius:   px(12.),
                                        spread_radius: px(2.),
                                    },
                                    gpui::BoxShadow {
                                        color:         shadow_medium(),
                                        offset:        gpui::point(px(0.), px(2.)),
                                        blur_radius:   px(6.),
                                        spread_radius: px(0.),
                                    },
                                ])
                        })
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _ev, _window, cx| {
                                this.add_item(cx);
                            }),
                        )
                        .child(ProductIcon::CirclePlus.to_svg().text_color(text_primary())),
                ),
        );

        container
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .px_2()
                    .when(self.items.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .py_3()
                                .px_3()
                                .border_1()
                                .border_color(Rgba {
                                    r: border_separator().r,
                                    g: border_separator().g,
                                    b: border_separator().b,
                                    a: 0.2,
                                })
                                .bg(Rgba {
                                    r: background_primary().r,
                                    g: background_primary().g,
                                    b: background_primary().b,
                                    a: 0.05,
                                })
                                .rounded_md()
                                .child(
                                    ProductIcon::SymbolArray.to_svg()
                                        .size_4()
                                        .text_color(text_muted())
                                        .opacity(0.4)
                                )
                                .child(
                                    with_default_font(div())
                                        .text_sm()
                                        .text_color(text_muted())
                                        .opacity(0.8)
                                        .child("No items yet — click the Add button above to add your first item")
                                )
                        )
                    })
                    .when(!self.items.is_empty(), |el| {
                        el.children(self.items.iter().enumerate().map(|(ix, item)| {
                            let is_even = ix % 2 == 0;

                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_2()
                                .py(px(4.0))
                                .bg(if is_even {
                                    Rgba {
                                        r: background_darker().r,
                                        g: background_darker().g,
                                        b: background_darker().b,
                                        a: 0.5,
                                    }
                                } else {
                                    Rgba {
                                        r: background_primary().r,
                                        g: background_primary().g,
                                        b: background_primary().b,
                                        a: 0.3,
                                    }
                                })
                                .border_1()
                                .border_color(Rgba {
                                    r: border_separator().r,
                                    g: border_separator().g,
                                    b: border_separator().b,
                                    a: 0.3,
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            div()
                                                .cursor_pointer()
                                                .p_1()
                                                .rounded_md()
                                                .bg(button_secondary())
                                                .border_1()
                                                .border_color(border_subtle())
                                                .hover(|el| {
                                                    el.bg(button_primary())
                                                        .shadow(vec![gpui::BoxShadow {
                                                            color: gpui::Rgba {
                                                                r: accent_blue_light().r,
                                                                g: accent_blue_light().g,
                                                                b: accent_blue_light().b,
                                                                a: 0.5,
                                                            }.into(),
                                                            offset: gpui::point(px(0.), px(0.)),
                                                            blur_radius: px(6.),
                                                            spread_radius: px(0.),
                                                        }])
                                                })
                                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _ev, _window, cx| {
                                                    this.move_item_up(ix, cx);
                                                }))
                                                .child(ProductIcon::ChevronUp.to_svg().size_4().text_color(text_primary()))
                                        )
                                        .child(
                                            div()
                                                .cursor_pointer()
                                                .p_1()
                                                .rounded_md()
                                                .bg(button_secondary())
                                                .border_1()
                                                .border_color(border_subtle())
                                                .hover(|el| {
                                                    el.bg(button_primary())
                                                        .shadow(vec![gpui::BoxShadow {
                                                            color: gpui::Rgba {
                                                                r: accent_blue_light().r,
                                                                g: accent_blue_light().g,
                                                                b: accent_blue_light().b,
                                                                a: 0.5,
                                                            }.into(),
                                                            offset: gpui::point(px(0.), px(0.)),
                                                            blur_radius: px(6.),
                                                            spread_radius: px(0.),
                                                        }])
                                                })
                                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _ev, _window, cx| {
                                                    this.move_item_down(ix, cx);
                                                }))
                                                .child(ProductIcon::ChevronDown.to_svg().size_4().text_color(text_primary()))
                                        )
                                        .child(
                                            div()
                                                .cursor_pointer()
                                                .p_1()
                                                .rounded_md()
                                                .bg(button_secondary())
                                                .border_1()
                                                .border_color(border_subtle())
                                                .hover(|el| {
                                                    el.bg(error())
                                                        .shadow(vec![gpui::BoxShadow {
                                                            color: gpui::Rgba {
                                                                r: error().r,
                                                                g: error().g,
                                                                b: error().b,
                                                                a: 0.5,
                                                            }.into(),
                                                            offset: gpui::point(px(0.), px(0.)),
                                                            blur_radius: px(6.),
                                                            spread_radius: px(0.),
                                                        }])
                                                })
                                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _ev, _window, cx| {
                                                    this.remove_item(ix, cx);
                                                }))
                                                .child(ProductIcon::Trash.to_svg().size_4().text_color(text_primary()))
                                        )
                                )
                                .child(
                                    div()
                                        .w(px(1.0))
                                        .h_full()
                                        .bg(border_separator())
                                        .opacity(0.5)
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(200.0))
                                        .w_full()
                                        .child(item.clone())
                                )
                        }))
                    })
            )
    }
}

/// KeyValue block for managing collections of key-value pairs.
///
/// This component provides a labeled editor for Vec<KeyValueInspector>
/// with add/remove functionality and type selection.
///
pub struct KeyValueBlock
{
    label:          SharedString,
    items:          Vec<Entity<KeyValueInspector>>,
    _subscriptions: Vec<Subscription>,
}

impl KeyValueBlock
{
    /// Create a new key-value field editor with a label and initial values.
    ///
    pub fn new(
        label: impl Into<SharedString>,
        values: Vec<KeyValueEntry>,
        cx: &mut Context<Self>,
    ) -> Self
    {
        let label = label.into();
        let items: Vec<Entity<KeyValueInspector>> = values
            .into_iter()
            .map(|val| cx.new(move |cx| KeyValueInspector::new(cx, val)))
            .collect();

        // Subscribe to each item's events and forward them
        let subscriptions: Vec<Subscription> = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                cx.subscribe(
                    item,
                    move |this, _item, event: &InspectorEvent<KeyValueEntry>, cx| {
                        match event {
                            InspectorEvent::Updated { v } => {
                                cx.emit(KeyValueBlockEvent::EntryChanged {
                                    index,
                                    entry: v.clone(),
                                });

                                // Emit the entire collection as a generic InspectorEvent
                                let all_entries: Vec<KeyValueEntry> = this
                                    .items
                                    .iter()
                                    .map(|item| item.read(cx).get_value(cx))
                                    .collect();
                                cx.emit(InspectorEvent::Updated { v: all_entries });
                            }
                        }
                    },
                )
            })
            .collect();

        Self {
            label,
            items,
            _subscriptions: subscriptions,
        }
    }

    // ====================
    // Event handlers.
    // ====================

    /// Add a new item with default value.
    ///
    fn add_item(&mut self, cx: &mut Context<Self>)
    {
        let key = format!("item_{}", self.items.len() + 1);
        let val = KeyValueEntry::new(key, KeyValue::default());
        let item = cx.new(|cx| KeyValueInspector::new(cx, val.clone()));
        let index = self.items.len();

        // Subscribe to the new item's events and forward them
        let subscription = cx.subscribe(
            &item,
            move |this, _item, event: &InspectorEvent<KeyValueEntry>, cx| {
                match event {
                    InspectorEvent::Updated { v } => {
                        cx.emit(KeyValueBlockEvent::EntryChanged {
                            index,
                            entry: v.clone(),
                        });

                        // Emit the entire collection as a generic InspectorEvent
                        let all_entries: Vec<KeyValueEntry> = this
                            .items
                            .iter()
                            .map(|item| item.read(cx).get_value(cx))
                            .collect();
                        cx.emit(InspectorEvent::Updated { v: all_entries });
                    }
                }
            },
        );

        self.items.push(item);
        self._subscriptions.push(subscription);

        // Emit events for the added item
        cx.emit(KeyValueBlockEvent::EntryAdded {
            index,
            entry: val.clone(),
        });

        // Emit the entire collection as a generic InspectorEvent
        let all_entries: Vec<KeyValueEntry> = self
            .items
            .iter()
            .map(|item| item.read(cx).get_value(cx))
            .collect();
        cx.emit(InspectorEvent::Updated { v: all_entries });

        cx.notify();
    }

    /// Remove item at index.
    ///
    fn remove_item(&mut self, index: usize, cx: &mut Context<Self>)
    {
        if index < self.items.len() {
            self.items.remove(index);
            let _ = self._subscriptions.remove(index);

            // Emit events for the removed item
            cx.emit(KeyValueBlockEvent::EntryRemoved { index });

            // Emit the entire collection as a generic InspectorEvent
            let all_entries: Vec<KeyValueEntry> = self
                .items
                .iter()
                .map(|item| item.read(cx).get_value(cx))
                .collect();
            cx.emit(InspectorEvent::Updated { v: all_entries });

            cx.notify();
        }
    }

    // ====================
    // Value management.
    // ====================

    /// Apply the current field values to a target vector.
    ///
    pub fn apply(&self, target: &mut Vec<KeyValueEntry>, cx: &Context<Self>)
    {
        target.clear();
        for item in &self.items {
            target.push(item.read(cx).get_value(cx));
        }
    }
}

// ====================
// Rendering.
// ====================

impl Render for KeyValueBlock
{
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let mut container = div().flex().flex_col().gap_1();

        container = container.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_1()
                .child(if !self.label.is_empty() {
                    div().child(
                        with_default_font(div())
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_default())
                            .child(self.label.clone()),
                    )
                } else {
                    div()
                })
                .child(
                    div()
                        .w(px(26.0))
                        .h(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .border_1()
                        .border_color(border_subtle())
                        .bg(button_secondary())
                        .cursor_pointer()
                        .hover(|el| {
                            el.bg(button_primary())
                                .border_color(accent_blue_light())
                                .shadow(vec![
                                    gpui::BoxShadow {
                                        color:         gpui::Rgba {
                                            r: accent_blue_light().r,
                                            g: accent_blue_light().g,
                                            b: accent_blue_light().b,
                                            a: 0.4,
                                        }
                                        .into(),
                                        offset:        gpui::point(px(0.), px(0.)),
                                        blur_radius:   px(12.),
                                        spread_radius: px(2.),
                                    },
                                    gpui::BoxShadow {
                                        color:         shadow_medium(),
                                        offset:        gpui::point(px(0.), px(2.)),
                                        blur_radius:   px(6.),
                                        spread_radius: px(0.),
                                    },
                                ])
                        })
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _ev, _window, cx| {
                                this.add_item(cx);
                            }),
                        )
                        .child(ProductIcon::CirclePlus.to_svg().text_color(text_primary())),
                ),
        );

        container.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .px_2()
                    .when(self.items.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .py_3()
                                .px_3()
                                .border_1()
                                .border_color(Rgba {
                                    r: border_separator().r,
                                    g: border_separator().g,
                                    b: border_separator().b,
                                    a: 0.2,
                                })
                                .bg(Rgba {
                                    r: background_primary().r,
                                    g: background_primary().g,
                                    b: background_primary().b,
                                    a: 0.05,
                                })
                                .rounded_md()
                                .child(
                                    ProductIcon::SymbolField.to_svg()
                                        .size_4()
                                        .text_color(text_muted())
                                        .opacity(0.4)
                                )
                                .child(
                                    with_default_font(div())
                                        .text_sm()
                                        .text_color(text_muted())
                                        .opacity(0.8)
                                        .child("No entries yet — click the Add button above to add your first entry")
                                )
                        )
                    })
                    .when(!self.items.is_empty(), |el| {
                        el.children(self.items.iter().enumerate().map(|(ix, item)| {
                            let is_even = ix % 2 == 0;

                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_2()
                                .py(px(4.0))
                                .bg(if is_even {
                                    Rgba {
                                        r: background_darker().r,
                                        g: background_darker().g,
                                        b: background_darker().b,
                                        a: 0.5,
                                    }
                                } else {
                                    Rgba {
                                        r: background_primary().r,
                                        g: background_primary().g,
                                        b: background_primary().b,
                                        a: 0.3,
                                    }
                                })
                                .border_1()
                                .border_color(Rgba {
                                    r: border_separator().r,
                                    g: border_separator().g,
                                    b: border_separator().b,
                                    a: 0.3,
                                })
                                .child(
                                    div()
                                        .cursor_pointer()
                                        .p_1()
                                        .rounded_md()
                                        .bg(button_secondary())
                                        .border_1()
                                        .border_color(border_subtle())
                                        .hover(|el| {
                                            el.bg(error())
                                                .shadow(vec![gpui::BoxShadow {
                                                    color: gpui::Rgba {
                                                        r: error().r,
                                                        g: error().g,
                                                        b: error().b,
                                                        a: 0.5,
                                                    }.into(),
                                                    offset: gpui::point(px(0.), px(0.)),
                                                    blur_radius: px(6.),
                                                    spread_radius: px(0.),
                                                }])
                                        })
                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _ev, _window, cx| {
                                            this.remove_item(ix, cx);
                                        }))
                                        .child(ProductIcon::Trash.to_svg().size_4().text_color(text_primary()))
                                )
                                .child(
                                    div()
                                        .w(px(1.0))
                                        .h_full()
                                        .bg(border_separator())
                                        .opacity(0.5)
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(200.0))
                                        .w_full()
                                        .child(item.clone())
                                )
                        }))
                    })
            )
    }
}
