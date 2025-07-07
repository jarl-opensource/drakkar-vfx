// ====================
// GPUI.
// ====================

use gpui::prelude::*;
use gpui::{
    Context,
    FocusHandle,
    Focusable,
    IntoElement,
    MouseDownEvent,
    ParentElement,
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
use crate::gui::primitives::events::*;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::ProductIcon;

#[derive(Clone, Debug)]
pub struct DropdownItem
{
    pub text:   SharedString,
    pub icon:   Option<ProductIcon>,
    pub detail: Option<SharedString>,
}

impl DropdownItem
{
    pub fn new(text: impl Into<SharedString>) -> Self
    {
        Self {
            text:   text.into(),
            icon:   None,
            detail: None,
        }
    }

    pub fn with_icon(mut self, icon: ProductIcon) -> Self
    {
        self.icon = Some(icon);
        self
    }

    pub fn with_detail(mut self, detail: impl Into<SharedString>) -> Self
    {
        self.detail = Some(detail.into());
        self
    }
}

// ====================
// Actions.
// ====================

pub mod actions
{
    use gpui::actions;
    actions!(dropdown, [Up, Down, Enter, Escape]);
}

use actions::{Down, Enter, Escape, Up};

/// Dropdown component for selecting from a list of options.
///
pub struct Dropdown
{
    pub focus_handle:   FocusHandle,
    pub selected_index: Option<usize>,
    pub items:          Vec<DropdownItem>,
    pub is_open:        bool,
    pub size_variant:   DropdownSizeVariant,
    pub placeholder:    SharedString,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DropdownSizeVariant
{
    Small,
    #[default]
    Medium,
    Large,
}

impl Dropdown
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        Self {
            focus_handle:   cx.focus_handle(),
            selected_index: None,
            items:          Vec::new(),
            is_open:        false,
            size_variant:   DropdownSizeVariant::default(),
            placeholder:    SharedString::from("Select..."),
        }
    }

    pub fn with_items(mut self, items: Vec<DropdownItem>) -> Self
    {
        self.items = items;
        self
    }

    pub fn with_text_items(mut self, items: Vec<SharedString>) -> Self
    {
        self.items = items.into_iter().map(DropdownItem::new).collect();
        self
    }

    pub fn with_selected_index(mut self, index: Option<usize>) -> Self
    {
        self.selected_index = index;
        self
    }

    pub fn with_size_variant(mut self, size_variant: DropdownSizeVariant) -> Self
    {
        self.size_variant = size_variant;
        self
    }

    pub fn with_placeholder(mut self, placeholder: SharedString) -> Self
    {
        self.placeholder = placeholder;
        self
    }

    pub fn get_selected(&self) -> Option<&SharedString>
    {
        self.selected_index
            .and_then(|ix| self.items.get(ix).map(|item| &item.text))
    }

    pub fn get_selected_item(&self) -> Option<&DropdownItem>
    {
        self.selected_index.and_then(|ix| self.items.get(ix))
    }

    // ====================
    // Event handlers.
    // ====================

    /// Handle mouse click on dropdown toggle.
    ///
    pub fn on_click(&mut self, _ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>)
    {
        if self.is_open {
            window.blur();
            self.is_open = false;
        } else {
            window.focus(&mut self.focus_handle);
            self.is_open = true;
        }
        cx.notify();
    }

    /// Handle keyboard Up event.
    ///
    fn on_up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>)
    {
        if !self.is_open || self.items.is_empty() {
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

    /// Handle keyboard Down event.
    ///
    fn on_down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>)
    {
        if !self.is_open || self.items.is_empty() {
            return;
        }

        let current = self.selected_index.unwrap_or(self.items.len() - 1);
        self.selected_index = Some((current + 1) % self.items.len());
        cx.notify();
    }

    /// Handle keyboard Enter event.
    ///
    fn on_enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>)
    {
        if self.is_open {
            if let Some(index) = self.selected_index {
                self.on_select_item(index, window, cx);
            } else {
                self.is_open = false;
                window.blur();
                cx.notify();
            }
        } else {
            window.focus(&mut self.focus_handle);
            self.is_open = true;
            cx.notify();
        }
    }

    /// Handle keyboard Escape event.
    ///
    fn on_escape(&mut self, _: &Escape, window: &mut Window, cx: &mut Context<Self>)
    {
        if self.is_open {
            self.is_open = false;
            window.blur();
            cx.notify();
        }
    }

    /// Select an item by index.
    ///
    fn on_select_item(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>)
    {
        self.selected_index = Some(index);
        self.is_open = false;

        cx.emit(DropdownEvent::SelectionChanged(index));
        cx.notify();
        cx.stop_propagation();
    }

    /// Handle mouse down outside dropdown.
    ///
    fn on_mouse_down_out(
        &mut self,
        _ev: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    )
    {
        if self.is_open {
            self.is_open = false;
            window.blur();
            cx.notify();
        }
    }
}

// ====================
// Rendering.
// ====================

impl Render for Dropdown
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_open = self.is_open;
        let is_focused = self.focus_handle.is_focused(window);

        let (height, text_size, icon_size, padding) = match self.size_variant {
            DropdownSizeVariant::Small => (px(32.), px(13.), px(12.), px(12.)),
            DropdownSizeVariant::Medium => (px(40.), px(14.), px(14.), px(16.)),
            DropdownSizeVariant::Large => (px(48.), px(15.), px(16.), px(20.)),
        };

        let display_text = self
            .get_selected()
            .cloned()
            .unwrap_or_else(|| self.placeholder.clone());

        let bg_color = background_darker();

        let border_color = if is_focused && is_open {
            border_focus()
        } else if is_open {
            border_focus()
        } else if is_focused {
            border_active()
        } else {
            border_default()
        };

        let text_color = if self.selected_index.is_some() {
            text_primary()
        } else {
            text_secondary()
        };

        let mut element = div()
            .relative()
            .w_full()
            .key_context("Dropdown")
            .track_focus(&self.focus_handle);

        if is_focused || is_open {
            element = element
                .on_action(cx.listener(Self::on_up))
                .on_action(cx.listener(Self::on_down))
                .on_action(cx.listener(Self::on_enter))
                .on_action(cx.listener(Self::on_escape));
        }

        element
            .when(is_open, |el| {
                el.on_mouse_down_out(cx.listener(Self::on_mouse_down_out))
            })
            .child(
                div()
                    .id("dropdown-input")
                    .focusable()
                    .h(height)
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(padding)
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|el| el.bg(surface_elevated()).border_color(border_active()))
                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_click))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .when_some(
                                self.get_selected_item().and_then(|item| item.icon.clone()),
                                |el, icon| {
                                    el.child(div().flex().items_center().justify_center().child(
                                        icon.to_svg().size(icon_size).text_color(icon_default()),
                                    ))
                                },
                            )
                            .child(
                                with_default_font(div())
                                    .flex()
                                    .mt(px(0.5))
                                    .items_center()
                                    .text_size(text_size)
                                    .text_color(text_color)
                                    .truncate()
                                    .child(display_text),
                            ),
                    )
                    .child(
                        div().flex().items_center().justify_center().child(
                            if is_open {
                                ProductIcon::ChevronUp.to_svg().size_10()
                            } else {
                                ProductIcon::ChevronDown.to_svg().size_10()
                            }
                            .size(icon_size)
                            .text_color(text_secondary()),
                        ),
                    ),
            )
            .when(is_open, |el| {
                el.child(
                    deferred(
                        div()
                            .occlude()
                            .id("dropdown-menu")
                            .p_1()
                            .absolute()
                            .top(height)
                            .left_0()
                            .right_0()
                            .mt_1()
                            .bg(background_darker())
                            .border_1()
                            .border_color(border_subtle())
                            .rounded_md()
                            .shadow_lg()
                            .max_h(px(300.))
                            .overflow_y_scroll()
                            .child(div().w_full().py_1().children(
                                self.items.iter().enumerate().map(|(ix, item)| {
                                    let is_selected = self.selected_index == Some(ix);

                                    div()
                                        .w_full()
                                        .px(padding)
                                        .py(px(4.))
                                        .rounded_md()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .mb_1()
                                        .bg(if is_selected {
                                            dropdown_selected()
                                        } else {
                                            background_darker()
                                        })
                                        .cursor_pointer()
                                        .hover(|el| el.bg(dropdown_hover()))
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(move |this, _ev, window, cx| {
                                                this.on_select_item(ix, window, cx);
                                            }),
                                        )
                                        .when_some(item.icon.clone(), |el, icon| {
                                            el.child(
                                                icon.to_svg()
                                                    .size(icon_size)
                                                    .text_color(text_secondary()),
                                            )
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .flex_1()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    with_default_font(div())
                                                        .justify_center()
                                                        .flex()
                                                        .mt(px(0.5))
                                                        .items_center()
                                                        .text_size(text_size)
                                                        .text_color(text_primary())
                                                        .child(item.text.clone()),
                                                )
                                                .when_some(item.detail.clone(), |el, detail| {
                                                    el.child(
                                                        with_default_font(div())
                                                            .text_xs()
                                                            .text_color(text_muted())
                                                            .child(detail),
                                                    )
                                                }),
                                        )
                                }),
                            )),
                    )
                    .with_priority(1),
                )
            })
    }
}

impl Focusable for Dropdown
{
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle
    {
        self.focus_handle.clone()
    }
}
