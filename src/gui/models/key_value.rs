use bevy::math::{Vec2, Vec3};
use gpui::SharedString;

use crate::gui::models::color::HdrColor;
use crate::gui::primitives::dropdown_input::DropdownItem;
use crate::gui::styling::icons::ProductIcon;

#[derive(Clone, Debug, PartialEq)]
pub enum ValueType
{
    Float,
    Integer,
    Vec2,
    Vec3,
    Color,
}

impl Default for ValueType
{
    fn default() -> Self
    {
        Self::Float
    }
}

impl ValueType
{
    pub fn variants() -> Vec<DropdownItem>
    {
        vec![
            DropdownItem {
                text:   SharedString::from("Float"),
                icon:   Some(ProductIcon::SymbolNumeric),
                detail: None,
            },
            DropdownItem {
                text:   SharedString::from("Integer"),
                icon:   Some(ProductIcon::SymbolNumeric),
                detail: None,
            },
            DropdownItem {
                text:   SharedString::from("Vec2"),
                icon:   Some(ProductIcon::SymbolArray),
                detail: None,
            },
            DropdownItem {
                text:   SharedString::from("Vec3"),
                icon:   Some(ProductIcon::SymbolArray),
                detail: None,
            },
            DropdownItem {
                text:   SharedString::from("Color"),
                icon:   Some(ProductIcon::SymbolColor),
                detail: None,
            },
        ]
    }

    pub fn from_index(index: usize) -> Self
    {
        match index {
            0 => Self::Float,
            1 => Self::Integer,
            2 => Self::Vec2,
            3 => Self::Vec3,
            4 => Self::Color,
            _ => panic!("Invalid index for KeyValue index: {}", index),
        }
    }

    pub fn to_index(&self) -> usize
    {
        match self {
            Self::Float => 0,
            Self::Integer => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Color => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeyValue
{
    Float(f32),
    Integer(i32),
    Vec2(Vec2),
    Vec3(Vec3),
    Color(HdrColor),
}

impl Default for KeyValue
{
    fn default() -> Self
    {
        Self::Float(0.0)
    }
}

impl KeyValue
{
    pub fn get_type(&self) -> ValueType
    {
        match self {
            Self::Float(_) => ValueType::Float,
            Self::Integer(_) => ValueType::Integer,
            Self::Vec2(_) => ValueType::Vec2,
            Self::Vec3(_) => ValueType::Vec3,
            Self::Color(_) => ValueType::Color,
        }
    }

    pub fn with_type(value_type: ValueType) -> Self
    {
        match value_type {
            ValueType::Float => Self::Float(0.0),
            ValueType::Integer => Self::Integer(0),
            ValueType::Vec2 => Self::Vec2(Vec2::ZERO),
            ValueType::Vec3 => Self::Vec3(Vec3::ZERO),
            ValueType::Color => Self::Color(HdrColor::default()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyValueEntry
{
    pub key:   SharedString,
    pub value: KeyValue,
}

impl KeyValueEntry
{
    pub fn new(key: impl Into<SharedString>, value: KeyValue) -> Self
    {
        Self {
            key: key.into(),
            value,
        }
    }
}
