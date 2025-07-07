use gpui::prelude::*;
use gpui::{AnyView, Context, Entity, EventEmitter, Window, div, px};
use strum::IntoEnumIterator;

// ====================
// Editor.
// ====================
use crate::gui::facets::boolean::BoolFacet;
use crate::gui::facets::float::FloatFacet;
use crate::gui::facets::{Facet, FacetEvent};
use crate::gui::primitives::dropdown_input::{Dropdown, DropdownItem, DropdownSizeVariant};
use crate::gui::primitives::events::DropdownEvent;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::ProductIcon;

impl EventEmitter<FacetEvent<SpawnerData>> for SpawnerFacet {}

#[derive(Debug, Clone, PartialEq, strum::EnumIter, strum::Display)]
pub enum SpawnerType
{
    Once,
    Burst,
    Rate,
    Custom,
}

impl Default for SpawnerType
{
    fn default() -> Self
    {
        SpawnerType::Once
    }
}

pub struct SpawnerFacet
{
    type_dropdown: Entity<Dropdown>,
    current_type:  SpawnerType,

    // Common properties
    num_particles:      Entity<FloatFacet>,
    starts_active:      Entity<BoolFacet>,
    starts_immediately: Entity<BoolFacet>,

    // Type-specific properties
    spawn_time: Entity<FloatFacet>,
    period:     Entity<FloatFacet>,

    _subscriptions: Vec<gpui::Subscription>,
}

impl Facet for SpawnerFacet
{
    type Value = SpawnerData;

    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self
    {
        // Determine spawner type from initial data
        let spawner_type = Self::determine_spawner_type(&initial);

        let type_dropdown = cx.new(|cx| {
            let items: Vec<DropdownItem> = SpawnerType::iter()
                .map(|spawner_type| DropdownItem {
                    text:   spawner_type.to_string().into(),
                    icon:   Some(ProductIcon::SymbolEvent),
                    detail: None,
                })
                .collect();

            let selected_index = SpawnerType::iter()
                .position(|v| v == spawner_type)
                .unwrap_or(0);

            Dropdown::new(cx)
                .with_items(items)
                .with_selected_index(Some(selected_index))
                .with_size_variant(DropdownSizeVariant::Small)
        });

        let num_particles = cx.new(|cx| FloatFacet::new(cx, initial.num_particles));
        let starts_active = cx.new(|cx| BoolFacet::new(cx, initial.starts_active));
        let starts_immediately = cx.new(|cx| BoolFacet::new(cx, initial.starts_immediately));
        let spawn_time = cx.new(|cx| FloatFacet::new(cx, initial.spawn_time));
        let period = cx.new(|cx| FloatFacet::new(cx, initial.period));

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
        subscriptions.push(cx.subscribe(
            &num_particles,
            |this, _entity, _event: &FacetEvent<f32>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &starts_active,
            |this, _entity, _event: &FacetEvent<bool>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &starts_immediately,
            |this, _entity, _event: &FacetEvent<bool>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(cx.subscribe(
            &spawn_time,
            |this, _entity, _event: &FacetEvent<f32>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            },
        ));
        subscriptions.push(
            cx.subscribe(&period, |this, _entity, _event: &FacetEvent<f32>, cx| {
                cx.emit(FacetEvent::Updated {
                    v: this.get_value(cx),
                });
            }),
        );

        Self {
            type_dropdown,
            current_type: spawner_type,
            num_particles,
            starts_active,
            starts_immediately,
            spawn_time,
            period,
            _subscriptions: subscriptions,
        }
    }

    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value
    {
        let num_particles = self.num_particles.read(cx).get_value(cx);
        let starts_active = self.starts_active.read(cx).get_value(cx);
        let starts_immediately = self.starts_immediately.read(cx).get_value(cx);
        let spawn_time = self.spawn_time.read(cx).get_value(cx);
        let period = self.period.read(cx).get_value(cx);

        // Adjust values based on spawner type
        let (adjusted_spawn_time, adjusted_period) = match self.current_type {
            SpawnerType::Once => (0.0, 1000.0),
            SpawnerType::Burst => (0.0, period),
            SpawnerType::Rate => (1.0, 1.0),
            SpawnerType::Custom => (spawn_time, period),
        };

        SpawnerData {
            num_particles,
            spawn_time: adjusted_spawn_time,
            period: adjusted_period,
            starts_active,
            starts_immediately,
        }
    }
}

impl SpawnerFacet
{
    fn determine_spawner_type(data: &SpawnerData) -> SpawnerType
    {
        if data.period.is_infinite() {
            SpawnerType::Once
        } else if data.spawn_time < f32::EPSILON && data.period > 0.0 {
            SpawnerType::Burst
        } else if (data.period - data.spawn_time).abs() < f32::EPSILON {
            SpawnerType::Rate
        } else {
            SpawnerType::Custom
        }
    }

    fn on_type_changed(&mut self, type_index: usize, cx: &mut Context<Self>)
    {
        let new_type = SpawnerType::iter().nth(type_index).unwrap_or_default();
        self.current_type = new_type.clone();
        cx.notify();
    }

    fn render_field_row(&self, label: &str, content: AnyView) -> impl IntoElement
    {
        div()
            .flex()
            .items_center()
            .gap_2()
            .mb_1()
            .child(
                div().w(px(140.0)).flex().justify_end().child(
                    with_default_font(div())
                        .text_xs()
                        .text_color(text_muted())
                        .child(format!("{}:", label)),
                ),
            )
            .child(div().flex_1().child(content))
    }
}

impl Render for SpawnerFacet
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .pr_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div().w_20().flex().justify_end().pr_2().child(
                            with_default_font(div())
                                .text_xs()
                                .text_color(text_muted())
                                .child("Type:"),
                        ),
                    )
                    .child(div().flex_1().child(self.type_dropdown.clone())),
            )
            .child(div().flex().flex_col().gap_1().pl_2().when(true, |el| {
                el.child(
                    self.render_field_row(
                        "Num Particles",
                        AnyView::from(self.num_particles.clone()),
                    ),
                )
                .child(
                    self.render_field_row(
                        "Starts Active",
                        AnyView::from(self.starts_active.clone()),
                    ),
                )
                .child(self.render_field_row(
                    "Starts Immediately",
                    AnyView::from(self.starts_immediately.clone()),
                ))
                .when(matches!(self.current_type, SpawnerType::Custom), |el| {
                    el.child(
                        self.render_field_row("Spawn Time", AnyView::from(self.spawn_time.clone())),
                    )
                    .child(self.render_field_row("Period", AnyView::from(self.period.clone())))
                })
                .when(matches!(self.current_type, SpawnerType::Burst), |el| {
                    el.child(self.render_field_row("Period", AnyView::from(self.period.clone())))
                })
            }))
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpawnerData
{
    pub num_particles:      f32,
    pub spawn_time:         f32,
    pub period:             f32,
    pub starts_active:      bool,
    pub starts_immediately: bool,
}
