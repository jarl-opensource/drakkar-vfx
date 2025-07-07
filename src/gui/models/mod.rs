use strum::{Display, EnumIter, EnumString};

pub mod attr;
pub mod color;
pub mod key_value;
pub mod modifier;
pub mod state;

#[derive(Clone, Debug, PartialEq, EnumIter, Display, EnumString, Default)]
pub enum XDimension
{
    #[default]
    Surface,
    Volume,
}
