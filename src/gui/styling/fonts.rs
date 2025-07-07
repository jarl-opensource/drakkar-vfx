use gpui::{FontWeight, SharedString, Styled};

pub const FONT_BYTES: &[u8] = include_bytes!("../../../assets/Roboto.ttf");

pub fn font_bytes() -> &'static [u8]
{
    FONT_BYTES
}

pub fn default_font_family() -> SharedString
{
    SharedString::from("Roboto")
}

pub fn with_default_font<T: Styled>(element: T) -> T
{
    element.font_family(default_font_family())
}

pub fn with_default_font_weight<T: Styled>(element: T, weight: FontWeight) -> T
{
    element
        .font_family(default_font_family())
        .font_weight(weight)
}
