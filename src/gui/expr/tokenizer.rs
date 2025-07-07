use crate::gui::expr::xop::{XBinaryOp, XBuiltInOp, XUnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub struct Token
{
    pub kind:  TokenKind,
    pub text:  String,
    pub start: usize,
    pub end:   usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind
{
    // Literals
    Integer,
    Float,
    String,

    Identifier,
    Attribute,
    Property,
    BuiltIn(XBuiltInOp),

    // Keywords/Functions
    Function(FunctionKind),

    // Operators
    UnaryOp(XUnaryOp),
    BinaryOp(XBinaryOp),

    // Punctuation
    LeftParen,
    RightParen,
    Comma,
    Dot,

    // Special
    Whitespace,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind
{
    // Unary
    Sin,
    Cos,
    Abs,
    Norm,
    All,
    Any,

    // Binary
    Dot,
    Cross,
    Min,
    Max,

    // Vector
    Vec2,
    Vec3,

    // Special
    Attr,
    Prop,
}

impl FunctionKind
{
    /// Parse function name from string.
    ///
    pub fn from_str(s: &str) -> Option<Self>
    {
        match s {
            "sin" => Some(FunctionKind::Sin),
            "cos" => Some(FunctionKind::Cos),
            "abs" => Some(FunctionKind::Abs),
            "norm" => Some(FunctionKind::Norm),
            "all" => Some(FunctionKind::All),
            "any" => Some(FunctionKind::Any),
            "dot" => Some(FunctionKind::Dot),
            "cross" => Some(FunctionKind::Cross),
            "min" => Some(FunctionKind::Min),
            "max" => Some(FunctionKind::Max),
            "vec2" => Some(FunctionKind::Vec2),
            "vec3" => Some(FunctionKind::Vec3),
            "attr" => Some(FunctionKind::Attr),
            "prop" => Some(FunctionKind::Prop),
            _ => None,
        }
    }

    /// Get expected parameter count for function.
    ///
    pub fn arity(&self) -> usize
    {
        use FunctionKind::*;
        match self {
            Sin | Cos | Abs | Norm | All | Any => 1,
            Dot | Cross | Min | Max | Vec2 => 2,
            Vec3 => 3,
            Attr | Prop => 1,
        }
    }
}

// ====================
// Main Tokenizer.
// ====================

pub struct Tokenizer
{
    input:  Vec<char>,
    pos:    usize,
    tokens: Vec<Token>,
}

impl Tokenizer
{
    /// Create new tokenizer for input string.
    ///
    pub fn new(input: &str) -> Self
    {
        Self {
            input:  input.chars().collect(),
            pos:    0,
            tokens: Vec::new(),
        }
    }

    /// Tokenize the entire input and return all tokens.
    ///
    pub fn tokenize(mut self) -> Vec<Token>
    {
        while self.pos < self.input.len() {
            self.scan_token();
        }
        self.tokens
    }

    /// Scan and classify next token from input.
    ///
    fn scan_token(&mut self)
    {
        let start = self.pos;
        let ch = self.advance();

        match ch {
            '(' => self.add_token(TokenKind::LeftParen, start),
            ')' => self.add_token(TokenKind::RightParen, start),
            ',' => self.add_token(TokenKind::Comma, start),
            '.' => {
                if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    self.pos = start;
                    self.scan_number();
                } else {
                    self.add_token(TokenKind::Dot, start);
                }
            }
            '+' => self.add_token(TokenKind::BinaryOp(XBinaryOp::Add), start),
            '-' => self.add_token(TokenKind::BinaryOp(XBinaryOp::Sub), start),
            '*' => self.add_token(TokenKind::BinaryOp(XBinaryOp::Mul), start),
            '/' => self.add_token(TokenKind::BinaryOp(XBinaryOp::Div), start),
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BinaryOp(XBinaryOp::Lte), start);
                } else {
                    self.add_token(TokenKind::BinaryOp(XBinaryOp::Lt), start);
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BinaryOp(XBinaryOp::Gte), start);
                } else {
                    self.add_token(TokenKind::BinaryOp(XBinaryOp::Ge), start);
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BinaryOp(XBinaryOp::Eq), start);
                } else {
                    self.add_token_with_text(TokenKind::Error, "=", start);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BinaryOp(XBinaryOp::Neq), start);
                } else {
                    self.add_token_with_text(TokenKind::Error, "!", start);
                }
            }
            '"' => self.scan_string(start),
            _ if ch.is_whitespace() => self.scan_whitespace(start),
            _ if ch.is_ascii_digit() => {
                self.pos = start;
                self.scan_number();
            }
            _ if ch.is_alphabetic() || ch == '_' => {
                self.pos = start;
                self.scan_identifier();
            }
            _ => self.add_token_with_text(TokenKind::Error, &ch.to_string(), start),
        }
    }

    /// Scan continuous whitespace sequence.
    ///
    fn scan_whitespace(&mut self, start: usize)
    {
        while self.peek().map_or(false, |c| c.is_whitespace()) {
            self.advance();
        }
        self.add_token(TokenKind::Whitespace, start);
    }

    /// Scan quoted string literal.
    ///
    fn scan_string(&mut self, start: usize)
    {
        while self.peek() != Some('"') && self.pos < self.input.len() {
            self.advance();
        }

        if self.pos >= self.input.len() {
            self.add_token(TokenKind::Error, start);
        } else {
            self.advance(); // Closing "
            self.add_token(TokenKind::String, start);
        }
    }

    /// Scan numeric literal (integer or float).
    ///
    fn scan_number(&mut self)
    {
        let start = self.pos;
        let mut has_dot = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.'
                && !has_dot
                && self.peek_next().map_or(false, |c| c.is_ascii_digit())
            {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let kind = if has_dot {
            TokenKind::Float
        } else {
            TokenKind::Integer
        };
        self.add_token(kind, start);
    }

    /// Scan identifier or keyword.
    ///
    fn scan_identifier(&mut self)
    {
        let start = self.pos;

        while self
            .peek()
            .map_or(false, |c| c.is_alphanumeric() || c == '_')
        {
            self.advance();
        }

        let text: String = self.input[start..self.pos].iter().collect();

        let kind = if let Some(func) = FunctionKind::from_str(&text) {
            TokenKind::Function(func)
        } else if let Ok(builtin) = text.parse::<XBuiltInOp>() {
            TokenKind::BuiltIn(builtin)
        } else {
            TokenKind::Identifier
        };

        self.add_token(kind, start);
    }

    // ====================
    // Character helpers.
    // ====================

    /// Advance position and return current character.
    ///
    fn advance(&mut self) -> char
    {
        let ch = self.input[self.pos];
        self.pos += 1;
        ch
    }

    /// Peek at current character without advancing.
    ///
    fn peek(&self) -> Option<char>
    {
        self.input.get(self.pos).copied()
    }

    /// Peek at next character without advancing.
    ///
    fn peek_next(&self) -> Option<char>
    {
        self.input.get(self.pos + 1).copied()
    }

    /// Match expected character and advance if found.
    ///
    fn match_char(&mut self, expected: char) -> bool
    {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Add token with text extracted from input.
    ///
    fn add_token(&mut self, kind: TokenKind, start: usize)
    {
        let text: String = self.input[start..self.pos].iter().collect();
        self.add_token_with_text(kind, &text, start);
    }

    /// Add token with provided text.
    ///
    fn add_token_with_text(&mut self, kind: TokenKind, text: &str, start: usize)
    {
        self.tokens.push(Token {
            kind,
            text: text.to_string(),
            start,
            end: self.pos,
        });
    }
}

impl std::str::FromStr for XBuiltInOp
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s {
            "time" => Ok(XBuiltInOp::Time),
            "delta_time" => Ok(XBuiltInOp::DeltaTime),
            "rand" => Ok(XBuiltInOp::Rand),
            "alpha_cutoff" => Ok(XBuiltInOp::AlphaCutoff),
            "particle_id" => Ok(XBuiltInOp::ParticleId),
            _ => Err(()),
        }
    }
}

// ====================
// Syntax Highlighting.
// ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxHighlight
{
    Number,
    String,
    Keyword,
    Function,
    Operator,
    Identifier,
    BuiltIn,
    Punctuation,
    Error,
}

impl Token
{
    /// Get syntax highlight category for token.
    ///
    pub fn syntax_highlight(&self) -> Option<SyntaxHighlight>
    {
        use TokenKind::*;
        match &self.kind {
            Integer | Float => Some(SyntaxHighlight::Number),
            String => Some(SyntaxHighlight::String),
            Function(_) => Some(SyntaxHighlight::Function),
            BuiltIn(_) => Some(SyntaxHighlight::BuiltIn),
            UnaryOp(_) | BinaryOp(_) => Some(SyntaxHighlight::Operator),
            Identifier | Attribute | Property => Some(SyntaxHighlight::Identifier),
            LeftParen | RightParen | Comma | Dot => Some(SyntaxHighlight::Punctuation),
            Error => Some(SyntaxHighlight::Error),
            Whitespace => None,
        }
    }
}

// ====================
// Auto-completion.
// ====================

pub struct CompletionItem
{
    pub label:       String,
    pub kind:        CompletionKind,
    pub detail:      Option<String>,
    pub insert_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind
{
    Function,
    BuiltIn,
    Attribute,
    Property,
    Keyword,
}

/// Get completion suggestions for given prefix.
///
pub fn get_completions(prefix: &str, _cursor_pos: usize) -> Vec<CompletionItem>
{
    let mut completions = Vec::new();
    let functions = [
        ("sin", "sin(1.0)", "Sine function"),
        ("cos", "cos(1.0)", "Cosine function"),
        ("abs", "abs(1.0)", "Absolute value"),
        ("norm", "norm(vec2(1.0, 1.0))", "Normalize vector"),
        (
            "all",
            "all(vec2(1.0, 1.0))",
            "Check if all components are true",
        ),
        (
            "any",
            "any(vec2(1.0, 1.0))",
            "Check if any component is true",
        ),
        ("dot", "dot(vec2(1.0, 1.0), vec2(1.0, 1.0))", "Dot product"),
        (
            "cross",
            "cross(vec3(1.0, 1.0, 1.0), vec3(1.0, 1.0, 1.0))",
            "Cross product",
        ),
        ("min", "min(1.0, 1.0)", "Minimum value"),
        ("max", "max(1.0, 1.0)", "Maximum value"),
        ("vec2", "vec2(1.0, 1.0)", "2D vector constructor"),
        ("vec3", "vec3(1.0, 1.0, 1.0)", "3D vector constructor"),
        ("attr", "attr(\"name\")", "Access particle attribute"),
        ("prop", "prop(\"name\")", "Access effect property"),
    ];

    for (name, insert, detail) in functions {
        if name.starts_with(prefix) {
            completions.push(CompletionItem {
                label:       name.to_string(),
                kind:        CompletionKind::Function,
                detail:      Some(detail.to_string()),
                insert_text: insert.to_string(),
            });
        }
    }

    let builtins = [
        ("time", "Current simulation time"),
        ("delta_time", "Time since last update"),
        ("rand", "Random value [0, 1]"),
        ("alpha_cutoff", "Alpha mask threshold"),
        ("particle_id", "Unique particle identifier"),
    ];

    for (name, detail) in builtins {
        if name.starts_with(prefix) {
            completions.push(CompletionItem {
                label:       name.to_string(),
                kind:        CompletionKind::BuiltIn,
                detail:      Some(detail.to_string()),
                insert_text: name.to_string(),
            });
        }
    }

    let attributes = [
        ("position", "Particle position"),
        ("velocity", "Particle velocity"),
        ("lifetime", "Particle lifetime"),
        ("age", "Particle age"),
        ("color", "Particle color"),
        ("size", "Particle size"),
    ];

    for (name, detail) in attributes {
        if name.starts_with(prefix) {
            completions.push(CompletionItem {
                label:       name.to_string(),
                kind:        CompletionKind::Attribute,
                detail:      Some(detail.to_string()),
                insert_text: name.to_string(),
            });
        }
    }

    completions
}

// ====================
// Tests.
// ====================

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_tokenize_simple()
    {
        let tokens = Tokenizer::new("1 + 2").tokenize();
        assert_eq!(tokens.len(), 5); // 1, space, +, space, 2

        let non_whitespace: Vec<_> = tokens
            .into_iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();

        assert_eq!(non_whitespace.len(), 3);
        assert_eq!(non_whitespace[0].kind, TokenKind::Integer);
        assert_eq!(non_whitespace[1].kind, TokenKind::BinaryOp(XBinaryOp::Add));
        assert_eq!(non_whitespace[2].kind, TokenKind::Integer);
    }

    #[test]
    fn test_tokenize_function()
    {
        let tokens = Tokenizer::new("sin(time)").tokenize();
        let non_whitespace: Vec<_> = tokens
            .into_iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();

        assert_eq!(non_whitespace.len(), 4);
        assert_eq!(
            non_whitespace[0].kind,
            TokenKind::Function(FunctionKind::Sin)
        );
        assert_eq!(non_whitespace[1].kind, TokenKind::LeftParen);
        assert_eq!(non_whitespace[2].kind, TokenKind::BuiltIn(XBuiltInOp::Time));
        assert_eq!(non_whitespace[3].kind, TokenKind::RightParen);
    }

    #[test]
    fn test_tokenize_string()
    {
        let tokens = Tokenizer::new("attr(\"position\")").tokenize();
        let non_whitespace: Vec<_> = tokens
            .into_iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();

        assert_eq!(non_whitespace.len(), 4);
        assert_eq!(
            non_whitespace[0].kind,
            TokenKind::Function(FunctionKind::Attr)
        );
        assert_eq!(non_whitespace[1].kind, TokenKind::LeftParen);
        assert_eq!(non_whitespace[2].kind, TokenKind::String);
        assert_eq!(non_whitespace[2].text, "\"position\"");
        assert_eq!(non_whitespace[3].kind, TokenKind::RightParen);
    }

    #[test]
    fn test_completions()
    {
        let completions = get_completions("si", 2);
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.label == "sin"));
        assert!(completions.iter().any(|c| c.label == "size"));

        let completions = get_completions("vec", 3);
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.label == "vec2"));
        assert!(completions.iter().any(|c| c.label == "vec3"));
    }
}
