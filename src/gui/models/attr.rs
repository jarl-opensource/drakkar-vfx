use strum_macros::{Display, EnumIter, EnumString};

use crate::gui::expr::xval::XExprReturnType;

#[derive(Clone, Debug, PartialEq, EnumIter, Display, Default, EnumString)]
pub enum XAttr
{
    #[default]
    #[strum(serialize = "position")]
    Position,
    #[strum(serialize = "velocity")]
    Velocity,
    #[strum(serialize = "age")]
    Age,
    #[strum(serialize = "lifetime")]
    Lifetime,
    #[strum(serialize = "color")]
    Color,
    #[strum(serialize = "alpha")]
    Alpha,
    #[strum(serialize = "size")]
    Size,
    #[strum(serialize = "size2")]
    Size2,
    #[strum(serialize = "axis_x")]
    AxisX,
    #[strum(serialize = "axis_y")]
    AxisY,
    #[strum(serialize = "axis_z")]
    AxisZ,
}

impl XAttr
{
    /// Get the expected type for this attribute
    pub fn get_type(&self) -> XExprReturnType
    {
        match self {
            XAttr::Position => XExprReturnType::Vec3,
            XAttr::Velocity => XExprReturnType::Vec3,
            XAttr::Age => XExprReturnType::Float,
            XAttr::Lifetime => XExprReturnType::Float,
            XAttr::Color => XExprReturnType::Vec3,
            XAttr::Alpha => XExprReturnType::Float,
            XAttr::Size => XExprReturnType::Float,
            XAttr::Size2 => XExprReturnType::Vec2,
            XAttr::AxisX => XExprReturnType::Vec3,
            XAttr::AxisY => XExprReturnType::Vec3,
            XAttr::AxisZ => XExprReturnType::Vec3,
        }
    }

    /// Get a human-readable description of this attribute
    pub fn get_description(&self) -> &'static str
    {
        match self {
            XAttr::Position => "3D position in world space",
            XAttr::Velocity => "3D velocity vector",
            XAttr::Age => "Current age in seconds",
            XAttr::Lifetime => "Total lifetime in seconds",
            XAttr::Color => "RGB color values (0-1)",
            XAttr::Alpha => "Alpha transparency (0-1)",
            XAttr::Size => "Uniform size scale",
            XAttr::Size2 => "2D size scale (width, height)",
            XAttr::AxisX => "X-axis orientation vector",
            XAttr::AxisY => "Y-axis orientation vector",
            XAttr::AxisZ => "Z-axis orientation vector",
        }
    }
}
