use gpui::prelude::*;
use gpui::{AnyView, Context, Entity, Window, div};
use strum::IntoEnumIterator;

use crate::gui::inspectors::enumeration::EnumInspector;
use crate::gui::inspectors::expr::ExprInspector;
use crate::gui::inspectors::{Inspector, InspectorEvent};
use crate::gui::models::XDimension;
use crate::gui::models::attr::XAttr;
use crate::gui::models::modifier::{
    XInitModifier,
    XSetAttributeModifier,
    XSetPositionCircleModifier,
    XSetPositionCone3dModifier,
    XSetPositionSphereModifier,
    XSetVelocityCircleModifier,
    XSetVelocitySphereModifier,
    XSetVelocityTangentModifier,
};
use crate::gui::primitives::dropdown_input::{Dropdown, DropdownItem, DropdownSizeVariant};
use crate::gui::primitives::events::DropdownEvent;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::ProductIcon;

pub struct InitModifierInspector
{
    type_dropdown:    Entity<Dropdown>,
    current_modifier: XInitModifier,

    // Expression properties for all possible fields
    center_expr:      Entity<ExprInspector>,
    axis_expr:        Entity<ExprInspector>,
    radius_expr:      Entity<ExprInspector>,
    speed_expr:       Entity<ExprInspector>,
    value_expr:       Entity<ExprInspector>,
    height_expr:      Entity<ExprInspector>,
    base_radius_expr: Entity<ExprInspector>,
    top_radius_expr:  Entity<ExprInspector>,

    // Enum properties
    dimension_enum:     Entity<EnumInspector<XDimension>>,
    attribute_dropdown: Entity<Dropdown>,

    _subscriptions: Vec<gpui::Subscription>,
}

impl Inspector for InitModifierInspector
{
    type Value = XInitModifier;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        let type_dropdown = cx.new(|cx| {
            let items: Vec<DropdownItem> = XInitModifier::iter()
                .map(|modifier| DropdownItem {
                    text:   modifier.to_string().into(),
                    icon:   Some(ProductIcon::Codepen),
                    detail: None,
                })
                .collect();

            let selected_index = XInitModifier::iter()
                .position(|v| std::mem::discriminant(&v) == std::mem::discriminant(&initial));

            Dropdown::new(cx)
                .with_items(items)
                .with_selected_index(selected_index)
                .with_size_variant(DropdownSizeVariant::Small)
        });

        // Initialize expression fields with values from the initial modifier
        let (
            center_init,
            axis_init,
            radius_init,
            speed_init,
            value_init,
            height_init,
            base_radius_init,
            top_radius_init,
            dimension_init,
            attribute_init,
        ) = Self::extract_initial_values(&initial);

        let center_expr = cx.new(|cx| ExprInspector::new(cx, center_init));
        let axis_expr = cx.new(|cx| ExprInspector::new(cx, axis_init));
        let radius_expr = cx.new(|cx| ExprInspector::new(cx, radius_init));
        let speed_expr = cx.new(|cx| ExprInspector::new(cx, speed_init));
        let value_expr = cx.new(|cx| ExprInspector::new(cx, value_init));
        let height_expr = cx.new(|cx| ExprInspector::new(cx, height_init));
        let base_radius_expr = cx.new(|cx| ExprInspector::new(cx, base_radius_init));
        let top_radius_expr = cx.new(|cx| ExprInspector::new(cx, top_radius_init));

        let dimension_enum = cx.new(|cx| EnumInspector::new(cx, dimension_init));

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
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        );

        let mut subscriptions = vec![dropdown_subscription];

        // Subscribe to all expression field changes
        subscriptions.push(cx.subscribe(
            &center_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &axis_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &radius_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &speed_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &value_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &height_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &base_radius_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &top_radius_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Subscribe to enum field changes
        subscriptions.push(cx.subscribe(
            &dimension_enum,
            |this, _entity, _event: &InspectorEvent<XDimension>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Subscribe to attribute dropdown changes
        subscriptions.push(cx.subscribe(
            &attribute_dropdown,
            |this, _dropdown, _event: &DropdownEvent, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        let instance = Self {
            type_dropdown,
            current_modifier: initial.clone(),
            center_expr,
            axis_expr,
            radius_expr,
            speed_expr,
            value_expr,
            height_expr,
            base_radius_expr,
            top_radius_expr,
            dimension_enum,
            attribute_dropdown,
            _subscriptions: subscriptions,
        };

        instance
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        match &self.current_modifier {
            XInitModifier::XSetPositionCircle(_) => {
                XInitModifier::XSetPositionCircle(XSetPositionCircleModifier {
                    center:    self
                        .center_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    axis:      self
                        .axis_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    radius:    self
                        .radius_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(1.0)),
                    dimension: self.dimension_enum.read(cx).get_value(cx),
                })
            }
            XInitModifier::XSetPositionSphere(_) => {
                XInitModifier::XSetPositionSphere(XSetPositionSphereModifier {
                    center:    self
                        .center_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    radius:    self
                        .radius_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(1.0)),
                    dimension: self.dimension_enum.read(cx).get_value(cx),
                })
            }
            XInitModifier::XSetPositionCone3d(_) => {
                XInitModifier::XSetPositionCone3d(XSetPositionCone3dModifier {
                    height:      self
                        .height_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(1.0)),
                    base_radius: self
                        .base_radius_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(1.0)),
                    top_radius:  self
                        .top_radius_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    dimension:   self.dimension_enum.read(cx).get_value(cx),
                })
            }
            XInitModifier::XSetVelocityCircle(_) => {
                XInitModifier::XSetVelocityCircle(XSetVelocityCircleModifier {
                    center: self
                        .center_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    axis:   self
                        .axis_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    speed:  self
                        .speed_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(1.0)),
                })
            }
            XInitModifier::XSetVelocitySphere(_) => {
                XInitModifier::XSetVelocitySphere(XSetVelocitySphereModifier {
                    center: self
                        .center_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    speed:  self
                        .speed_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(1.0)),
                })
            }
            XInitModifier::XSetVelocityTangent(_) => {
                XInitModifier::XSetVelocityTangent(XSetVelocityTangentModifier {
                    center: self
                        .center_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(0.0)),
                    speed:  self
                        .speed_expr
                        .read(cx)
                        .get_value(cx)
                        .unwrap_or_else(|| crate::gui::expr::XExpr::lit(1.0)),
                })
            }
            XInitModifier::XSetAttribute(_) => {
                let attr =
                    if let Some(selected_text) = self.attribute_dropdown.read(cx).get_selected() {
                        XAttr::iter()
                            .find(|attr| attr.to_string() == selected_text.to_string())
                            .unwrap_or_default()
                    } else {
                        XAttr::default()
                    };

                XInitModifier::XSetAttribute(XSetAttributeModifier {
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

impl InitModifierInspector
{
    fn on_type_changed(&mut self, type_index: usize, cx: &mut Context<Self>)
    {
        let new_modifier = XInitModifier::iter().nth(type_index).unwrap_or_default();
        self.current_modifier = new_modifier.clone();

        // Extract values for the new modifier type
        let (
            center_init,
            axis_init,
            radius_init,
            speed_init,
            value_init,
            height_init,
            base_radius_init,
            top_radius_init,
            dimension_init,
            attribute_init,
        ) = Self::extract_initial_values(&new_modifier);

        // Recreate the inspector instances with new values
        self.center_expr = cx.new(|cx| ExprInspector::new(cx, center_init));
        self.axis_expr = cx.new(|cx| ExprInspector::new(cx, axis_init));
        self.radius_expr = cx.new(|cx| ExprInspector::new(cx, radius_init));
        self.speed_expr = cx.new(|cx| ExprInspector::new(cx, speed_init));
        self.value_expr = cx.new(|cx| ExprInspector::new(cx, value_init));
        self.height_expr = cx.new(|cx| ExprInspector::new(cx, height_init));
        self.base_radius_expr = cx.new(|cx| ExprInspector::new(cx, base_radius_init));
        self.top_radius_expr = cx.new(|cx| ExprInspector::new(cx, top_radius_init));

        self.dimension_enum = cx.new(|cx| EnumInspector::new(cx, dimension_init));

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

        // Clear existing subscriptions and re-subscribe to all new entities
        self._subscriptions.clear();

        // Re-subscribe to dropdown
        self._subscriptions.push(cx.subscribe(
            &self.type_dropdown,
            |this, _dropdown, event: &DropdownEvent, cx| {
                let DropdownEvent::SelectionChanged(index) = event;
                this.on_type_changed(*index, cx);
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Re-subscribe to all expression field changes
        self._subscriptions.push(cx.subscribe(
            &self.center_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.axis_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.radius_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.speed_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.value_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.height_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.base_radius_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        self._subscriptions.push(cx.subscribe(
            &self.top_radius_expr,
            |this, _entity, _event: &InspectorEvent<Option<crate::gui::expr::XExpr>>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Re-subscribe to enum field changes
        self._subscriptions.push(cx.subscribe(
            &self.dimension_enum,
            |this, _entity, _event: &InspectorEvent<XDimension>, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Re-subscribe to attribute dropdown changes
        self._subscriptions.push(cx.subscribe(
            &self.attribute_dropdown,
            |this, _dropdown, _event: &DropdownEvent, cx| {
                cx.emit(InspectorEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        cx.notify();
    }

    fn extract_initial_values(
        modifier: &XInitModifier,
    ) -> (
        Option<crate::gui::expr::XExpr>, // center
        Option<crate::gui::expr::XExpr>, // axis
        Option<crate::gui::expr::XExpr>, // radius
        Option<crate::gui::expr::XExpr>, // speed
        Option<crate::gui::expr::XExpr>, // value
        Option<crate::gui::expr::XExpr>, // height
        Option<crate::gui::expr::XExpr>, // base_radius
        Option<crate::gui::expr::XExpr>, // top_radius
        XDimension,                      // dimension
        XAttr,                           // attribute
    )
    {
        match modifier {
            XInitModifier::XSetPositionCircle(m) => (
                Some(m.center.clone()),
                Some(m.axis.clone()),
                Some(m.radius.clone()),
                None,
                None,
                None,
                None,
                None,
                m.dimension.clone(),
                XAttr::default(),
            ),
            XInitModifier::XSetPositionSphere(m) => (
                Some(m.center.clone()),
                None,
                Some(m.radius.clone()),
                None,
                None,
                None,
                None,
                None,
                m.dimension.clone(),
                XAttr::default(),
            ),
            XInitModifier::XSetPositionCone3d(m) => (
                None,
                None,
                None,
                None,
                None,
                Some(m.height.clone()),
                Some(m.base_radius.clone()),
                Some(m.top_radius.clone()),
                m.dimension.clone(),
                XAttr::default(),
            ),
            XInitModifier::XSetVelocityCircle(m) => (
                Some(m.center.clone()),
                Some(m.axis.clone()),
                None,
                Some(m.speed.clone()),
                None,
                None,
                None,
                None,
                XDimension::default(),
                XAttr::default(),
            ),
            XInitModifier::XSetVelocitySphere(m) => (
                Some(m.center.clone()),
                None,
                None,
                Some(m.speed.clone()),
                None,
                None,
                None,
                None,
                XDimension::default(),
                XAttr::default(),
            ),
            XInitModifier::XSetVelocityTangent(m) => (
                Some(m.center.clone()),
                None,
                None,
                Some(m.speed.clone()),
                None,
                None,
                None,
                None,
                XDimension::default(),
                XAttr::default(),
            ),
            XInitModifier::XSetAttribute(m) => (
                None,
                None,
                None,
                None,
                Some(m.value.clone()),
                None,
                None,
                None,
                XDimension::default(),
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

impl Render for InitModifierInspector
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
                    XInitModifier::XSetPositionCircle(_) => el
                        .child(
                            self.render_field_row(
                                "Center",
                                AnyView::from(self.center_expr.clone()),
                            ),
                        )
                        .child(self.render_field_row("Axis", AnyView::from(self.axis_expr.clone())))
                        .child(
                            self.render_field_row(
                                "Radius",
                                AnyView::from(self.radius_expr.clone()),
                            ),
                        )
                        .child(self.render_field_row(
                            "Dimension",
                            AnyView::from(self.dimension_enum.clone()),
                        )),
                    XInitModifier::XSetPositionSphere(_) => el
                        .child(
                            self.render_field_row(
                                "Center",
                                AnyView::from(self.center_expr.clone()),
                            ),
                        )
                        .child(
                            self.render_field_row(
                                "Radius",
                                AnyView::from(self.radius_expr.clone()),
                            ),
                        )
                        .child(self.render_field_row(
                            "Dimension",
                            AnyView::from(self.dimension_enum.clone()),
                        )),
                    XInitModifier::XSetPositionCone3d(_) => el
                        .child(
                            self.render_field_row(
                                "Height",
                                AnyView::from(self.height_expr.clone()),
                            ),
                        )
                        .child(self.render_field_row(
                            "Base Radius",
                            AnyView::from(self.base_radius_expr.clone()),
                        ))
                        .child(self.render_field_row(
                            "Top Radius",
                            AnyView::from(self.top_radius_expr.clone()),
                        ))
                        .child(self.render_field_row(
                            "Dimension",
                            AnyView::from(self.dimension_enum.clone()),
                        )),
                    XInitModifier::XSetVelocityCircle(_) => el
                        .child(
                            self.render_field_row(
                                "Center",
                                AnyView::from(self.center_expr.clone()),
                            ),
                        )
                        .child(self.render_field_row("Axis", AnyView::from(self.axis_expr.clone())))
                        .child(
                            self.render_field_row("Speed", AnyView::from(self.speed_expr.clone())),
                        ),
                    XInitModifier::XSetVelocitySphere(_) => el
                        .child(
                            self.render_field_row(
                                "Center",
                                AnyView::from(self.center_expr.clone()),
                            ),
                        )
                        .child(
                            self.render_field_row("Speed", AnyView::from(self.speed_expr.clone())),
                        ),
                    XInitModifier::XSetVelocityTangent(_) => el
                        .child(
                            self.render_field_row(
                                "Center",
                                AnyView::from(self.center_expr.clone()),
                            ),
                        )
                        .child(
                            self.render_field_row("Speed", AnyView::from(self.speed_expr.clone())),
                        ),
                    XInitModifier::XSetAttribute(_) => el
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
