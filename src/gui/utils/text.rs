use gpui::SharedString;

pub struct TextUtil;

#[derive(Clone, Copy)]
pub enum ValidationMode
{
    AllowAll,
    Numeric,
    Integer,
    Float,
    AlphaNumeric,
    Custom(&'static str),
}

impl TextUtil
{
    pub fn validate(mode: ValidationMode, c: char, content: &SharedString) -> bool
    {
        match mode {
            ValidationMode::AllowAll => true,

            ValidationMode::AlphaNumeric => c.is_alphanumeric(),

            ValidationMode::Custom(chars) => (*chars).find(c).is_some(),

            ValidationMode::Numeric => c.is_ascii_digit(),

            ValidationMode::Integer => c.is_ascii_digit() || (c == '-' && content.is_empty()),

            ValidationMode::Float => {
                c.is_ascii_digit()
                    || (c == '.' && !content.is_empty())
                    || (c == '-' && content.is_empty())
            }
        }
    }
}
