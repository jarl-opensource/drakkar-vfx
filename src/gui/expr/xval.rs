use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum XValue
{
    Float(f32),
    Integer(i32),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
}

impl XValue
{
    /// Create a float value.
    ///
    pub fn float(v: f32) -> Self
    {
        XValue::Float(v)
    }

    /// Create an integer value.
    ///
    pub fn int(v: i32) -> Self
    {
        XValue::Integer(v)
    }

    /// Create a 2D vector value.
    ///
    pub fn vec2(x: f32, y: f32) -> Self
    {
        XValue::Vec2(x, y)
    }

    /// Create a 3D vector value.
    ///
    pub fn vec3(x: f32, y: f32, z: f32) -> Self
    {
        XValue::Vec3(x, y, z)
    }
}

impl fmt::Display for XValue
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            XValue::Float(v) => {
                if v.fract() == 0.0 {
                    write!(f, "{:.1}", v)
                } else {
                    write!(f, "{}", v)
                }
            }
            XValue::Integer(v) => write!(f, "{}", v),
            XValue::Vec2(x, y) => {
                let x_str = if x.fract() == 0.0 {
                    format!("{:.1}", x)
                } else {
                    format!("{}", x)
                };
                let y_str = if y.fract() == 0.0 {
                    format!("{:.1}", y)
                } else {
                    format!("{}", y)
                };
                write!(f, "vec2({}, {})", x_str, y_str)
            }
            XValue::Vec3(x, y, z) => {
                let x_str = if x.fract() == 0.0 {
                    format!("{:.1}", x)
                } else {
                    format!("{}", x)
                };
                let y_str = if y.fract() == 0.0 {
                    format!("{:.1}", y)
                } else {
                    format!("{}", y)
                };
                let z_str = if z.fract() == 0.0 {
                    format!("{:.1}", z)
                } else {
                    format!("{}", z)
                };
                write!(f, "vec3({}, {}, {})", x_str, y_str, z_str)
            }
        }
    }
}

impl From<f32> for XValue
{
    fn from(v: f32) -> Self
    {
        XValue::Float(v)
    }
}

impl From<i32> for XValue
{
    fn from(v: i32) -> Self
    {
        XValue::Integer(v)
    }
}

impl From<(f32, f32)> for XValue
{
    fn from((x, y): (f32, f32)) -> Self
    {
        XValue::Vec2(x, y)
    }
}

impl From<(f32, f32, f32)> for XValue
{
    fn from((x, y, z): (f32, f32, f32)) -> Self
    {
        XValue::Vec3(x, y, z)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum XExprReturnType
{
    Float,
    Integer,
    Vec2,
    Vec3,
    Error,
}

impl fmt::Display for XExprReturnType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            XExprReturnType::Float => write!(f, "float"),
            XExprReturnType::Integer => write!(f, "integer"),
            XExprReturnType::Vec2 => write!(f, "vec2"),
            XExprReturnType::Vec3 => write!(f, "vec3"),
            XExprReturnType::Error => write!(f, "error"),
        }
    }
}
