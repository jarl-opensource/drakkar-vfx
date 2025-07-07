use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XUnaryOp
{
    Abs,
    Norm,
    Sin,
    Cos,
    Neg,
    All,
    Any,
}

impl fmt::Display for XUnaryOp
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            XUnaryOp::Abs => write!(f, "abs"),
            XUnaryOp::Norm => write!(f, "norm"),
            XUnaryOp::Sin => write!(f, "sin"),
            XUnaryOp::Cos => write!(f, "cos"),
            XUnaryOp::Neg => write!(f, "-"),
            XUnaryOp::All => write!(f, "all"),
            XUnaryOp::Any => write!(f, "any"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XBinaryOp
{
    Add,
    Sub,
    Mul,
    Div,
    Dot,
    Cross,
    Min,
    Max,
    Lt,
    Lte,
    Ge,
    Gte,
    Eq,
    Neq,
    Vec2,
    Vec3,
}

impl fmt::Display for XBinaryOp
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            XBinaryOp::Add => write!(f, "+"),
            XBinaryOp::Sub => write!(f, "-"),
            XBinaryOp::Mul => write!(f, "*"),
            XBinaryOp::Div => write!(f, "/"),
            XBinaryOp::Dot => write!(f, "dot"),
            XBinaryOp::Cross => write!(f, "cross"),
            XBinaryOp::Min => write!(f, "min"),
            XBinaryOp::Max => write!(f, "max"),
            XBinaryOp::Lt => write!(f, "<"),
            XBinaryOp::Lte => write!(f, "<="),
            XBinaryOp::Ge => write!(f, ">"),
            XBinaryOp::Gte => write!(f, ">="),
            XBinaryOp::Eq => write!(f, "=="),
            XBinaryOp::Neq => write!(f, "!="),
            XBinaryOp::Vec2 => write!(f, "vec2"),
            XBinaryOp::Vec3 => write!(f, "vec3"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XBuiltInOp
{
    Time,
    DeltaTime,
    Rand,
    AlphaCutoff,
    ParticleId,
}

impl fmt::Display for XBuiltInOp
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            XBuiltInOp::Time => write!(f, "time"),
            XBuiltInOp::DeltaTime => write!(f, "delta_time"),
            XBuiltInOp::Rand => write!(f, "rand"),
            XBuiltInOp::AlphaCutoff => write!(f, "alpha_cutoff"),
            XBuiltInOp::ParticleId => write!(f, "particle_id"),
        }
    }
}
