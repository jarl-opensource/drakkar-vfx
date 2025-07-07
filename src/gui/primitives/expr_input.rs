// ====================
// GPUI.
// ====================

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::{
    BoxShadow,
    Context,
    Entity,
    FocusHandle,
    Focusable,
    IntoElement,
    KeyDownEvent,
    ParentElement,
    SharedString,
    Styled,
    Window,
    deferred,
    div,
    point,
    px,
    relative,
};

// ====================
// Editor.
// ====================
use crate::gui::expr::tokenizer::CompletionKind;
use crate::gui::expr::xexpr::XExpr;
use crate::gui::expr::xval::XExprReturnType;
use crate::gui::expr::{CompletionItem, XParseError, get_completions};
use crate::gui::primitives::dropdown_menu::{DropdownMenu, MenuItem};
use crate::gui::primitives::events::{DropdownMenuEvent, ExprInputEvent, TextInputEvent};
use crate::gui::primitives::expr_highlighter::ExprHighlighter;
use crate::gui::primitives::text_input::{SizeVariant, TextInput};
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::ProductIcon;

// ====================
// Types.
// ====================

pub struct ExprInput
{
    pub focus_handle:     FocusHandle,
    pub text_input:       Entity<TextInput<ExprHighlighter>>,
    pub completions_menu: Entity<DropdownMenu>,
    pub completions:      Vec<CompletionItem>,
    pub parsed_expr:      Option<Result<XExpr, XParseError>>,
    pub size_variant:     SizeVariant,
    pub placeholder:      SharedString,
    pub on_change:        Option<Box<dyn Fn(&str) + 'static>>,
    pub on_submit:        Option<Box<dyn Fn(&str) + 'static>>,
    pub matching_paren:   Option<(usize, usize)>, // (open_pos, close_pos)
    pub attributes:       HashMap<String, XExprReturnType>,
    pub props:            HashMap<String, XExprReturnType>,
}

// ====================
// Actions.
// ====================

pub mod actions
{
    use gpui::actions;
    actions!(expr_input, [Tab, Up, Down, Enter, Escape]);
}

impl ExprInput
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        let focus_handle = cx.focus_handle();

        let text_input = cx.new(|cx| {
            TextInput::with_highlighter(ExprHighlighter::new(), cx)
                .with_placeholder("Enter expression...")
                .with_size_variant(SizeVariant::Large)
        });

        let completions_menu = cx.new(|cx| DropdownMenu::new(cx).with_max_height(px(200.)));

        // Subscribe to text input changes
        cx.subscribe(&text_input, |this, _, event: &TextInputEvent, cx| {
            match event {
                TextInputEvent::Edited => {
                    let text = this.text_input.read(cx).content.clone();
                    this.on_text_changed(text, cx);
                }
                TextInputEvent::Confirmed => {
                    // Check if we have completions and should accept one instead of submitting
                    let has_completions = !this.completions_menu.read(cx).items.is_empty();

                    if has_completions {
                        // Accept the completion instead of submitting
                        this.accept_completion(cx);
                    } else {
                        // No completions, proceed with normal submit behavior
                        let content = this.get_content(cx);
                        if let Some(_handler) = &this.on_submit {
                            // TODO
                            // cx.update(|window, cx| {
                            //     _handler(&content, window, cx);
                            // });
                        }
                        cx.emit(ExprInputEvent::Submit(SharedString::from(content)));
                    }
                }
                _ => {}
            }
        })
        .detach();

        // Subscribe to menu events
        cx.subscribe(
            &completions_menu,
            |this, _, event: &DropdownMenuEvent, cx| match event {
                DropdownMenuEvent::ItemSelected(index) => {
                    if let Some(completion) = this.completions.get(*index) {
                        let insert_text = completion.insert_text.clone();
                        this.apply_completion_text(&insert_text, cx);
                    }
                }
                DropdownMenuEvent::Cancelled => {
                    this.completions_menu.update(cx, |menu, cx| {
                        menu.items.clear();
                        cx.notify();
                    });
                }
            },
        )
        .detach();

        Self {
            focus_handle,
            text_input,
            completions_menu,
            completions: Vec::new(),
            parsed_expr: None,
            size_variant: SizeVariant::Medium,
            placeholder: SharedString::from("Enter expression..."),
            on_change: None,
            on_submit: None,
            matching_paren: None,
            attributes: HashMap::new(),
            props: HashMap::new(),
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self
    {
        self.placeholder = placeholder.into();
        self
    }

    pub fn with_size_variant(mut self, size_variant: SizeVariant) -> Self
    {
        self.size_variant = size_variant;
        self
    }

    pub fn with_content(mut self, content: impl Into<SharedString>, cx: &mut Context<Self>)
    -> Self
    {
        let content_str = content.into();
        self.text_input.update(cx, |input, _| {
            input.content = content_str.clone();
        });
        self.on_text_changed(content_str, cx);
        self
    }

    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + 'static,
    {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + 'static,
    {
        self.on_submit = Some(Box::new(f));
        self
    }

    pub fn get_content(&self, cx: &Context<Self>) -> String
    {
        self.text_input.read(cx).content.to_string()
    }

    pub fn get_parsed_expr(&self) -> Option<&Result<XExpr, XParseError>>
    {
        self.parsed_expr.as_ref()
    }

    /// Get the inferred type of the current expression
    pub fn get_inferred_type(&self) -> Option<XExprReturnType>
    {
        self.parsed_expr
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .and_then(|expr| expr.get_result_type(&self.attributes, &self.props))
    }

    /// Set the available attributes and their types
    pub fn set_attributes(
        &mut self,
        attributes: HashMap<String, XExprReturnType>,
        cx: &mut Context<Self>,
    )
    {
        self.attributes = attributes;
        cx.notify();
    }

    /// Set the available properties and their types
    pub fn set_props(&mut self, props: HashMap<String, XExprReturnType>, cx: &mut Context<Self>)
    {
        self.props = props;
        cx.notify();
    }

    /// Add a single attribute type
    pub fn add_attribute(
        &mut self,
        name: impl Into<String>,
        return_type: XExprReturnType,
        cx: &mut Context<Self>,
    )
    {
        self.attributes.insert(name.into(), return_type);
        cx.notify();
    }

    /// Add a single property type
    pub fn add_prop(
        &mut self,
        name: impl Into<String>,
        return_type: XExprReturnType,
        cx: &mut Context<Self>,
    )
    {
        self.props.insert(name.into(), return_type);
        cx.notify();
    }

    /// Set both attributes and properties at once
    pub fn set_types(
        &mut self,
        attributes: HashMap<String, XExprReturnType>,
        props: HashMap<String, XExprReturnType>,
        cx: &mut Context<Self>,
    )
    {
        self.attributes = attributes;
        self.props = props;
        cx.notify();
    }

    /// Check if the current expression is valid.
    pub fn is_valid(&self) -> bool
    {
        self.parsed_expr
            .as_ref()
            .map(|result| result.is_ok())
            .unwrap_or(false)
    }

    /// Get the parse error if the expression is invalid.
    pub fn get_error(&self) -> Option<&XParseError>
    {
        self.parsed_expr
            .as_ref()
            .and_then(|result| result.as_ref().err())
    }

    /// Set the content of the expression input programmatically.
    pub fn set_content(&mut self, content: impl Into<SharedString>, cx: &mut Context<Self>)
    {
        self.text_input.update(cx, |input, _| {
            input.content = content.into();
        });
        let text = self.text_input.read(cx).content.clone();
        self.on_text_changed(text, cx);
    }

    /// Clear the expression input.
    pub fn clear(&mut self, cx: &mut Context<Self>)
    {
        self.set_content("", cx);
    }

    /// Focus the input field.
    pub fn focus(&mut self, window: &mut Window)
    {
        window.focus(&self.focus_handle);
    }

    /// Check if the input is focused.
    pub fn is_focused(&self, window: &Window) -> bool
    {
        self.focus_handle.is_focused(window)
    }

    // ====================
    // Event handlers.
    // ====================

    fn on_text_changed(&mut self, text: SharedString, cx: &mut Context<Self>)
    {
        // Parse the expression
        self.parsed_expr = Some(XExpr::parse(&text));

        // Update completions
        self.update_completions(&text, cx);

        // Update matching parentheses
        self.update_matching_parens(&text, cx);

        // Call the change handler
        if let Some(handler) = &self.on_change {
            let text_str = text.to_string();
            handler(&text_str);
        }

        cx.emit(ExprInputEvent::Change(text));
        cx.notify();
    }

    fn update_completions(&mut self, text: &str, cx: &mut Context<Self>)
    {
        // Get cursor position
        let cursor_pos = self.text_input.read(cx).selected_range.end;

        // Find the word being typed at cursor
        let prefix = self.get_prefix_at_cursor(text, cursor_pos);

        if prefix.is_empty() {
            self.completions.clear();
            self.completions_menu.update(cx, |menu, _cx| {
                menu.items.clear();
                menu.selected_index = None;
            });
        } else {
            self.completions = get_completions(&prefix, cursor_pos);

            // Create menu items before the closure to avoid borrow issues
            let menu_items: Vec<MenuItem> = self
                .completions
                .iter()
                .map(|comp| {
                    // Choose icon based on completion kind
                    let icon = match comp.kind {
                        CompletionKind::Function => Some(ProductIcon::SquareFunction),
                        CompletionKind::BuiltIn => Some(ProductIcon::SymbolEvent),
                        CompletionKind::Attribute | CompletionKind::Property => {
                            Some(ProductIcon::RectangleEllipsis)
                        }
                        _ => None,
                    };

                    // Add type description based on completion kind
                    let type_desc = match comp.kind {
                        CompletionKind::Function => "[Func.]",
                        CompletionKind::BuiltIn => "[Built-In]",
                        CompletionKind::Attribute => "[Attr.]",
                        CompletionKind::Property => "[Prop.]",
                        CompletionKind::Keyword => "[Keyword]",
                    };

                    let mut item = MenuItem::new(comp.label.clone());

                    // Combine existing detail with type description
                    let detail = match &comp.detail {
                        Some(existing_detail) => format!("{} {}", type_desc, existing_detail),
                        None => type_desc.to_string(),
                    };
                    item = item.with_detail(detail);

                    if let Some(icon) = icon {
                        item = item.with_icon(icon);
                    }
                    item
                })
                .collect();
            let has_items = !menu_items.is_empty();

            self.completions_menu.update(cx, |menu, _cx| {
                menu.items = menu_items;
                menu.selected_index = if has_items { Some(0) } else { None };
            });
        }
    }

    fn get_prefix_at_cursor(&self, text: &str, cursor_pos: usize) -> String
    {
        if cursor_pos == 0 || text.is_empty() {
            return String::new();
        }

        let chars: Vec<char> = text.chars().collect();

        // Start from cursor position and go backwards
        let mut start = cursor_pos;
        while start > 0 {
            let char_idx = start - 1;
            if char_idx >= chars.len() {
                break;
            }
            let ch = chars[char_idx];
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            start -= 1;
        }

        let result = if cursor_pos <= chars.len() {
            let prefix: String = chars[start..cursor_pos].iter().collect();
            prefix
        } else {
            String::new()
        };

        result
    }

    fn accept_completion(&mut self, cx: &mut Context<Self>)
    {
        let has_items = !self.completions_menu.read(cx).items.is_empty();
        if has_items {
            let index = self.completions_menu.read(cx).selected_index.unwrap_or(0);
            if let Some(completion) = self.completions.get(index) {
                let insert_text = completion.insert_text.clone();
                self.apply_completion_text(&insert_text, cx);
            }
        }
    }

    fn apply_completion_text(&mut self, completion_text: &str, cx: &mut Context<Self>)
    {
        let content = self.text_input.read(cx).content.to_string();
        let cursor_pos = self.text_input.read(cx).selected_range.end;
        let prefix = self.get_prefix_at_cursor(&content, cursor_pos);

        if !prefix.is_empty() {
            let start = cursor_pos - prefix.len();
            let new_content = format!(
                "{}{}{}",
                &content[..start],
                completion_text,
                &content[cursor_pos..]
            );

            self.text_input.update(cx, |input, _| {
                input.content = SharedString::from(new_content.clone());
                let new_cursor_pos = start + completion_text.len();
                input.selected_range = new_cursor_pos..new_cursor_pos;
            });

            // Manually trigger text change since we updated content directly
            let text = SharedString::from(new_content);
            self.on_text_changed(text, cx);
        }

        // Clear the menu and completions
        self.completions.clear();
        self.completions_menu.update(cx, |menu, _cx| {
            menu.items.clear();
        });

        cx.notify();
    }

    /// Update matching parentheses based on cursor position.
    fn update_matching_parens(&mut self, text: &str, cx: &mut Context<Self>)
    {
        let old_matching_paren = self.matching_paren;
        self.matching_paren = None;

        // If text is empty, just clear matching parens and return early
        if text.is_empty() {
            if old_matching_paren.is_some() {
                self.text_input.update(cx, |input, _| {
                    input.highlighter = ExprHighlighter::new();
                });
            }
            return;
        }

        // Get cursor position
        let cursor_pos = self.text_input.read(cx).selected_range.start;

        if cursor_pos == 0 || cursor_pos > text.len() {
            // Only update if we had matching parens before
            if old_matching_paren.is_some() {
                self.text_input.update(cx, |input, _| {
                    input.highlighter = ExprHighlighter::new();
                });
            }
            return;
        }

        let chars: Vec<char> = text.chars().collect();

        // Check if cursor is after a parenthesis
        let check_pos = if cursor_pos > 0 {
            cursor_pos - 1
        } else {
            cursor_pos
        };
        if check_pos < chars.len() {
            match chars[check_pos] {
                '(' => {
                    // Find matching closing paren
                    if let Some(close_pos) = self.find_matching_paren(&chars, check_pos, true) {
                        self.matching_paren = Some((check_pos, close_pos));
                    }
                }
                ')' => {
                    // Find matching opening paren
                    if let Some(open_pos) = self.find_matching_paren(&chars, check_pos, false) {
                        self.matching_paren = Some((open_pos, check_pos));
                    }
                }
                _ => {}
            }
        }

        // Only update the highlighter if the matching parens state actually changed
        if self.matching_paren != old_matching_paren {
            self.text_input.update(cx, |input, _| {
                input.highlighter =
                    ExprHighlighter::new().with_matching_parens(self.matching_paren);
            });
        }
    }

    /// Find matching parenthesis.
    fn find_matching_paren(&self, chars: &[char], start_pos: usize, forward: bool)
    -> Option<usize>
    {
        let (open_char, close_char) = ('(', ')');
        let mut depth = 1;

        if forward {
            for i in (start_pos + 1)..chars.len() {
                match chars[i] {
                    c if c == open_char => depth += 1,
                    c if c == close_char => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(i);
                        }
                    }
                    _ => {}
                }
            }
        } else {
            for i in (0..start_pos).rev() {
                match chars[i] {
                    c if c == close_char => depth += 1,
                    c if c == open_char => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(i);
                        }
                    }
                    _ => {}
                }
            }
        }

        None
    }
}

// ====================
// Rendering.
// ====================

impl Render for ExprInput
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_focused = self.focus_handle.is_focused(window)
            || self.text_input.read(cx).focus_handle.is_focused(window);
        let has_completions = !self.completions_menu.read(cx).items.is_empty();

        // Update text input properties only if needed
        let needs_update = self.text_input.read(cx).placeholder != self.placeholder
            || self.text_input.read(cx).size_variant != self.size_variant;

        if needs_update {
            self.text_input.update(cx, |input, cx| {
                input.placeholder = self.placeholder.clone();
                input.size_variant = self.size_variant;
                cx.notify();
            });
        }

        // Position dropdown based on text input size
        let dropdown_top = match self.size_variant {
            SizeVariant::Small => px(32.),
            SizeVariant::Medium => px(40.),
            SizeVariant::Large => px(48.),
        };

        div()
            .relative()
            .w_full()
            .key_context("ExprInput")
            .track_focus(&self.focus_handle)
            .when(has_completions && is_focused, |el| {
                el.on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                    match event.keystroke.key.as_str() {
                        "tab" => {
                            this.accept_completion(cx);
                            cx.stop_propagation();
                        }
                        "up" => {
                            this.completions_menu.update(cx, |menu, cx| {
                                menu.select_previous(cx);
                            });
                            cx.stop_propagation();
                        }
                        "down" => {
                            this.completions_menu.update(cx, |menu, cx| {
                                if menu.selected_index.is_none() && !menu.items.is_empty() {
                                    menu.selected_index = Some(0);
                                    cx.notify();
                                } else {
                                    menu.select_next(cx);
                                }
                            });
                            cx.stop_propagation();
                        }

                        "escape" => {
                            this.completions.clear();
                            this.completions_menu.update(cx, |menu, cx| {
                                menu.items.clear();
                                menu.selected_index = None;
                                cx.notify();
                            });
                            cx.stop_propagation();
                        }
                        _ => {}
                    }
                }))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .child(div().flex_1().child(self.text_input.clone()))
                    .when(self.get_error().is_none(), |el| {
                        let content = self.text_input.read(cx).content.to_string();
                        el.when(!content.is_empty(), |el| {
                            el.when_some(self.get_inferred_type(), |el, expr_type| {
                                el.child(self.render_inline_type_info(expr_type, cx))
                            })
                        })
                    }),
            )
            .when(has_completions && is_focused, |el| {
                el.child(
                    div()
                        .absolute()
                        .top(dropdown_top)
                        .left_0()
                        .right_0()
                        .child(self.completions_menu.clone()),
                )
            })
            .when_some(self.get_error(), |el, err| {
                let content = self.text_input.read(cx).content.to_string();
                el.when(!content.is_empty() && is_focused, |el| {
                    el.child(self.render_error(err, has_completions, window, cx))
                })
            })
    }
}

impl ExprInput
{
    fn render_error(
        &self,
        error: &XParseError,
        render_above: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement
    {
        let height = match self.size_variant {
            SizeVariant::Small => px(32.),
            SizeVariant::Medium => px(40.),
            SizeVariant::Large => px(48.),
        };

        // Choose colors based on error type - using less aggressive background, red only for border/shadow
        use crate::gui::expr::XParseError::*;
        let (bg_color, border_color, glow_color) = match error {
            // Syntax errors - Red theme
            UnexpectedToken(_) | UnexpectedEndOfInput | UnmatchedParenthesis => (
                background_darker(),
                error_panel_red_border(),
                error_panel_red_glow(),
            ),

            // Invalid values - Orange/Amber theme
            InvalidNumber(_) | InvalidIdentifier(_) => (
                background_darker(),
                error_panel_orange_border(),
                error_panel_orange_glow(),
            ),

            // Unknown references - Purple theme
            UnknownFunction(_) => (
                background_darker(),
                error_panel_purple_border(),
                error_panel_purple_glow(),
            ),

            // Type errors - Blue theme
            InvalidVectorLiteral => (
                background_darker(),
                error_panel_blue_border(),
                error_panel_blue_glow(),
            ),
        };

        deferred(
            div()
                .occlude()
                .absolute()
                .when(render_above, |el| el.bottom(height))
                .when(!render_above, |el| el.top(height))
                .left_0()
                .right_0()
                .when(render_above, |el| el.mb_1())
                .when(!render_above, |el| el.mt_1())
                .px_4()
                .py_3()
                .mx_2()
                .max_w(px(400.))
                .bg(bg_color)
                .shadow(vec![
                    BoxShadow {
                        color:         error_panel_shadow(),
                        offset:        point(px(0.), px(4.)),
                        blur_radius:   px(12.),
                        spread_radius: px(0.),
                    },
                    BoxShadow {
                        color:         glow_color,
                        offset:        point(px(0.), px(0.)),
                        blur_radius:   px(20.),
                        spread_radius: px(-4.),
                    },
                ])
                .border_1()
                .border_color(border_color)
                .rounded_lg()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            // Error icon
                            div()
                                .flex_shrink_0()
                                .size_5()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    ProductIcon::OctagonAlert
                                        .to_svg()
                                        .size_5()
                                        .text_color(text_danger()),
                                ),
                        )
                        .child(
                            with_default_font(div())
                                .text_sm()
                                .text_color(text_primary())
                                .font_weight(gpui::FontWeight::BOLD)
                                .line_height(relative(1.4))
                                .child(format!("{}", error)),
                        ),
                ),
        )
        .with_priority(1000)
    }

    fn render_inline_type_info(
        &self,
        expr_type: XExprReturnType,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement
    {
        // Choose colors and icon based on type
        let (text_color, icon) = match expr_type {
            XExprReturnType::Float => (text_success(), ProductIcon::SymbolNumeric),
            XExprReturnType::Integer => (text_success(), ProductIcon::SymbolNumeric),
            XExprReturnType::Vec2 => (text_info(), ProductIcon::SymbolArray),
            XExprReturnType::Vec3 => (text_info(), ProductIcon::SymbolArray),
            XExprReturnType::Error => (text_danger(), ProductIcon::OctagonAlert),
        };

        div()
            .flex()
            .items_center()
            .gap_2()
            .ml_2()
            .px_2()
            .py_1()
            .child(
                div()
                    .flex_shrink_0()
                    .size_4()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon.to_svg().size_4().text_color(text_color)),
            )
            .child(
                with_default_font(div())
                    .text_xs()
                    .text_color(text_color)
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(format!("{}", expr_type)),
            )
    }
}

impl Focusable for ExprInput
{
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle
    {
        self.focus_handle.clone()
    }
}
