use gpui::EventEmitter;

// ====================
// Editor.
// ====================
use crate::gui::blocks::{KeyValueBlock, ScalarBlock, SequenceBlock};
use crate::gui::expr::xexpr::XExpr;
use crate::gui::facets::FacetEvent;
use crate::gui::facets::boolean::BoolFacet;
use crate::gui::facets::color::ColorFacet;
use crate::gui::facets::enumeration::EnumFacet;
use crate::gui::facets::expr::ExprFacet;
use crate::gui::facets::float::FloatFacet;
use crate::gui::facets::init_modifier::InitModifierFacet;
use crate::gui::facets::integer::IntegerFacet;
use crate::gui::facets::key_value::KeyValueFacet;
use crate::gui::facets::slider::SliderFacet;
use crate::gui::facets::text::TextFacet;
use crate::gui::facets::time_color::{TimeColor, TimeColorFacet};
use crate::gui::facets::time_vec2::{TimeVec2, TimeVec2Facet};
use crate::gui::facets::update_modifier::UpdateModifierFacet;
use crate::gui::models::color::HdrColor;
use crate::gui::models::key_value::KeyValueEntry;
use crate::gui::models::modifier::{XInitModifier, XUpdateModifier};

/// Block event types
#[derive(Debug, Clone)]
pub enum ScalarBlockEvent<V: Clone + std::fmt::Debug + Default>
{
    Changed
    {
        v: V
    },
}

/// Event for SequenceBlock.
#[derive(Debug, Clone)]
pub enum SequenceBlockEvent<V: Clone + std::fmt::Debug + Default>
{
    ItemChanged
    {
        index: usize, v: V
    },
    ItemAdded
    {
        index: usize, v: V
    },
    ItemRemoved
    {
        index: usize
    },
    ItemMoved
    {
        from_index: usize,
        to_index:   usize,
    },
}

/// Event for KeyValueBlock.
#[derive(Debug, Clone)]
pub enum KeyValueBlockEvent
{
    EntryChanged
    {
        index: usize, entry: KeyValueEntry
    },
    EntryAdded
    {
        index: usize, entry: KeyValueEntry
    },
    EntryRemoved
    {
        index: usize
    },
}

impl EventEmitter<FacetEvent<String>> for TextFacet {}
impl EventEmitter<FacetEvent<f32>> for FloatFacet {}
impl EventEmitter<FacetEvent<i32>> for IntegerFacet {}
impl EventEmitter<FacetEvent<bool>> for BoolFacet {}
impl EventEmitter<FacetEvent<HdrColor>> for ColorFacet {}
impl EventEmitter<FacetEvent<f32>> for SliderFacet {}
impl<E> EventEmitter<FacetEvent<E>> for EnumFacet<E> where
    E: 'static
        + strum::IntoEnumIterator
        + Clone
        + Default
        + std::fmt::Debug
        + std::fmt::Display
        + std::str::FromStr
{
}
impl EventEmitter<FacetEvent<Option<XExpr>>> for ExprFacet {}
impl EventEmitter<FacetEvent<KeyValueEntry>> for KeyValueFacet {}
impl EventEmitter<FacetEvent<TimeVec2>> for TimeVec2Facet {}
impl EventEmitter<FacetEvent<TimeColor>> for TimeColorFacet {}
impl EventEmitter<FacetEvent<XInitModifier>> for InitModifierFacet {}
impl EventEmitter<FacetEvent<XUpdateModifier>> for UpdateModifierFacet {}

impl<F> EventEmitter<ScalarBlockEvent<F::Value>> for ScalarBlock<F>
where
    F: crate::gui::facets::Facet + gpui::Render,
    F::Value: Clone + std::fmt::Debug + Default,
{
}

impl<F> EventEmitter<SequenceBlockEvent<F::Value>> for SequenceBlock<F>
where
    F: crate::gui::facets::Facet + gpui::Render,
    F::Value: Clone + std::fmt::Debug + Default,
{
}

impl EventEmitter<KeyValueBlockEvent> for KeyValueBlock {}

impl<F> EventEmitter<FacetEvent<F::Value>> for ScalarBlock<F>
where
    F: crate::gui::facets::Facet + gpui::Render,
    F::Value: Clone + std::fmt::Debug + Default,
{
}

impl<F> EventEmitter<FacetEvent<Vec<F::Value>>> for SequenceBlock<F>
where
    F: crate::gui::facets::Facet + gpui::Render,
    F::Value: Clone + std::fmt::Debug + Default,
{
}

impl EventEmitter<FacetEvent<Vec<KeyValueEntry>>> for KeyValueBlock {}
