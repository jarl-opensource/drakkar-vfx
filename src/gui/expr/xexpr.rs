use std::collections::HashMap;
use std::fmt;

use crate::gui::expr::xparser::Parser;
use crate::gui::expr::{XBinaryOp, XBuiltInOp, XExprReturnType, XParseError, XUnaryOp, XValue};

#[derive(Debug, Clone, PartialEq)]
pub enum XExpr
{
    Lit(XValue),
    Attr(String),
    Prop(String),
    BuiltIn(XBuiltInOp),
    Unary
    {
        op:   XUnaryOp,
        expr: Box<XExpr>,
    },
    Binary
    {
        left:  Box<XExpr>,
        op:    XBinaryOp,
        right: Box<XExpr>,
    },
}

impl XExpr
{
    /// Create literal expression.
    ///
    pub fn lit<V: Into<XValue>>(value: V) -> Self
    {
        XExpr::Lit(value.into())
    }

    /// Create attribute expression.
    ///
    pub fn attr(name: impl Into<String>) -> Self
    {
        XExpr::Attr(name.into())
    }

    /// Create property expression.
    ///
    pub fn prop(name: impl Into<String>) -> Self
    {
        XExpr::Prop(name.into())
    }

    /// Create builtin expression.
    ///
    pub fn builtin(op: XBuiltInOp) -> Self
    {
        XExpr::BuiltIn(op)
    }

    /// Create unary expression.
    ///
    pub fn unary(op: XUnaryOp, expr: impl Into<Box<XExpr>>) -> Self
    {
        XExpr::Unary {
            op,
            expr: expr.into(),
        }
    }

    /// Create binary expression.
    ///
    pub fn binary(left: impl Into<Box<XExpr>>, op: XBinaryOp, right: impl Into<Box<XExpr>>)
    -> Self
    {
        XExpr::Binary {
            left: left.into(),
            op,
            right: right.into(),
        }
    }

    /// Create addition expression.
    ///
    pub fn add(self, other: Self) -> Self
    {
        Self::binary(self, XBinaryOp::Add, other)
    }

    /// Create subtraction expression.
    ///
    pub fn sub(self, other: Self) -> Self
    {
        Self::binary(self, XBinaryOp::Sub, other)
    }

    /// Create multiplication expression.
    ///
    pub fn mul(self, other: Self) -> Self
    {
        Self::binary(self, XBinaryOp::Mul, other)
    }

    /// Create division expression.
    ///
    pub fn div(self, other: Self) -> Self
    {
        Self::binary(self, XBinaryOp::Div, other)
    }

    /// Create sine expression.
    ///
    pub fn sin(expr: impl Into<Box<XExpr>>) -> Self
    {
        Self::unary(XUnaryOp::Sin, expr)
    }

    /// Create cosine expression.
    ///
    pub fn cos(expr: impl Into<Box<XExpr>>) -> Self
    {
        Self::unary(XUnaryOp::Cos, expr)
    }

    /// Create absolute value expression.
    ///
    pub fn abs(expr: impl Into<Box<XExpr>>) -> Self
    {
        Self::unary(XUnaryOp::Abs, expr)
    }

    /// Create normalize expression.
    ///
    pub fn norm(expr: impl Into<Box<XExpr>>) -> Self
    {
        Self::unary(XUnaryOp::Norm, expr)
    }

    /// Parse expression from string.
    ///
    pub fn parse(input: &str) -> Result<Self, XParseError>
    {
        Parser::new(input).parse()
    }

    /// Infer return type of expression.
    ///
    pub fn get_result_type(
        &self,
        attributes: &HashMap<String, XExprReturnType>,
        props: &HashMap<String, XExprReturnType>,
    ) -> Option<XExprReturnType>
    {
        match self {
            XExpr::Lit(value) => Some(match value {
                XValue::Float(_) => XExprReturnType::Float,
                XValue::Integer(_) => XExprReturnType::Integer,
                XValue::Vec2(..) => XExprReturnType::Vec2,
                XValue::Vec3(..) => XExprReturnType::Vec3,
            }),
            XExpr::Attr(name) => attributes.get(name).cloned(),
            XExpr::Prop(name) => props.get(name).cloned(),
            XExpr::BuiltIn(_) => Some(XExprReturnType::Float),
            XExpr::Unary { op, expr } => {
                let inner_type = expr.get_result_type(attributes, props)?;
                match op {
                    XUnaryOp::Abs | XUnaryOp::Sin | XUnaryOp::Cos | XUnaryOp::Neg => {
                        Some(inner_type)
                    }
                    XUnaryOp::Norm => Some(XExprReturnType::Float),
                    XUnaryOp::All | XUnaryOp::Any => match inner_type {
                        XExprReturnType::Vec2 | XExprReturnType::Vec3 => {
                            Some(XExprReturnType::Float)
                        }
                        _ => Some(XExprReturnType::Error),
                    },
                }
            }
            XExpr::Binary { left, op, right } => {
                let left_type = left.get_result_type(attributes, props)?;
                let right_type = right.get_result_type(attributes, props)?;

                match op {
                    XBinaryOp::Add | XBinaryOp::Sub | XBinaryOp::Min | XBinaryOp::Max => {
                        match (left_type, right_type) {
                            (XExprReturnType::Float, XExprReturnType::Float) => {
                                Some(XExprReturnType::Float)
                            }
                            (XExprReturnType::Integer, XExprReturnType::Integer) => {
                                Some(XExprReturnType::Integer)
                            }
                            (XExprReturnType::Vec2, XExprReturnType::Vec2) => {
                                Some(XExprReturnType::Vec2)
                            }
                            (XExprReturnType::Vec3, XExprReturnType::Vec3) => {
                                Some(XExprReturnType::Vec3)
                            }
                            (XExprReturnType::Float, XExprReturnType::Vec2)
                            | (XExprReturnType::Vec2, XExprReturnType::Float) => {
                                Some(XExprReturnType::Vec2)
                            }
                            (XExprReturnType::Float, XExprReturnType::Vec3)
                            | (XExprReturnType::Vec3, XExprReturnType::Float) => {
                                Some(XExprReturnType::Vec3)
                            }
                            (XExprReturnType::Integer, XExprReturnType::Vec2)
                            | (XExprReturnType::Vec2, XExprReturnType::Integer) => {
                                Some(XExprReturnType::Vec2)
                            }
                            (XExprReturnType::Integer, XExprReturnType::Vec3)
                            | (XExprReturnType::Vec3, XExprReturnType::Integer) => {
                                Some(XExprReturnType::Vec3)
                            }
                            _ => Some(XExprReturnType::Error),
                        }
                    }
                    XBinaryOp::Mul | XBinaryOp::Div => match (left_type, right_type) {
                        (XExprReturnType::Float, XExprReturnType::Float) => {
                            Some(XExprReturnType::Float)
                        }
                        (XExprReturnType::Integer, XExprReturnType::Integer) => {
                            Some(XExprReturnType::Integer)
                        }
                        (XExprReturnType::Vec2, XExprReturnType::Vec2) => {
                            Some(XExprReturnType::Vec2)
                        }
                        (XExprReturnType::Vec3, XExprReturnType::Vec3) => {
                            Some(XExprReturnType::Vec3)
                        }
                        (XExprReturnType::Float, XExprReturnType::Vec2)
                        | (XExprReturnType::Vec2, XExprReturnType::Float) => {
                            Some(XExprReturnType::Vec2)
                        }
                        (XExprReturnType::Float, XExprReturnType::Vec3)
                        | (XExprReturnType::Vec3, XExprReturnType::Float) => {
                            Some(XExprReturnType::Vec3)
                        }
                        (XExprReturnType::Integer, XExprReturnType::Vec2)
                        | (XExprReturnType::Vec2, XExprReturnType::Integer) => {
                            Some(XExprReturnType::Vec2)
                        }
                        (XExprReturnType::Integer, XExprReturnType::Vec3)
                        | (XExprReturnType::Vec3, XExprReturnType::Integer) => {
                            Some(XExprReturnType::Vec3)
                        }
                        _ => Some(XExprReturnType::Error),
                    },
                    XBinaryOp::Dot => match (left_type, right_type) {
                        (XExprReturnType::Vec2, XExprReturnType::Vec2)
                        | (XExprReturnType::Vec3, XExprReturnType::Vec3) => {
                            Some(XExprReturnType::Float)
                        }
                        _ => Some(XExprReturnType::Error),
                    },
                    XBinaryOp::Cross => match (left_type, right_type) {
                        (XExprReturnType::Vec3, XExprReturnType::Vec3) => {
                            Some(XExprReturnType::Vec3)
                        }
                        _ => Some(XExprReturnType::Error),
                    },
                    XBinaryOp::Vec2 => match (left_type, right_type) {
                        (XExprReturnType::Float, XExprReturnType::Float) => {
                            Some(XExprReturnType::Vec2)
                        }
                        _ => Some(XExprReturnType::Error),
                    },
                    XBinaryOp::Vec3 => match (left_type, right_type) {
                        (XExprReturnType::Vec2, XExprReturnType::Float) => {
                            Some(XExprReturnType::Vec3)
                        }
                        _ => Some(XExprReturnType::Error),
                    },
                    XBinaryOp::Lt
                    | XBinaryOp::Lte
                    | XBinaryOp::Ge
                    | XBinaryOp::Gte
                    | XBinaryOp::Eq
                    | XBinaryOp::Neq => match (left_type, right_type) {
                        (XExprReturnType::Float, XExprReturnType::Float)
                        | (XExprReturnType::Integer, XExprReturnType::Integer)
                        | (XExprReturnType::Vec2, XExprReturnType::Vec2)
                        | (XExprReturnType::Vec3, XExprReturnType::Vec3) => {
                            Some(XExprReturnType::Float)
                        }
                        _ => Some(XExprReturnType::Error),
                    },
                }
            }
        }
    }

    /// Format expression with optional parentheses control
    fn fmt_with_parens(
        &self,
        f: &mut fmt::Formatter<'_>,
        parent_precedence: Option<u8>,
    ) -> fmt::Result
    {
        match self {
            XExpr::Lit(value) => write!(f, "{}", value),
            XExpr::Attr(name) => write!(f, "attr({})", name),
            XExpr::Prop(name) => write!(f, "prop({})", name),
            XExpr::BuiltIn(op) => write!(f, "{}", op),

            XExpr::Unary { op, expr } => {
                if matches!(op, XUnaryOp::Neg) {
                    write!(f, "-")?;
                    expr.fmt_with_parens(f, Some(100)) // High precedence for unary operators
                } else {
                    write!(f, "{}(", op)?;
                    expr.fmt_with_parens(f, None)?;
                    write!(f, ")")
                }
            }

            XExpr::Binary { left, op, right } => {
                if matches!(
                    op,
                    XBinaryOp::Dot
                        | XBinaryOp::Cross
                        | XBinaryOp::Min
                        | XBinaryOp::Max
                        | XBinaryOp::Vec2
                        | XBinaryOp::Vec3
                ) {
                    write!(f, "{}(", op)?;
                    left.fmt_with_parens(f, None)?;
                    write!(f, ", ")?;
                    right.fmt_with_parens(f, None)?;
                    write!(f, ")")
                } else {
                    let current_precedence = self.get_precedence();
                    let needs_parens = if let Some(parent_prec) = parent_precedence {
                        current_precedence < parent_prec
                    } else {
                        false // No parentheses needed at top level
                    };

                    if needs_parens {
                        write!(f, "(")?;
                    }
                    left.fmt_with_parens(f, Some(current_precedence))?;
                    write!(f, " {} ", op)?;
                    right.fmt_with_parens(f, Some(current_precedence))?;
                    if needs_parens {
                        write!(f, ")")?;
                    }
                    Ok(())
                }
            }
        }
    }

    /// Get the precedence of this expression (higher number = higher precedence)
    fn get_precedence(&self) -> u8
    {
        match self {
            XExpr::Lit(_) | XExpr::Attr(_) | XExpr::Prop(_) | XExpr::BuiltIn(_) => 100, // Highest precedence
            XExpr::Unary { .. } => 90, // High precedence for unary operators
            XExpr::Binary { op, .. } => match op {
                XBinaryOp::Mul | XBinaryOp::Div => 3,
                XBinaryOp::Add | XBinaryOp::Sub => 2,
                XBinaryOp::Lt
                | XBinaryOp::Lte
                | XBinaryOp::Ge
                | XBinaryOp::Gte
                | XBinaryOp::Eq
                | XBinaryOp::Neq => 1,
                XBinaryOp::Dot
                | XBinaryOp::Cross
                | XBinaryOp::Min
                | XBinaryOp::Max
                | XBinaryOp::Vec2
                | XBinaryOp::Vec3 => 100, // Function-like, highest precedence
            },
        }
    }
}

impl fmt::Display for XExpr
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        self.fmt_with_parens(f, None)
    }
}
