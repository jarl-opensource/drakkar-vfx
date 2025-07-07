use std::path::Path;

use gpui::prelude::*;
use gpui::{Context, IntoElement, ParentElement, Styled, Window, div, px, rems};

use crate::gui::styling::colors::*;
use crate::gui::styling::icons::ProductIcon;

/// Error panel for displaying asset parsing/conversion errors.
///
/// Shows error information when an asset file cannot be parsed
/// or converted to the editor's state representation.
pub struct ErrorPanel
{
    file_path:     String,
    error_message: String,
}

impl ErrorPanel
{
    /// Create a new error panel with the given file path and error message.
    pub fn new(file_path: impl AsRef<Path>, error_message: String) -> Self
    {
        Self {
            file_path: file_path.as_ref().display().to_string(),
            error_message,
        }
    }
}

impl Render for ErrorPanel
{
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .id("error-panel")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_start()
            .bg(panel_asset_editor())
            .pt_8()
            .px_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .max_w(px(500.0))
                    .bg(background_darker())
                    .border_1()
                    .border_color(border_subtle())
                    .rounded_lg()
                    .overflow_hidden()
                    .child(
                        // Header
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .px_4()
                            .py_3()
                            .bg(surface_elevated())
                            .child(
                                ProductIcon::OctagonAlert
                                    .to_svg()
                                    .text_color(text_warning()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(text_primary())
                                    .child("Failed to load asset"),
                            ),
                    )
                    .child(
                        // File path
                        div()
                            .px_4()
                            .py_3()
                            .border_b_1()
                            .border_color(border_subtle())
                            .child(div().text_xs().text_color(text_muted()).child("File:"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_secondary())
                                    .child(self.file_path.clone()),
                            ),
                    )
                    .child(
                        // Error message
                        div().px_4().py_4().max_h(px(200.0)).child(
                            div()
                                .text_xs()
                                .font_family("mono")
                                .text_color(text_accent())
                                .line_height(rems(1.4))
                                .child(self.error_message.clone()),
                        ),
                    ),
            )
    }
}
