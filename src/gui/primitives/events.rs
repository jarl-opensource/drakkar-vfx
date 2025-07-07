use gpui::EventEmitter;

// ====================
// Editor.
// ====================
use crate::gui::primitives::button::Button;
use crate::gui::primitives::checkbox_input::Checkbox;
use crate::gui::primitives::color_picker_input::ColorPicker;
use crate::gui::primitives::dropdown_input::Dropdown;
use crate::gui::primitives::dropdown_menu::DropdownMenu;
use crate::gui::primitives::expr_input::ExprInput;
use crate::gui::primitives::slider::Slider;
use crate::gui::primitives::vec2_input::Vec2Input;
use crate::gui::primitives::vec3_input::Vec3Input;

/// Event for TextInput objects.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextInputEvent
{
    Confirmed,
    Cancelled,
    Edited,
}

/// Event for Checkbox objects.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxEvent
{
    Changed(bool),
}

/// Event for Dropdown objects.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropdownEvent
{
    SelectionChanged(usize),
}

impl gpui::EventEmitter<DropdownEvent> for Dropdown {}

/// Event for ColorPicker objects.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorPickerEvent
{
    ColorChanged(gpui::Rgba),
    IntensityChanged(f32),
    ValuesChanged
    {
        color:       gpui::Rgba,
        intensity:   f32,
        final_color: gpui::Rgba,
    },
}

impl gpui::EventEmitter<ColorPickerEvent> for ColorPicker {}

/// Event for Slider objects.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SliderEvent
{
    ValueChanged(f32),
}

impl gpui::EventEmitter<SliderEvent> for Slider {}

/// Event for Vec2Input objects.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Vec2InputEvent
{
    Changed(bevy::math::Vec2),
}

/// Event for Vec3Input objects.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Vec3InputEvent
{
    Changed(bevy::math::Vec3),
}

/// Event for Button objects.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent
{
    Clicked,
}

/// Event for ExprInput objects.
///
#[derive(Debug, Clone, PartialEq)]
pub enum ExprInputEvent
{
    Change(gpui::SharedString),
    Submit(gpui::SharedString),
}

/// Event for DropdownMenu objects.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropdownMenuEvent
{
    ItemSelected(usize),
    Cancelled,
}

impl EventEmitter<Vec2InputEvent> for Vec2Input {}

impl EventEmitter<Vec3InputEvent> for Vec3Input {}

impl EventEmitter<ButtonEvent> for Button {}

impl EventEmitter<ExprInputEvent> for ExprInput {}

impl EventEmitter<DropdownMenuEvent> for DropdownMenu {}

impl EventEmitter<CheckboxEvent> for Checkbox {}
