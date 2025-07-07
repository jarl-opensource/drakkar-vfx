pub mod boolean;
pub mod color;
pub mod enumeration;
pub mod events;
pub mod expr;
pub mod float;
pub mod force_field;
pub mod init_modifier;
pub mod integer;
pub mod key_value;
pub mod render_modifier;
pub mod slider;
pub mod spawner;
pub mod text;
pub mod time_color;
pub mod time_vec2;
pub mod update_modifier;

// ====================
// GPUI.
// ====================
use gpui::{Context, Render};

// ====================
// Events.
// ====================

#[derive(Debug, Clone)]
pub enum FacetEvent<V: Clone + std::fmt::Debug + Default>
{
    /// The facet's value has changed.
    Updated
    {
        v: V
    },
}

// ====================
// Traits.
// ====================

/// Trait for composite facets which are built from primitives and combined into facets.
///
pub trait Facet: Sized + 'static + Render
where
    Self::Value: Clone + std::fmt::Debug + Default,
{
    type Value;

    /// Create a new facet with an initial value.
    ///
    fn new(cx: &mut Context<Self>, initial: Self::Value) -> Self;

    /// Get the current value from the facet field.
    ///
    fn get_value<T>(&self, cx: &Context<T>) -> Self::Value;
}
