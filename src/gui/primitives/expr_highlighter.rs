// ====================
// Editor.
// ====================
use crate::gui::expr::{SyntaxHighlight, Tokenizer};
use crate::gui::primitives::text_input::{HighlightSpan, Highlighter};
use crate::gui::styling::colors::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct ExprHighlighter
{
    pub matching_parens: Option<(usize, usize)>,
}

impl ExprHighlighter
{
    pub fn new() -> Self
    {
        Self {
            matching_parens: None,
        }
    }

    pub fn with_matching_parens(mut self, parens: Option<(usize, usize)>) -> Self
    {
        self.matching_parens = parens;
        self
    }
}

impl Highlighter for ExprHighlighter
{
    fn highlight(&self, text: &str) -> Vec<HighlightSpan>
    {
        let tokens = Tokenizer::new(text).tokenize();
        let mut spans = Vec::new();

        for token in tokens {
            let color = match token.syntax_highlight() {
                Some(SyntaxHighlight::Number) => syntax_number(),
                Some(SyntaxHighlight::String) => syntax_string(),
                Some(SyntaxHighlight::Function) => syntax_function(),
                Some(SyntaxHighlight::BuiltIn) => syntax_builtin(),
                Some(SyntaxHighlight::Operator) => syntax_operator(),
                Some(SyntaxHighlight::Identifier) => syntax_identifier(),
                Some(SyntaxHighlight::Punctuation) => syntax_punctuation(),
                Some(SyntaxHighlight::Error) => syntax_error(),
                Some(SyntaxHighlight::Keyword) => syntax_keyword(),
                None => continue,
            };

            // Override color for matching parentheses
            let final_color = if let Some((open_pos, close_pos)) = self.matching_parens {
                if (token.kind == crate::gui::expr::TokenKind::LeftParen && token.start == open_pos)
                    || (token.kind == crate::gui::expr::TokenKind::RightParen
                        && token.start == close_pos)
                {
                    expr_valid()
                } else {
                    color
                }
            } else {
                color
            };

            spans.push(HighlightSpan {
                start: token.start,
                end:   token.start + token.text.len(),
                color: final_color,
            });
        }

        spans
    }
}
