use gpui::prelude::*;
use gpui::{Context, Entity, Window, div};

// ====================
// Editor.
// ====================
use crate::gui::expr::XParseError;
use crate::gui::expr::xexpr::XExpr;
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::primitives::events::ExprInputEvent;
use crate::gui::primitives::expr_input::ExprInput;
use crate::gui::primitives::text_input::SizeVariant;
use crate::gui::styling::colors::*;
use crate::gui::styling::icons::ProductIcon;

/// Expression facet field for models editor.
/// Provides an expression input widget with syntax highlighting and auto-completion.
///
pub struct ExprFacet
{
    expr_input:     Entity<ExprInput>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for ExprFacet
{
    type Value = Option<XExpr>;

    /// Create a new expression facet with initial value.
    ///
    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let initial_content = if let Some(expr) = &initial {
            expr.to_string()
        } else {
            String::new()
        };

        let expr_input = cx.new(|cx| {
            ExprInput::new(cx)
                .with_content(initial_content, cx)
                .with_placeholder("Enter expression...")
                .with_size_variant(SizeVariant::Medium)
        });

        // Subscribe to expression input events
        let subscription = cx.subscribe(
            &expr_input,
            |this, _expr_input, event: &ExprInputEvent, cx| match event {
                ExprInputEvent::Change(_) => {
                    cx.emit(FacetEvent::Updated {
                        v: this.get_value(cx),
                    });
                }
                ExprInputEvent::Submit(_) => {
                    cx.emit(FacetEvent::Updated {
                        v: this.get_value(cx),
                    });
                }
            },
        );

        Self {
            expr_input,
            _subscriptions: vec![subscription],
        }
    }

    /// Get current expression text from the input.
    /// Returns the raw expression string, which can be parsed later if needed.
    ///
    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        if let Some(Ok(parsed)) = &self.expr_input.read(cx).get_parsed_expr() {
            return Some(parsed.clone());
        }
        None
    }
}

// ====================
// Additional methods.
// ====================

impl ExprFacet
{
    /// Check if the current expression is valid.
    /// Returns true if the expression can be parsed successfully.
    ///
    pub fn is_valid<T>(&self, cx: &Context<T>) -> bool
    {
        self.expr_input
            .read(cx)
            .get_parsed_expr()
            .map(|result| result.is_ok())
            .unwrap_or(false)
    }

    /// Get the parsed expression if valid.
    /// Returns None if the expression is invalid or empty.
    ///
    pub fn get_parsed_expr<T>(&self, cx: &Context<T>) -> Option<XExpr>
    {
        self.get_value(cx)
    }

    /// Get the parse error if the expression is invalid.
    ///
    pub fn get_error<T>(&self, cx: &Context<T>) -> Option<XParseError>
    {
        if let Some(Err(err)) = &self.expr_input.read(cx).get_parsed_expr() {
            return Some(err.clone());
        }
        None
    }
}

// ====================
// Rendering.
// ====================

impl Render for ExprFacet
{
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_valid = self.is_valid(cx);
        let has_content = !self.expr_input.read_with(cx, |input, cx| {
            input.text_input.read(cx).content.trim().is_empty()
        });

        div()
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .child(self.expr_input.clone().into_element())
            .when(has_content, |el| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w_6()
                        .h_6()
                        .child(if is_valid {
                            ProductIcon::Check
                                .to_svg()
                                .size_4()
                                .text_color(text_success())
                        } else {
                            ProductIcon::OctagonAlert
                                .to_svg()
                                .size_4()
                                .text_color(text_danger())
                        }),
                )
            })
    }
}
