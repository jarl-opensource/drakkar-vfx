use gpui::EventEmitter;

// ====================
// Editor.
// ====================
use crate::gui::blocks::{KeyValueBlock, ScalarBlock, SequenceBlock};
use crate::gui::expr::xexpr::XExpr;
use crate::gui::inspectors::InspectorEvent;
use crate::gui::inspectors::boolean::BoolInspector;
use crate::gui::inspectors::color::ColorInspector;
use crate::gui::inspectors::enumeration::EnumInspector;
use crate::gui::inspectors::expr::ExprInspector;
use crate::gui::inspectors::float::FloatInspector;
use crate::gui::inspectors::init_modifier::InitModifierInspector;
use crate::gui::inspectors::integer::IntegerInspector;
use crate::gui::inspectors::key_value::KeyValueInspector;
use crate::gui::inspectors::slider::SliderInspector;
use crate::gui::inspectors::text::TextInspector;
use crate::gui::inspectors::time_color::{TimeColor, TimeColorInspector};
use crate::gui::inspectors::time_vec2::{TimeVec2, TimeVec2Inspector};
use crate::gui::inspectors::update_modifier::UpdateModifierInspector;
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

impl EventEmitter<InspectorEvent<String>> for TextInspector {}
impl EventEmitter<InspectorEvent<f32>> for FloatInspector {}
impl EventEmitter<InspectorEvent<i32>> for IntegerInspector {}
impl EventEmitter<InspectorEvent<bool>> for BoolInspector {}
impl EventEmitter<InspectorEvent<HdrColor>> for ColorInspector {}
impl EventEmitter<InspectorEvent<f32>> for SliderInspector {}
impl<E> EventEmitter<InspectorEvent<E>> for EnumInspector<E> where
    E: 'static
        + strum::IntoEnumIterator
        + Clone
        + Default
        + std::fmt::Debug
        + std::fmt::Display
        + std::str::FromStr
{
}
impl EventEmitter<InspectorEvent<Option<XExpr>>> for ExprInspector {}
impl EventEmitter<InspectorEvent<KeyValueEntry>> for KeyValueInspector {}
impl EventEmitter<InspectorEvent<TimeVec2>> for TimeVec2Inspector {}
impl EventEmitter<InspectorEvent<TimeColor>> for TimeColorInspector {}
impl EventEmitter<InspectorEvent<XInitModifier>> for InitModifierInspector {}
impl EventEmitter<InspectorEvent<XUpdateModifier>> for UpdateModifierInspector {}

impl<F> EventEmitter<ScalarBlockEvent<F::Value>> for ScalarBlock<F>
where
    F: crate::gui::inspectors::Inspector + gpui::Render,
    F::Value: Clone + std::fmt::Debug + Default,
{
}

impl<F> EventEmitter<SequenceBlockEvent<F::Value>> for SequenceBlock<F>
where
    F: crate::gui::inspectors::Inspector + gpui::Render,
    F::Value: Clone + std::fmt::Debug + Default,
{
}

impl EventEmitter<KeyValueBlockEvent> for KeyValueBlock {}

impl<F> EventEmitter<InspectorEvent<F::Value>> for ScalarBlock<F>
where
    F: crate::gui::inspectors::Inspector + gpui::Render,
    F::Value: Clone + std::fmt::Debug + Default,
{
}

impl<F> EventEmitter<InspectorEvent<Vec<F::Value>>> for SequenceBlock<F>
where
    F: crate::gui::inspectors::Inspector + gpui::Render,
    F::Value: Clone + std::fmt::Debug + Default,
{
}

impl EventEmitter<InspectorEvent<Vec<KeyValueEntry>>> for KeyValueBlock {}
