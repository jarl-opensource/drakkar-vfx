use strum_macros::{Display, EnumIter, EnumString};

// ====================
// Editor.
// ====================
use crate::gui::expr::xexpr::XExpr;
use crate::gui::models::XDimension;
use crate::gui::models::attr::XAttr;

#[derive(Clone, Debug, PartialEq)]
pub struct XSetPositionCircleModifier
{
    pub center:    XExpr,
    pub axis:      XExpr,
    pub radius:    XExpr,
    pub dimension: XDimension,
}

impl Default for XSetPositionCircleModifier
{
    fn default() -> Self
    {
        Self {
            center:    XExpr::lit(0.0),
            axis:      XExpr::lit(0.0),
            radius:    XExpr::lit(1.0),
            dimension: XDimension::Surface,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XSetPositionSphereModifier
{
    pub center:    XExpr,
    pub radius:    XExpr,
    pub dimension: XDimension,
}

impl Default for XSetPositionSphereModifier
{
    fn default() -> Self
    {
        Self {
            center:    XExpr::lit(0.0),
            radius:    XExpr::lit(1.0),
            dimension: XDimension::Surface,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XSetPositionCone3dModifier
{
    pub height:      XExpr,
    pub base_radius: XExpr,
    pub top_radius:  XExpr,
    pub dimension:   XDimension,
}

impl Default for XSetPositionCone3dModifier
{
    fn default() -> Self
    {
        Self {
            height:      XExpr::lit(1.0),
            base_radius: XExpr::lit(1.0),
            top_radius:  XExpr::lit(0.0),
            dimension:   XDimension::Surface,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XSetVelocityCircleModifier
{
    pub center: XExpr,
    pub axis:   XExpr,
    pub speed:  XExpr,
}

impl Default for XSetVelocityCircleModifier
{
    fn default() -> Self
    {
        Self {
            center: XExpr::lit(0.0),
            axis:   XExpr::lit(0.0),
            speed:  XExpr::lit(1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XSetVelocitySphereModifier
{
    pub center: XExpr,
    pub speed:  XExpr,
}

impl Default for XSetVelocitySphereModifier
{
    fn default() -> Self
    {
        Self {
            center: XExpr::lit(0.0),
            speed:  XExpr::lit(1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XSetVelocityTangentModifier
{
    pub center: XExpr,
    pub speed:  XExpr,
}

impl Default for XSetVelocityTangentModifier
{
    fn default() -> Self
    {
        Self {
            center: XExpr::lit(0.0),
            speed:  XExpr::lit(1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XSetAttributeModifier
{
    pub attr:  XAttr,
    pub value: XExpr,
}

impl Default for XSetAttributeModifier
{
    fn default() -> Self
    {
        Self {
            attr:  XAttr::default(),
            value: XExpr::lit(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter, Display, EnumString)]
pub enum XInitModifier
{
    #[strum(serialize = "Set Position Circle")]
    XSetPositionCircle(XSetPositionCircleModifier),
    #[strum(serialize = "Set Position Sphere")]
    XSetPositionSphere(XSetPositionSphereModifier),
    #[strum(serialize = "Set Position Cone 3D")]
    XSetPositionCone3d(XSetPositionCone3dModifier),
    #[strum(serialize = "Set Velocity Circle")]
    XSetVelocityCircle(XSetVelocityCircleModifier),
    #[strum(serialize = "Set Velocity Sphere")]
    XSetVelocitySphere(XSetVelocitySphereModifier),
    #[strum(serialize = "Set Velocity Tangent")]
    XSetVelocityTangent(XSetVelocityTangentModifier),
    #[strum(serialize = "Set Attribute")]
    XSetAttribute(XSetAttributeModifier),
}

impl Default for XInitModifier
{
    fn default() -> Self
    {
        Self::XSetPositionCircle(XSetPositionCircleModifier::default())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XAccelModifier
{
    pub accel: XExpr,
}

impl Default for XAccelModifier
{
    fn default() -> Self
    {
        Self {
            accel: XExpr::lit(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XRadialAccelModifier
{
    pub origin: XExpr,
    pub accel:  XExpr,
}

impl Default for XRadialAccelModifier
{
    fn default() -> Self
    {
        Self {
            origin: XExpr::lit(0.0),
            accel:  XExpr::lit(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XTangentAccelModifier
{
    pub origin: XExpr,
    pub axis:   XExpr,
    pub accel:  XExpr,
}

impl Default for XTangentAccelModifier
{
    fn default() -> Self
    {
        Self {
            origin: XExpr::lit(0.0),
            axis:   XExpr::lit(0.0),
            accel:  XExpr::lit(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XLinearDragModifier
{
    pub drag: XExpr,
}

impl Default for XLinearDragModifier
{
    fn default() -> Self
    {
        Self {
            drag: XExpr::lit(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XForceFieldSource
{
    pub position:          XExpr,
    pub max_radius:        XExpr,
    pub min_radius:        XExpr,
    pub mass:              XExpr,
    pub force_exponent:    XExpr,
    pub conform_to_sphere: bool,
}

impl Default for XForceFieldSource
{
    fn default() -> Self
    {
        Self {
            position:          XExpr::lit(0.0),
            max_radius:        XExpr::lit(10.0),
            min_radius:        XExpr::lit(0.0),
            mass:              XExpr::lit(1.0),
            force_exponent:    XExpr::lit(2.0),
            conform_to_sphere: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XForceFieldModifier
{
    pub sources: Vec<XForceFieldSource>,
}

impl Default for XForceFieldModifier
{
    fn default() -> Self
    {
        Self {
            sources: vec![XForceFieldSource::default()],
        }
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter, Display, EnumString)]
pub enum XUpdateModifier
{
    #[strum(serialize = "Acceleration")]
    XAccel(XAccelModifier),
    #[strum(serialize = "Radial Acceleration")]
    XRadialAccel(XRadialAccelModifier),
    #[strum(serialize = "Tangent Acceleration")]
    XTangentAccel(XTangentAccelModifier),
    #[strum(serialize = "Linear Drag")]
    XLinearDrag(XLinearDragModifier),
    #[strum(serialize = "Set Attribute")]
    XSetAttribute(XSetAttributeModifier),
}

impl Default for XUpdateModifier
{
    fn default() -> Self
    {
        Self::XLinearDrag(XLinearDragModifier::default())
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter, Display, EnumString)]
pub enum XOrientMode
{
    #[strum(serialize = "Parallel Camera Depth Plane")]
    ParallelCameraDepthPlane,
    #[strum(serialize = "Face Camera Position")]
    FaceCameraPosition,
    #[strum(serialize = "Along Velocity")]
    AlongVelocity,
}

impl Default for XOrientMode
{
    fn default() -> Self
    {
        Self::ParallelCameraDepthPlane
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct XOrientModifier
{
    pub mode: XOrientMode,
}

impl Default for XOrientModifier
{
    fn default() -> Self
    {
        Self {
            mode: XOrientMode::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter, Display, EnumString)]
pub enum XRenderModifier
{
    #[strum(serialize = "Orient")]
    XOrient(XOrientModifier),
}

impl Default for XRenderModifier
{
    fn default() -> Self
    {
        Self::XOrient(XOrientModifier::default())
    }
}
