use gpui::prelude::*;
use gpui::{
    App,
    Context,
    FocusHandle,
    Focusable,
    MouseButton,
    ParentElement,
    Pixels,
    SharedString,
    Styled,
    Window,
    deferred,
    div,
    px,
};

// ====================
// Editor.
// ====================
use crate::gui::primitives::events::DropdownMenuEvent;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::ProductIcon;

#[derive(Clone, Debug)]
pub struct MenuItem
{
    pub text:   SharedString,
    pub detail: Option<SharedString>,
    pub icon:   Option<ProductIcon>,
}

impl MenuItem
{
    pub fn new(text: impl Into<SharedString>) -> Self
    {
        Self {
            text:   text.into(),
            detail: None,
            icon:   None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<SharedString>) -> Self
    {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_icon(mut self, icon: ProductIcon) -> Self
    {
        self.icon = Some(icon);
        self
    }
}

pub mod actions
{
    use gpui::actions;
    actions!(dropdown_menu, [Up, Down, Enter, Escape]);
}

use actions::{Down, Enter, Escape, Up};

pub struct DropdownMenu
{
    pub focus_handle:   FocusHandle,
    pub items:          Vec<MenuItem>,
    pub selected_index: Option<usize>,
    pub max_height:     Pixels,
    pub item_height:    Pixels,
}

impl DropdownMenu
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        Self {
            focus_handle:   cx.focus_handle(),
            items:          Vec::new(),
            selected_index: None,
            max_height:     px(200.),
            item_height:    px(28.),
        }
    }

    pub fn with_items(mut self, items: Vec<MenuItem>) -> Self
    {
        self.items = items;
        if !self.items.is_empty() && self.selected_index.is_none() {
            self.selected_index = Some(0);
        }
        self
    }

    pub fn with_max_height(mut self, height: Pixels) -> Self
    {
        self.max_height = height;
        self
    }

    pub fn with_selected_index(mut self, index: Option<usize>) -> Self
    {
        self.selected_index = index;
        self
    }

    pub fn select_next(&mut self, cx: &mut Context<Self>)
    {
        if self.items.is_empty() {
            return;
        }

        let current = self.selected_index.unwrap_or(self.items.len() - 1);
        self.selected_index = Some((current + 1) % self.items.len());
        cx.notify();
    }

    pub fn select_previous(&mut self, cx: &mut Context<Self>)
    {
        if self.items.is_empty() {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        self.selected_index = Some(if current > 0 {
            current - 1
        } else {
            self.items.len() - 1
        });
        cx.notify();
    }

    fn on_up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>)
    {
        self.select_previous(cx);
    }

    fn on_down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>)
    {
        self.select_next(cx);
    }

    fn on_enter(&mut self, _: &Enter, _: &mut Window, cx: &mut Context<Self>)
    {
        if let Some(index) = self.selected_index {
            cx.emit(DropdownMenuEvent::ItemSelected(index));
        }
    }

    fn on_escape(&mut self, _: &Escape, _: &mut Window, cx: &mut Context<Self>)
    {
        cx.emit(DropdownMenuEvent::Cancelled);
    }

    fn on_item_click(&mut self, index: usize, cx: &mut Context<Self>)
    {
        self.selected_index = Some(index);
        cx.emit(DropdownMenuEvent::ItemSelected(index));
        cx.notify();
    }
}

impl Render for DropdownMenu
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_focused = self.focus_handle.is_focused(window);

        deferred(
            div()
                .occlude()
                .id("dropdown-menu")
                .key_context("DropdownMenu")
                .track_focus(&self.focus_handle)
                .when(is_focused, |el| {
                    el.on_action(cx.listener(Self::on_up))
                        .on_action(cx.listener(Self::on_down))
                        .on_action(cx.listener(Self::on_enter))
                        .on_action(cx.listener(Self::on_escape))
                })
                .p_1()
                .bg(background_darker())
                .border_1()
                .border_color(border_subtle())
                .rounded_md()
                .shadow_lg()
                .max_h(self.max_height)
                .overflow_y_scroll()
                .child(div().w_full().children(self.items.iter().enumerate().map(
                    |(index, item)| {
                        let is_selected = self.selected_index == Some(index);

                        div()
                            .w_full()
                            .h(self.item_height)
                            .px_3()
                            .rounded_md()
                            .mb_0p5()
                            .flex()
                            .items_center()
                            .bg(if is_selected {
                                dropdown_selected()
                            } else {
                                transparent_black()
                            })
                            .cursor_pointer()
                            .hover(|el| el.bg(dropdown_hover()))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.on_item_click(index, cx);
                                }),
                            )
                            .child(
                                // Icon section - fixed width for alignment
                                div()
                                    .w_6()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when_some(item.icon.clone(), |el, icon| {
                                        el.child(
                                            icon.to_svg().size_4().text_color(text_secondary()),
                                        )
                                    }),
                            )
                            .child(
                                // Text section - flexible width
                                div().flex_1().child(
                                    with_default_font(div())
                                        .text_sm()
                                        .text_color(text_primary())
                                        .truncate()
                                        .child(item.text.clone()),
                                ),
                            )
                            .when_some(item.detail.as_ref(), |el, detail| {
                                el.child(
                                    // Detail section - fixed width for alignment
                                    div().w_20().flex().justify_end().child(
                                        with_default_font(div())
                                            .text_xs()
                                            .text_color(autocomplete_type())
                                            .child(detail.clone()),
                                    ),
                                )
                            })
                    },
                ))),
        )
        .with_priority(100)
    }
}

impl Focusable for DropdownMenu
{
    fn focus_handle(&self, _: &App) -> FocusHandle
    {
        self.focus_handle.clone()
    }
}
