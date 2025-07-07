use gpui::prelude::*;
use gpui::{AnyView, Context, Entity, Window, div};
use strum::IntoEnumIterator;

// ====================
// Editor.
// ====================
use crate::gui::facets::expr::ExprFacet;
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::models::attr::XAttr;
use crate::gui::models::modifier::{
    XAccelModifier,
    XLinearDragModifier,
    XRadialAccelModifier,
    XSetAttributeModifier,
    XTangentAccelModifier,
    XUpdateModifier,
};
use crate::gui::primitives::dropdown_input::{Dropdown, DropdownItem, DropdownSizeVariant};
use crate::gui::primitives::events::DropdownEvent;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::ProductIcon;

pub struct UpdateModifierFacet
{
    type_dropdown:      Entity<Dropdown>,
    current_modifier:   XUpdateModifier,
    accel_expr:         Entity<ExprFacet>,
    origin_expr:        Entity<ExprFacet>,
    axis_expr:          Entity<ExprFacet>,
    drag_expr:          Entity<ExprFacet>,
    value_expr:         Entity<ExprFacet>,
    attribute_dropdown: Entity<Dropdown>,
    _subscriptions:     Vec<gpui::Subscription>,
}

impl Facet for UpdateModifierFacet
{
    type Value = XUpdateModifier;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let type_dropdown = cx.new(|cx| {
            let items: Vec<DropdownItem> = XUpdateModifier::iter()
                .map(|modifier| DropdownItem {
                    text:   modifier.to_string().into(),
                    icon:   Some(ProductIcon::SymbolEvent),
                    detail: None,
                })
                .collect();

            let selected_index = XUpdateModifier::iter()
                .position(|v| std::mem::discriminant(&v) == std::mem::discriminant(&initial));

            Dropdown::new(cx)
                .with_items(items)
                .with_selected_index(selected_index)
                .with_size_variant(DropdownSizeVariant::Small)
        });

        // Initialize expression fields with values from the initial modifier
        let (accel_init, origin_init, axis_init, drag_init, value_init, attribute_init) =
            Self::extract_initial_values(&initial);

        let accel_expr = cx.new(|cx| ExprFacet::new(cx, accel_init));
        let origin_expr = cx.new(|cx| ExprFacet::new(cx, origin_init));
        let axis_expr = cx.new(|cx| ExprFacet::new(cx, axis_init));
        let drag_expr = cx.new(|cx| ExprFacet::new(cx, drag_init));
        let value_expr = cx.new(|cx| ExprFacet::new(cx, value_init));

        // Create custom attribute dropdown with icons and type information
        let attribute_dropdown = cx.new(|cx| {
            let items: Vec<DropdownItem> = XAttr::iter()
                .map(|attr| DropdownItem {
                    text:   attr.to_string().into(),
                    icon:   Some(ProductIcon::GitCommitHorizontal),
                    detail: Some(
                        format!("{} • {}", attr.get_type(), attr.get_description()).into(),
                    ),
                })
                .collect();

            let selected_index = XAttr::iter().position(|v| {
                std::mem::discriminant(&v) == std::mem::discriminant(&attribute_init)
            });

            Dropdown::new(cx)
                .with_items(items)
                .with_selected_index(selected_index)
                .with_size_variant(DropdownSizeVariant::Small)
        });

        let dropdown_subscription = cx.subscribe(
            &type_dropdown,
            |this, _dropdown, event: &DropdownEvent, cx| {
                let DropdownEvent::SelectionChanged(index) = event;
                this.on_type_changed(*index, cx);
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        );

        let mut subscriptions = vec![dropdown_subscription];

        // Subscribe to all expression field changes
        subscriptions.push(cx.subscribe(
            &accel_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &origin_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &axis_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &drag_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &value_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Subscribe to attribute dropdown changes
        subscriptions.push(cx.subscribe(
            &attribute_dropdown,
            |this, _dropdown, _event: &DropdownEvent, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        let instance = Self {
            type_dropdown,
            current_modifier: initial.clone(),
            accel_expr,
            origin_expr,
            axis_expr,
            drag_expr,
            value_expr,
            attribute_dropdown,
            _subscriptions: subscriptions,
        };

        instance
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        match &self.current_modifier {
            XUpdateModifier::XAccel(_) => XUpdateModifier::XAccel(XAccelModifier {
                accel: self
                    .accel_expr
                    .read(cx)
                    .get_value(cx)
                    .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
            }),
            XUpdateModifier::XRadialAccel(_) => {
                XUpdateModifier::XRadialAccel(XRadialAccelModifier {
                    origin: self
                        .origin_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    accel:  self
                        .accel_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                })
            }
            XUpdateModifier::XTangentAccel(_) => {
                XUpdateModifier::XTangentAccel(XTangentAccelModifier {
                    origin: self
                        .origin_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    axis:   self
                        .axis_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    accel:  self
                        .accel_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                })
            }

            XUpdateModifier::XLinearDrag(_) => XUpdateModifier::XLinearDrag(XLinearDragModifier {
                drag: self
                    .drag_expr
                    .read(cx)
                    .get_value(cx)
                    .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.1)),
            }),
            XUpdateModifier::XSetAttribute(_) => {
                let attr =
                    if let Some(selected_text) = self.attribute_dropdown.read(cx).get_selected() {
                        XAttr::iter()
                            .find(|attr| attr.to_string() == selected_text.to_string())
                            .unwrap_or_default()
                    } else {
                        XAttr::default()
                    };

                XUpdateModifier::XSetAttribute(XSetAttributeModifier {
                    attr,
                    value: self
                        .value_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                })
            }
        }
    }
}

impl UpdateModifierFacet
{
    fn on_type_changed(&mut self, type_index: usize, cx: &mut Context<Self>)
    {
        let new_modifier = XUpdateModifier::iter().nth(type_index).unwrap_or_default();
        self.current_modifier = new_modifier.clone();

        // Extract values for the new modifier type
        let (accel_init, origin_init, axis_init, drag_init, value_init, attribute_init) =
            Self::extract_initial_values(&new_modifier);

        // Recreate the facet instances with new values
        self.accel_expr = cx.new(|cx| ExprFacet::new(cx, accel_init));
        self.origin_expr = cx.new(|cx| ExprFacet::new(cx, origin_init));
        self.axis_expr = cx.new(|cx| ExprFacet::new(cx, axis_init));
        self.drag_expr = cx.new(|cx| ExprFacet::new(cx, drag_init));
        self.value_expr = cx.new(|cx| ExprFacet::new(cx, value_init));

        // Recreate attribute dropdown with icons and type information
        self.attribute_dropdown = cx.new(|cx| {
            let items: Vec<DropdownItem> = XAttr::iter()
                .map(|attr| DropdownItem {
                    text:   attr.to_string().into(),
                    icon:   Some(ProductIcon::GitCommitHorizontal),
                    detail: Some(
                        format!("{} • {}", attr.get_type(), attr.get_description()).into(),
                    ),
                })
                .collect();

            let selected_index = XAttr::iter().position(|v| {
                std::mem::discriminant(&v) == std::mem::discriminant(&attribute_init)
            });

            Dropdown::new(cx)
                .with_items(items)
                .with_selected_index(selected_index)
                .with_size_variant(DropdownSizeVariant::Small)
        });

        // Clear existing subscriptions and re-subscribe to dropdown
        self._subscriptions.clear();
        self._subscriptions.push(cx.subscribe(
            &self.type_dropdown,
            |this, _dropdown, event: &DropdownEvent, cx| {
                let DropdownEvent::SelectionChanged(index) = event;
                this.on_type_changed(*index, cx);
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Re-subscribe to all expression field changes
        self._subscriptions.push(cx.subscribe(
            &self.accel_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.origin_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.axis_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.drag_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.value_expr,
            |this, _entity, _event: &FacetEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Re-subscribe to attribute dropdown changes
        self._subscriptions.push(cx.subscribe(
            &self.attribute_dropdown,
            |this, _dropdown, _event: &DropdownEvent, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        cx.notify();
    }

    fn extract_initial_values(
        modifier: &XUpdateModifier,
    ) -> (
        Option<crate::gui::expr::XExpr>, // accel
        Option<crate::gui::expr::XExpr>, // origin
        Option<crate::gui::expr::XExpr>, // axis
        Option<crate::gui::expr::XExpr>, // drag
        Option<crate::gui::expr::XExpr>, // value
        XAttr,                           // attribute
    )
    {
        match modifier {
            XUpdateModifier::XAccel(m) => (
                Some(m.accel.clone()),
                None,
                None,
                None,
                None,
                XAttr::default(),
            ),
            XUpdateModifier::XRadialAccel(m) => (
                Some(m.accel.clone()),
                Some(m.origin.clone()),
                None,
                None,
                None,
                XAttr::default(),
            ),
            XUpdateModifier::XTangentAccel(m) => (
                Some(m.accel.clone()),
                Some(m.origin.clone()),
                Some(m.axis.clone()),
                None,
                None,
                XAttr::default(),
            ),

            XUpdateModifier::XLinearDrag(m) => (
                None,
                None,
                None,
                Some(m.drag.clone()),
                None,
                XAttr::default(),
            ),
            XUpdateModifier::XSetAttribute(m) => (
                None,
                None,
                None,
                None,
                Some(m.value.clone()),
                m.attr.clone(),
            ),
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
                div().w_20().flex().justify_end().child(
                    with_default_font(div())
                        .text_xs()
                        .text_color(text_muted())
                        .child(format!("{}:", label)),
                ),
            )
            .child(div().flex_1().child(content))
    }
}

impl Render for UpdateModifierFacet
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div().w_20().flex().justify_end().child(
                            with_default_font(div())
                                .text_xs()
                                .text_color(text_muted())
                                .child("Type:"),
                        ),
                    )
                    .child(div().flex_1().child(self.type_dropdown.clone())),
            )
            .child(div().flex().flex_col().gap_1().pl_2().when(true, |el| {
                match &self.current_modifier {
                    XUpdateModifier::XAccel(_) => el.child(
                        self.render_field_row("Accel", AnyView::from(self.accel_expr.clone())),
                    ),
                    XUpdateModifier::XRadialAccel(_) => el
                        .child(
                            self.render_field_row(
                                "Origin",
                                AnyView::from(self.origin_expr.clone()),
                            ),
                        )
                        .child(
                            self.render_field_row("Accel", AnyView::from(self.accel_expr.clone())),
                        ),
                    XUpdateModifier::XTangentAccel(_) => el
                        .child(
                            self.render_field_row(
                                "Origin",
                                AnyView::from(self.origin_expr.clone()),
                            ),
                        )
                        .child(self.render_field_row("Axis", AnyView::from(self.axis_expr.clone())))
                        .child(
                            self.render_field_row("Accel", AnyView::from(self.accel_expr.clone())),
                        ),

                    XUpdateModifier::XLinearDrag(_) => el.child(
                        self.render_field_row("Drag", AnyView::from(self.drag_expr.clone())),
                    ),
                    XUpdateModifier::XSetAttribute(_) => el
                        .child(self.render_field_row(
                            "Attribute",
                            AnyView::from(self.attribute_dropdown.clone()),
                        ))
                        .child(
                            self.render_field_row("Value", AnyView::from(self.value_expr.clone())),
                        ),
                }
            }))
    }
}
