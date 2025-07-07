use gpui::prelude::*;
use gpui::{Context, Entity, Window, div};

// ====================
// Editor.
// ====================
use crate::gui::primitives::button::Button;
use crate::gui::styling::colors::*;

pub struct IncrementButtons
{
    minus_button:   Entity<Button>,
    plus_button:    Entity<Button>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl IncrementButtons
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        let minus_button = cx.new(|_| {
            Button::new("-")
                .variant(crate::gui::primitives::button::ButtonVariant::Secondary)
                .as_button_group_left()
                .font_size("sm")
        });

        let plus_button = cx.new(|_| {
            Button::new("+")
                .variant(crate::gui::primitives::button::ButtonVariant::Secondary)
                .as_button_group_right()
                .font_size("sm")
        });

        let minus_subscription = cx.subscribe(&minus_button, |_this, _button, _event, cx| {
            cx.emit(IncrementEvent::Decrement);
        });

        let plus_subscription = cx.subscribe(&plus_button, |_this, _button, _event, cx| {
            cx.emit(IncrementEvent::Increment);
        });

        Self {
            minus_button,
            plus_button,
            _subscriptions: vec![minus_subscription, plus_subscription],
        }
    }
}

#[derive(Debug, Clone)]
pub enum IncrementEvent
{
    Increment,
    Decrement,
}

impl gpui::EventEmitter<IncrementEvent> for IncrementButtons {}

impl Render for IncrementButtons
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .items_center()
            .child(self.minus_button.clone())
            .child(
                div()
                    .border_l_1()
                    .border_color(border_subtle())
                    .child(self.plus_button.clone()),
            )
    }
}
