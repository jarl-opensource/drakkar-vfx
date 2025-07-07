use gpui::prelude::*;
use gpui::{Context, Entity, Window, div};

// ====================
// Editor.
// ====================
use crate::gui::facets::boolean::BoolFacet;
use crate::gui::facets::float::FloatFacet;
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::models::modifier::XForceFieldSource;
use crate::gui::primitives::vec3_input::Vec3Input;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;

pub struct ForceFieldSourceFacet
{
    position:          Entity<Vec3Input>,
    max_radius:        Entity<FloatFacet>,
    min_radius:        Entity<FloatFacet>,
    mass:              Entity<FloatFacet>,
    force_exponent:    Entity<FloatFacet>,
    conform_to_sphere: Entity<BoolFacet>,

    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for ForceFieldSourceFacet
{
    type Value = XForceFieldSource;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        // Extract position as Vec3 from the XExpr
        let position_vec3 =
            if let crate::gui::expr::XExpr::Lit(crate::gui::expr::XValue::Vec3(x, y, z)) =
                &initial.position
            {
                bevy::math::Vec3::new(*x, *y, *z)
            } else {
                bevy::math::Vec3::ZERO
            };

        let position = cx.new(|cx| Vec3Input::with_value(cx, position_vec3));

        let max_radius = cx.new(|cx| {
            FloatFacet::new(
                cx,
                if let crate::gui::expr::XExpr::Lit(crate::gui::expr::XValue::Float(f)) =
                    &initial.max_radius
                {
                    *f
                } else {
                    10.0
                },
            )
        });
        let min_radius = cx.new(|cx| {
            FloatFacet::new(
                cx,
                if let crate::gui::expr::XExpr::Lit(crate::gui::expr::XValue::Float(f)) =
                    &initial.min_radius
                {
                    *f
                } else {
                    0.0
                },
            )
        });
        let mass = cx.new(|cx| {
            FloatFacet::new(
                cx,
                if let crate::gui::expr::XExpr::Lit(crate::gui::expr::XValue::Float(f)) =
                    &initial.mass
                {
                    *f
                } else {
                    1.0
                },
            )
        });
        let force_exponent = cx.new(|cx| {
            FloatFacet::new(
                cx,
                if let crate::gui::expr::XExpr::Lit(crate::gui::expr::XValue::Float(f)) =
                    &initial.force_exponent
                {
                    *f
                } else {
                    1.0
                },
            )
        });
        let conform_to_sphere = cx.new(|cx| BoolFacet::new(cx, initial.conform_to_sphere));

        let mut subscriptions = Vec::new();

        // Subscribe to position changes
        subscriptions.push(cx.subscribe(
            &position,
            |this, _entity, _event: &crate::gui::primitives::events::Vec3InputEvent, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        // Subscribe to other field changes
        subscriptions.push(cx.subscribe(
            &max_radius,
            |this, _entity, _event: &FacetEvent<f32>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &min_radius,
            |this, _entity, _event: &FacetEvent<f32>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(
            cx.subscribe(&mass, |this, _entity, _event: &FacetEvent<f32>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            }),
        );
        subscriptions.push(cx.subscribe(
            &force_exponent,
            |this, _entity, _event: &FacetEvent<f32>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &conform_to_sphere,
            |this, _entity, _event: &FacetEvent<bool>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));

        Self {
            position,
            max_radius,
            min_radius,
            mass,
            force_exponent,
            conform_to_sphere,
            _subscriptions: subscriptions,
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        let position_vec3 = self.position.read(cx).get_value(cx);
        let position = crate::gui::expr::XExpr::lit(crate::gui::expr::XValue::vec3(
            position_vec3.x,
            position_vec3.y,
            position_vec3.z,
        ));

        let max_radius = crate::gui::expr::XExpr::lit(self.max_radius.read(cx).get_value(cx));
        let min_radius = crate::gui::expr::XExpr::lit(self.min_radius.read(cx).get_value(cx));
        let mass = crate::gui::expr::XExpr::lit(self.mass.read(cx).get_value(cx));
        let force_exponent =
            crate::gui::expr::XExpr::lit(self.force_exponent.read(cx).get_value(cx));
        let conform_to_sphere = self.conform_to_sphere.read(cx).get_value(cx);

        XForceFieldSource {
            position,
            max_radius,
            min_radius,
            mass,
            force_exponent,
            conform_to_sphere,
        }
    }
}

impl Render for ForceFieldSourceFacet
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
                                .child("Position:"),
                        ),
                    )
                    .child(div().flex_1().child(self.position.clone())),
            )
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
                                .child("Max Radius:"),
                        ),
                    )
                    .child(div().flex_1().child(self.max_radius.clone())),
            )
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
                                .child("Min Radius:"),
                        ),
                    )
                    .child(div().flex_1().child(self.min_radius.clone())),
            )
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
                                .child("Mass:"),
                        ),
                    )
                    .child(div().flex_1().child(self.mass.clone())),
            )
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
                                .child("Force Exp:"),
                        ),
                    )
                    .child(div().flex_1().child(self.force_exponent.clone())),
            )
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
                                .child("Conform:"),
                        ),
                    )
                    .child(div().flex_1().child(self.conform_to_sphere.clone())),
            )
    }
}

impl
    gpui::EventEmitter<
        crate::gui::facets::FacetEvent<crate::gui::models::modifier::XForceFieldSource>,
    > for ForceFieldSourceFacet
{
}
