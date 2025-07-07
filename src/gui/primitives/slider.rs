use gpui::prelude::*;
use gpui::{
    Bounds,
    Context,
    DragMoveEvent,
    Empty,
    Entity,
    EntityId,
    FocusHandle,
    Focusable,
    IntoElement,
    MouseDownEvent,
    ParentElement,
    Pixels,
    Point,
    Render,
    SharedString,
    Styled,
    Window,
    canvas,
    div,
    px,
};

// ====================
// Editor.
// ====================
use crate::gui::primitives::events::*;
use crate::gui::primitives::increment_buttons::{IncrementButtons, IncrementEvent};
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SizeVariant
{
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone)]
struct DragThumb(EntityId);

impl Render for DragThumb
{
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement
    {
        Empty
    }
}

/// Slider component for selecting f32 values within a range.
///
pub struct Slider
{
    pub focus_handle:  FocusHandle,
    pub value:         f32,
    pub min:           f32,
    pub max:           f32,
    pub step:          f32,
    pub size_variant:  SizeVariant,
    pub label:         Option<SharedString>,
    pub is_dragging:   bool,
    bounds:            Bounds<Pixels>,
    increment_buttons: Entity<IncrementButtons>,
    _subscriptions:    Vec<gpui::Subscription>,
}

// ====================
// Actions.
// ====================

pub mod actions
{
    use gpui::actions;
    actions!(slider, [Left, Right, Home, End, Escape]);
}

use actions::{End, Escape, Home, Left, Right};

impl Slider
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        let increment_buttons = cx.new(|cx| IncrementButtons::new(cx));

        let increment_subscription = cx.subscribe(
            &increment_buttons,
            |this, _buttons, event, cx| match event {
                IncrementEvent::Increment => {
                    let new_value = (this.value + this.step).clamp(this.min, this.max);
                    if new_value != this.value {
                        this.value = new_value;
                        cx.emit(SliderEvent::ValueChanged(this.value));
                        cx.notify();
                    }
                }
                IncrementEvent::Decrement => {
                    let new_value = (this.value - this.step).clamp(this.min, this.max);
                    if new_value != this.value {
                        this.value = new_value;
                        cx.emit(SliderEvent::ValueChanged(this.value));
                        cx.notify();
                    }
                }
            },
        );

        Self {
            focus_handle: cx.focus_handle(),
            value: 0.0,
            min: 0.0,
            max: 1.0,
            step: 0.01,
            size_variant: SizeVariant::default(),
            label: None,
            is_dragging: false,
            bounds: Bounds::default(),
            increment_buttons,
            _subscriptions: vec![increment_subscription],
        }
    }

    pub fn with_value(mut self, value: f32) -> Self
    {
        self.value = value.clamp(self.min, self.max);
        self
    }

    pub fn with_range(mut self, min: f32, max: f32) -> Self
    {
        self.min = min;
        self.max = max;
        self.value = self.value.clamp(min, max);
        self
    }

    pub fn with_step(mut self, step: f32) -> Self
    {
        self.step = step;
        self
    }

    pub fn with_size_variant(mut self, size_variant: SizeVariant) -> Self
    {
        self.size_variant = size_variant;
        self
    }

    pub fn with_label(mut self, label: impl Into<SharedString>) -> Self
    {
        self.label = Some(label.into());
        self
    }

    pub fn get_value(&self) -> f32
    {
        self.value
    }

    // ====================
    // Event handlers.
    // ====================

    fn on_mouse_down(&mut self, ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>)
    {
        window.focus(&mut self.focus_handle);
        self.is_dragging = true;
        self.update_value_from_mouse(ev.position, cx);
        cx.notify();
    }

    fn on_left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>)
    {
        let new_value = (self.value - self.step).clamp(self.min, self.max);
        if new_value != self.value {
            self.value = new_value;
            cx.emit(SliderEvent::ValueChanged(self.value));
            cx.notify();
        }
    }

    fn on_right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>)
    {
        let new_value = (self.value + self.step).clamp(self.min, self.max);
        if new_value != self.value {
            self.value = new_value;
            cx.emit(SliderEvent::ValueChanged(self.value));
            cx.notify();
        }
    }

    fn on_home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>)
    {
        if self.value != self.min {
            self.value = self.min;
            cx.emit(SliderEvent::ValueChanged(self.value));
            cx.notify();
        }
    }

    fn on_end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>)
    {
        if self.value != self.max {
            self.value = self.max;
            cx.emit(SliderEvent::ValueChanged(self.value));
            cx.notify();
        }
    }

    fn on_escape(&mut self, _: &Escape, window: &mut Window, cx: &mut Context<Self>)
    {
        window.blur();
        if self.is_dragging {
            self.is_dragging = false;
            cx.notify();
        }
    }

    fn update_value_from_mouse(&mut self, position: Point<Pixels>, cx: &mut Context<Self>)
    {
        let bounds = self.bounds;
        let relative_x = (position.x - bounds.origin.x).clamp(px(0.), bounds.size.width);
        let percentage = relative_x / bounds.size.width;

        let value = self.min + (self.max - self.min) * percentage;
        let value = (value / self.step).round() * self.step;
        let value = value.clamp(self.min, self.max);

        if (value - self.value).abs() > f32::EPSILON {
            self.value = value;
            cx.emit(SliderEvent::ValueChanged(self.value));
            cx.notify();
        }
    }
}

// ====================
// Rendering.
// ====================

impl Render for Slider
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_focused = self.focus_handle.is_focused(window);
        let is_dragging = self.is_dragging;

        let (height, text_size, track_height) = match self.size_variant {
            SizeVariant::Small => (px(24.), px(12.), px(4.)),
            SizeVariant::Medium => (px(32.), px(14.), px(6.)),
            SizeVariant::Large => (px(40.), px(16.), px(8.)),
        };

        let thumb_size = match self.size_variant {
            SizeVariant::Small => px(16.),
            SizeVariant::Medium => px(20.),
            SizeVariant::Large => px(24.),
        };

        // Calculate thumb position (0.0 to 1.0)
        let progress = if self.max > self.min {
            (self.value - self.min) / (self.max - self.min)
        } else {
            0.0
        };

        let entity_id = cx.entity_id();
        let thumb_color = if is_dragging {
            button_primary_hover()
        } else if is_focused {
            button_primary_hover()
        } else {
            button_primary()
        };

        let mut element = div()
            .w_full()
            .key_context("Slider")
            .track_focus(&self.focus_handle);

        if is_focused {
            element = element
                .on_action(cx.listener(Self::on_left))
                .on_action(cx.listener(Self::on_right))
                .on_action(cx.listener(Self::on_home))
                .on_action(cx.listener(Self::on_end))
                .on_action(cx.listener(Self::on_escape));
        }

        element.child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .when_some(self.label.clone(), |el, label| {
                    el.child(
                        with_default_font(div())
                            .text_size(text_size)
                            .text_color(text_primary())
                            .child(label),
                    )
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .id("slider-track-container")
                                .focusable()
                                .relative()
                                .flex_1()
                                .h(height)
                                .flex()
                                .items_center()
                                // Track container with overflow hidden
                                .child(
                                    div()
                                        .absolute()
                                        .left_0()
                                        .right_0()
                                        .top(px(height.0 / 2.0 - track_height.0 / 2.0))
                                        .h(track_height)
                                        .bg(background_darker())
                                        .rounded_full()
                                        .border_1()
                                        .border_color(border_subtle()),
                                )
                                // Progress fill
                                .child(
                                    div()
                                        .absolute()
                                        .left_0()
                                        .w(px({
                                            let width = if self.bounds.size.width.0 > 0.0 {
                                                self.bounds.size.width.0
                                            } else {
                                                200.0 // Default width
                                            };
                                            (progress * width).max(0.0).min(width)
                                        }))
                                        .top(px(height.0 / 2.0 - track_height.0 / 2.0))
                                        .h(track_height)
                                        .bg(button_primary())
                                        .rounded_full(),
                                )
                                // Thumb with drag
                                .child(
                                    div()
                                        .id("slider-thumb")
                                        .on_drag(DragThumb(entity_id), |drag, _, _, cx| {
                                            cx.stop_propagation();
                                            cx.new(|_| drag.clone())
                                        })
                                        .on_drag_move(cx.listener(
                                            move |this, e: &DragMoveEvent<DragThumb>, _, cx| {
                                                match e.drag(cx) {
                                                    DragThumb(id) => {
                                                        if *id != entity_id {
                                                            return;
                                                        }
                                                        this.is_dragging = true;
                                                        this.update_value_from_mouse(
                                                            e.event.position,
                                                            cx,
                                                        );
                                                    }
                                                }
                                            },
                                        ))
                                        .absolute()
                                        .left(px({
                                            let width = if self.bounds.size.width.0 > 0.0 {
                                                self.bounds.size.width.0
                                            } else {
                                                200.0 // Default width
                                            };
                                            (progress * width - thumb_size.0 / 2.0)
                                                .max(-(thumb_size.0 / 2.0))
                                        }))
                                        .top(px(height.0 / 2.0 - thumb_size.0 / 2.0))
                                        .w(thumb_size)
                                        .h(thumb_size)
                                        .bg(thumb_color)
                                        .border_2()
                                        .border_color(if is_focused {
                                            border_focus()
                                        } else {
                                            border_default()
                                        })
                                        .rounded_full()
                                        .hover(|el| el.bg(button_primary_hover()))
                                        .cursor_pointer(),
                                )
                                // Canvas to capture bounds
                                .child({
                                    let state_entity = cx.entity().clone();
                                    canvas(
                                        move |bounds, _, cx| {
                                            state_entity.update(cx, |slider, _| {
                                                slider.bounds = bounds;
                                            });
                                        },
                                        |_, _, _, _| {},
                                    )
                                    .absolute()
                                    .left_0()
                                    .right_0()
                                    .top_0()
                                    .bottom_0()
                                })
                                // Click handler
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(Self::on_mouse_down),
                                ),
                        )
                        .child(
                            div().min_w(px(60.)).child(
                                with_default_font(div())
                                    .text_size(text_size)
                                    .text_color(text_secondary())
                                    .child(if self.step >= 1.0 {
                                        format!("{:.0}", self.value)
                                    } else if self.step >= 0.1 {
                                        format!("{:.1}", self.value)
                                    } else {
                                        format!("{:.2}", self.value)
                                    }),
                            ),
                        )
                        .child(self.increment_buttons.clone()),
                ),
        )
    }
}

impl Focusable for Slider
{
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle
    {
        self.focus_handle.clone()
    }
}
