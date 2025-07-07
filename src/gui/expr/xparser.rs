use std::fmt;

use crate::gui::expr::xexpr::XExpr;
use crate::gui::expr::xop::{XBinaryOp, XBuiltInOp, XUnaryOp};
use crate::gui::expr::xval::XValue;

#[derive(Debug, Clone, PartialEq)]
pub enum XParseError
{
    UnexpectedToken(String),
    UnexpectedEndOfInput,
    InvalidNumber(String),
    InvalidIdentifier(String),
    UnmatchedParenthesis,
    UnknownFunction(String),
    InvalidVectorLiteral,
}

impl fmt::Display for XParseError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            XParseError::UnexpectedToken(token) => write!(f, "Unexpected token: '{}'", token),
            XParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            XParseError::InvalidNumber(s) => write!(f, "Invalid number: '{}'", s),
            XParseError::InvalidIdentifier(s) => write!(f, "Invalid identifier: '{}'", s),
            XParseError::UnmatchedParenthesis => write!(f, "Unmatched parenthesis"),
            XParseError::UnknownFunction(name) => write!(f, "Unknown function: '{}'", name),
            XParseError::InvalidVectorLiteral => write!(f, "Invalid vector literal"),
        }
    }
}

impl std::error::Error for XParseError {}

// ====================
// Parser Implementation.
// ====================

pub struct Parser
{
    input: Vec<char>,
    pos:   usize,
}

impl Parser
{
    /// Create new parser for input string.
    ///
    pub fn new(input: &str) -> Self
    {
        Self {
            input: input.chars().collect(),
            pos:   0,
        }
    }

    /// Parse complete expression from input.
    ///
    pub fn parse(&mut self) -> Result<XExpr, XParseError>
    {
        self.skip_whitespace();
        let expr = self.parse_expression()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(XParseError::UnexpectedToken(
                self.input[self.pos].to_string(),
            ));
        }
        Ok(expr)
    }

    /// Parse top-level expression.
    ///
    pub fn parse_expression(&mut self) -> Result<XExpr, XParseError>
    {
        self.parse_comparison()
    }

    /// Parse comparison operators (==, !=, <, >, <=, >=).
    ///
    pub fn parse_comparison(&mut self) -> Result<XExpr, XParseError>
    {
        let mut left = self.parse_additive()?;

        loop {
            self.skip_whitespace();
            let op = if self.match_str("<=") {
                XBinaryOp::Lte
            } else if self.match_str(">=") {
                XBinaryOp::Gte
            } else if self.match_str("==") {
                XBinaryOp::Eq
            } else if self.match_str("!=") {
                XBinaryOp::Neq
            } else if self.match_char('<') {
                XBinaryOp::Lt
            } else if self.match_char('>') {
                XBinaryOp::Ge
            } else {
                break;
            };

            self.skip_whitespace();
            let right = self.parse_additive()?;
            left = XExpr::binary(left, op, right);
        }

        Ok(left)
    }

    /// Parse addition and subtraction.
    ///
    pub fn parse_additive(&mut self) -> Result<XExpr, XParseError>
    {
        let mut left = self.parse_multiplicative()?;

        loop {
            self.skip_whitespace();
            let op = if self.match_char('+') {
                XBinaryOp::Add
            } else if self.match_char('-') {
                XBinaryOp::Sub
            } else {
                break;
            };

            self.skip_whitespace();
            let right = self.parse_multiplicative()?;
            left = XExpr::binary(left, op, right);
        }

        Ok(left)
    }

    /// Parse multiplication and division.
    ///
    pub fn parse_multiplicative(&mut self) -> Result<XExpr, XParseError>
    {
        let mut left = self.parse_unary()?;

        loop {
            self.skip_whitespace();
            let op = if self.match_char('*') {
                XBinaryOp::Mul
            } else if self.match_char('/') {
                XBinaryOp::Div
            } else {
                break;
            };

            self.skip_whitespace();
            let right = self.parse_unary()?;
            left = XExpr::binary(left, op, right);
        }

        Ok(left)
    }

    /// Parse unary operators (-, sin, cos, etc).
    ///
    pub fn parse_unary(&mut self) -> Result<XExpr, XParseError>
    {
        self.skip_whitespace();

        if self.match_char('-') {
            self.skip_whitespace();
            let expr = self.parse_unary()?;
            return Ok(XExpr::unary(XUnaryOp::Neg, expr));
        }

        self.parse_postfix()
    }

    /// Parse postfix operators (dot access).
    ///
    pub fn parse_postfix(&mut self) -> Result<XExpr, XParseError>
    {
        let mut expr = self.parse_primary()?;

        loop {
            self.skip_whitespace();
            if self.match_char('.') {
                if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    return Err(XParseError::UnexpectedToken(".".to_string()));
                }
                let field = self.parse_identifier()?;
                expr = XExpr::binary(expr, XBinaryOp::Dot, XExpr::attr(field));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse primary expressions (literals, identifiers, parentheses).
    ///
    pub fn parse_primary(&mut self) -> Result<XExpr, XParseError>
    {
        self.skip_whitespace();

        if self.match_char('(') {
            let expr = self.parse_expression()?;
            self.skip_whitespace();
            if !self.match_char(')') {
                return Err(XParseError::UnmatchedParenthesis);
            }
            return Ok(expr);
        }

        if let Some(num) = self.parse_number()? {
            return Ok(XExpr::lit(num));
        }

        if let Some(ident) = self.parse_identifier_opt() {
            self.skip_whitespace();

            if self.match_char('(') {
                return self.parse_function_call(&ident);
            }

            if let Ok(builtin) = self.parse_builtin(&ident) {
                return Ok(XExpr::builtin(builtin));
            }

            return Ok(XExpr::attr(ident));
        }

        Err(XParseError::UnexpectedEndOfInput)
    }

    /// Parse function call expressions.
    ///
    pub fn parse_function_call(&mut self, name: &str) -> Result<XExpr, XParseError>
    {
        match name {
            "sin" => {
                let arg = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::sin(arg))
            }
            "cos" => {
                let arg = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::cos(arg))
            }
            "abs" => {
                let arg = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::abs(arg))
            }
            "norm" => {
                let arg = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::norm(arg))
            }
            "all" => {
                let arg = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::unary(XUnaryOp::All, arg))
            }
            "any" => {
                let arg = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::unary(XUnaryOp::Any, arg))
            }
            "dot" => {
                let left = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(',') {
                    return Err(XParseError::UnexpectedToken(
                        self.peek().unwrap_or('\0').to_string(),
                    ));
                }
                self.skip_whitespace();
                let right = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::binary(left, XBinaryOp::Dot, right))
            }
            "cross" => {
                let left = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(',') {
                    return Err(XParseError::UnexpectedToken(
                        self.peek().unwrap_or('\0').to_string(),
                    ));
                }
                self.skip_whitespace();
                let right = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::binary(left, XBinaryOp::Cross, right))
            }
            "min" => {
                let left = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(',') {
                    return Err(XParseError::UnexpectedToken(
                        self.peek().unwrap_or('\0').to_string(),
                    ));
                }
                self.skip_whitespace();
                let right = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::binary(left, XBinaryOp::Min, right))
            }
            "max" => {
                let left = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(',') {
                    return Err(XParseError::UnexpectedToken(
                        self.peek().unwrap_or('\0').to_string(),
                    ));
                }
                self.skip_whitespace();
                let right = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::binary(left, XBinaryOp::Max, right))
            }
            "vec2" => {
                let x = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(',') {
                    return Err(XParseError::InvalidVectorLiteral);
                }
                self.skip_whitespace();
                let y = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }

                // Create a binary expression that constructs vec2 from the two expressions
                Ok(XExpr::binary(x, XBinaryOp::Vec2, y))
            }
            "vec3" => {
                let x = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(',') {
                    return Err(XParseError::InvalidVectorLiteral);
                }
                self.skip_whitespace();
                let y = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(',') {
                    return Err(XParseError::InvalidVectorLiteral);
                }
                self.skip_whitespace();
                let z = self.parse_expression()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }

                // Create a binary expression that constructs vec3 from the three expressions
                // We use a nested binary structure: (x, y, z) -> ((x, y), z)
                let xy = XExpr::binary(x, XBinaryOp::Vec2, y);
                Ok(XExpr::binary(xy, XBinaryOp::Vec3, z))
            }
            "attr" => {
                self.skip_whitespace();
                let name = self.parse_string_literal()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::attr(name))
            }
            "prop" => {
                self.skip_whitespace();
                let name = self.parse_string_literal()?;
                self.skip_whitespace();
                if !self.match_char(')') {
                    return Err(XParseError::UnmatchedParenthesis);
                }
                Ok(XExpr::prop(name))
            }
            _ => Err(XParseError::UnknownFunction(name.to_string())),
        }
    }

    /// Parse builtin operator from identifier.
    ///
    pub fn parse_builtin(&self, name: &str) -> Result<XBuiltInOp, XParseError>
    {
        match name {
            "time" => Ok(XBuiltInOp::Time),
            "delta_time" => Ok(XBuiltInOp::DeltaTime),
            "rand" => Ok(XBuiltInOp::Rand),
            "alpha_cutoff" => Ok(XBuiltInOp::AlphaCutoff),
            "particle_id" => Ok(XBuiltInOp::ParticleId),
            _ => Err(XParseError::InvalidIdentifier(name.to_string())),
        }
    }

    /// Parse string literal with or without quotes.
    ///
    pub fn parse_string_literal(&mut self) -> Result<String, XParseError>
    {
        if !self.match_char('"') {
            return self.parse_identifier();
        }

        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(result);
            }
            result.push(ch);
            self.advance();
        }
        Err(XParseError::UnmatchedParenthesis)
    }

    /// Parse numeric literal.
    ///
    pub fn parse_number(&mut self) -> Result<Option<XValue>, XParseError>
    {
        let start = self.pos;
        let mut has_dot = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        if self.pos == start {
            return Ok(None);
        }

        let num_str: String = self.input[start..self.pos].iter().collect();

        if has_dot {
            match num_str.parse::<f32>() {
                Ok(f) => Ok(Some(XValue::Float(f))),
                Err(_) => Err(XParseError::InvalidNumber(num_str)),
            }
        } else {
            match num_str.parse::<i32>() {
                Ok(i) => Ok(Some(XValue::Integer(i))),
                Err(_) => Err(XParseError::InvalidNumber(num_str)),
            }
        }
    }

    /// Parse identifier (required).
    ///
    pub fn parse_identifier(&mut self) -> Result<String, XParseError>
    {
        match self.parse_identifier_opt() {
            Some(ident) => Ok(ident),
            None => Err(XParseError::InvalidIdentifier(String::new())),
        }
    }

    /// Parse identifier (optional).
    ///
    pub fn parse_identifier_opt(&mut self) -> Option<String>
    {
        let start = self.pos;

        if let Some(ch) = self.peek() {
            if !ch.is_alphabetic() && ch != '_' {
                return None;
            }
            self.advance();
        } else {
            return None;
        }

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        Some(self.input[start..self.pos].iter().collect())
    }

    // ====================
    // Character helpers.
    // ====================

    /// Skip whitespace characters.
    ///
    pub fn skip_whitespace(&mut self)
    {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Peek at current character without advancing.
    ///
    pub fn peek(&self) -> Option<char>
    {
        self.input.get(self.pos).copied()
    }

    /// Advance position by one character.
    ///
    pub fn advance(&mut self)
    {
        self.pos += 1;
    }

    /// Match expected character and advance if found.
    ///
    pub fn match_char(&mut self, expected: char) -> bool
    {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Match expected string and advance if found.
    ///
    pub fn match_str(&mut self, expected: &str) -> bool
    {
        let chars: Vec<char> = expected.chars().collect();
        if self.pos + chars.len() > self.input.len() {
            return false;
        }

        for (i, &ch) in chars.iter().enumerate() {
            if self.input[self.pos + i] != ch {
                return false;
            }
        }

        self.pos += chars.len();
        true
    }
}
