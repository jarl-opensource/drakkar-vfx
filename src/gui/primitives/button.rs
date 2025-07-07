use gpui::prelude::*;
use gpui::{
    BoxShadow,
    ClickEvent,
    Context,
    Div,
    FontWeight,
    IntoElement,
    ParentElement,
    SharedString,
    Styled,
    Window,
    div,
    px,
};

// ====================
// Editor.
// ====================
use crate::gui::primitives::events::ButtonEvent;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::ProductIcon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant
{
    Primary,
    Secondary,
    Danger,
}

pub struct Button
{
    label:           SharedString,
    variant:         ButtonVariant,
    on_click:        Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut Context<Button>) + 'static>>,
    is_button_group: bool,
    is_left:         bool,
    font_size:       Option<&'static str>,
}

impl Button
{
    pub fn new(label: impl Into<SharedString>) -> Self
    {
        Self {
            label:           label.into(),
            variant:         ButtonVariant::Secondary,
            on_click:        None,
            is_button_group: false,
            is_left:         false,
            font_size:       None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self
    {
        self.variant = variant;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut Context<Button>) + 'static,
    ) -> Self
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn as_button_group_left(mut self) -> Self
    {
        self.is_button_group = true;
        self.is_left = true;
        self
    }

    pub fn as_button_group_right(mut self) -> Self
    {
        self.is_button_group = true;
        self.is_left = false;
        self
    }

    pub fn font_size(mut self, size: &'static str) -> Self
    {
        self.font_size = Some(size);
        self
    }

    fn handle_click(&mut self, event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>)
    {
        cx.emit(ButtonEvent::Clicked);
        if let Some(handler) = &self.on_click {
            handler(event, window, cx);
        }
    }
}

impl Render for Button
{
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let (bg_color, hover_color, text_color) = match self.variant {
            ButtonVariant::Primary => (button_primary(), button_primary_hover(), text_on_primary()),
            ButtonVariant::Secondary => {
                (button_secondary(), button_secondary_hover(), text_primary())
            }
            ButtonVariant::Danger => (button_danger(), button_danger_hover(), text_on_error()),
        };

        let mut element = div()
            .id("button")
            .px_1()
            .py_0()
            .bg(bg_color)
            .border_1()
            .border_color(border_subtle())
            .cursor_pointer()
            .on_click(cx.listener(Self::handle_click));

        // Apply button group styling if needed
        if self.is_button_group {
            if self.is_left {
                element = element.rounded_l_md().border_r_0();
            } else {
                element = element.rounded_r_md().border_l_0();
            }
        } else {
            element = element.rounded_md().shadow(vec![BoxShadow {
                color:         shadow_light(),
                offset:        gpui::point(px(0.), px(1.)),
                blur_radius:   px(1.),
                spread_radius: px(0.),
            }]);
        }

        let is_button_group = self.is_button_group;
        let hover_color = hover_color;
        element
            .hover(move |el| {
                let el = el.bg(hover_color).border_color(border_default());
                if !is_button_group {
                    el.shadow(vec![BoxShadow {
                        color:         shadow_medium(),
                        offset:        gpui::point(px(0.), px(2.)),
                        blur_radius:   px(3.),
                        spread_radius: px(0.),
                    }])
                } else {
                    el
                }
            })
            .child(
                with_default_font(div())
                    .when(self.font_size == Some("lg"), |el| el.text_lg())
                    .when(self.font_size == Some("xl"), |el| el.text_xl())
                    .when(self.font_size == Some("2xl"), |el| el.text_2xl())
                    .when(self.font_size == Some("3xl"), |el| el.text_3xl())
                    .when(self.font_size == Some("xs"), |el| el.text_xs())
                    .when(self.font_size == Some("sm"), |el| el.text_sm())
                    .when(self.font_size == Some("base"), |el| el.text_base())
                    .when(self.font_size.is_none(), |el| el.text_sm())
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(self.label.clone()),
            )
    }
}

pub fn render_toolbar_button<S: Into<String>>(text: S) -> impl IntoElement
{
    render_toolbar_button_with_icon(text, None)
}

pub fn render_toolbar_button_with_icon<S: Into<String>>(
    text: S,
    icon: Option<ProductIcon>,
) -> impl IntoElement
{
    div()
        .px_3()
        .py_2()
        .rounded_md()
        .bg(button_secondary())
        .border_1()
        .border_color(border_subtle())
        .shadow(vec![BoxShadow {
            color:         shadow_light(),
            offset:        gpui::point(px(0.), px(1.)),
            blur_radius:   px(1.),
            spread_radius: px(0.),
        }])
        .cursor_pointer()
        .hover(|el| {
            el.bg(button_primary())
                .border_color(accent_blue_light())
                .shadow(vec![
                    BoxShadow {
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
                    BoxShadow {
                        color:         shadow_medium(),
                        offset:        gpui::point(px(0.), px(2.)),
                        blur_radius:   px(6.),
                        spread_radius: px(0.),
                    },
                ])
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .when_some(icon, |el, icon| {
                    el.child(icon.to_svg().size_4().text_color(text_primary()))
                })
                .child(
                    with_default_font(div())
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_primary())
                        .child(text.into()),
                ),
        )
}

pub fn render_uniform_list_item_button() -> Div
{
    div()
        .w_full()
        .flex()
        .items_center()
        .px_3()
        .py_1()
        .cursor_pointer()
}

pub fn render_uniform_list_item_selected() -> Div
{
    render_uniform_list_item_button()
        .h_full()
        .bg(selection_active())
        .border_l_3()
        .border_color(accent_blue_light())
        .shadow(vec![BoxShadow {
            color:         shadow_light(),
            offset:        gpui::point(px(1.), px(0.)),
            blur_radius:   px(2.),
            spread_radius: px(0.),
        }])
}

pub fn render_uniform_list_item_button_unselected() -> Div
{
    render_uniform_list_item_button()
        .h_full()
        .border_l_3()
        .hover(|hover_el| {
            hover_el
                .bg(hover_subtle())
                .border_l_2()
                .border_color(border_separator())
        })
}
