use std::ops::Range;

// ====================
// GPUI.
// ====================
use gpui::{App, prelude::*};
use gpui::{
    Bounds,
    BoxShadow,
    ClipboardItem,
    Context,
    CursorStyle,
    Div,
    ElementId,
    ElementInputHandler,
    Entity,
    EntityInputHandler,
    FocusHandle,
    Focusable,
    GlobalElementId,
    MouseButton,
    MouseDownEvent,
    MouseMoveEvent,
    MouseUpEvent,
    PaintQuad,
    Pixels,
    Point,
    Rgba,
    ShapedLine,
    SharedString,
    Style,
    TextRun,
    UTF16Selection,
    UnderlineStyle,
    Window,
    div,
    fill,
    point,
    px,
    relative,
    size,
};
// ====================
// Deps.
// ====================
use unicode_segmentation::*;

// ====================
// Editor.
// ====================
use crate::gui::primitives::events::*;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::utils::text::*;

#[derive(Clone, Debug)]
pub struct HighlightSpan
{
    pub start: usize,
    pub end:   usize,
    pub color: Rgba,
}

pub trait Highlighter: 'static
{
    fn highlight(&self, text: &str) -> Vec<HighlightSpan>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoopHighlighter;

impl Highlighter for NoopHighlighter
{
    fn highlight(&self, _text: &str) -> Vec<HighlightSpan>
    {
        vec![]
    }
}

// ====================
// Actions.
// ====================
pub mod actions
{
    use gpui::actions;
    actions!(
        text_input,
        [
            Backspace,
            Delete,
            Left,
            Right,
            SelectLeft,
            SelectRight,
            SelectAll,
            Home,
            End,
            Paste,
            Cut,
            Copy,
            Enter,
            Escape,
        ]
    );
}
pub use actions::*;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeVariant
{
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum ColorVariant
{
    #[default]
    Default,
    Primary,
    Secondary,
    Ghost,
    Subtle,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum BorderStyle
{
    #[default]
    Solid,
    Dashed,
    Dotted,
    None,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum BorderRadius
{
    #[default]
    Medium,
    None,
    Small,
    Large,
    Full,
}

#[derive(Clone, Debug)]
pub struct BorderOpts
{
    pub style:  BorderStyle,
    pub radius: BorderRadius,
    pub width:  Pixels,
}

impl Default for BorderOpts
{
    fn default() -> Self
    {
        Self {
            style:  BorderStyle::Solid,
            radius: BorderRadius::Medium,
            width:  px(1.0),
        }
    }
}

/// Options for configuring text input padding
#[derive(Clone, Debug)]
pub struct PaddingOpts
{
    /// Horizontal padding (X-axis)
    pub x: Option<Pixels>,
    /// Vertical padding (Y-axis)
    pub y: Option<Pixels>,
}

impl Default for PaddingOpts
{
    fn default() -> Self
    {
        Self { x: None, y: None }
    }
}

#[derive(Clone, Debug)]
pub struct Visual
{
    pub color_variant:        ColorVariant,
    pub border:               BorderOpts,
    pub padding:              PaddingOpts,
    pub disabled:             bool,
    pub full_width:           bool,
    pub full_height:          bool,
    pub text_color_focused:   Option<Rgba>,
    pub text_color_unfocused: Option<Rgba>,
}

impl Default for Visual
{
    fn default() -> Self
    {
        Self {
            color_variant:        ColorVariant::Default,
            border:               BorderOpts::default(),
            padding:              PaddingOpts::default(),
            disabled:             false,
            full_width:           false,
            full_height:          false,
            text_color_focused:   None,
            text_color_unfocused: None,
        }
    }
}

pub struct TextInput<H = NoopHighlighter>
where
    H: Highlighter,
{
    pub focus_handle:       FocusHandle,
    pub content:            SharedString,
    pub placeholder:        SharedString,
    pub selected_range:     Range<usize>,
    pub selection_reversed: bool,
    pub marked_range:       Option<Range<usize>>,
    pub last_layout:        Option<ShapedLine>,
    pub last_bounds:        Option<Bounds<Pixels>>,
    pub is_selecting:       bool,
    pub mode:               ValidationMode,
    pub max_length:         Option<usize>,
    pub size_variant:       SizeVariant,
    pub visual:             Visual,
    pub highlighter:        H,
    pub scroll_offset:      Pixels,
}

impl TextInput<NoopHighlighter>
{
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        Self::with_highlighter(NoopHighlighter, cx)
    }
}

impl<H> TextInput<H>
where
    H: Highlighter,
{
    fn create_text_runs(&self, content: &str, text_color: Rgba, window: &mut Window)
    -> Vec<TextRun>
    {
        let style = window.text_style();
        let highlight_spans = self.highlighter.highlight(content);
        let mut runs = Vec::new();
        let mut pos = 0;

        // If no highlights, create runs based on marked range
        if highlight_spans.is_empty() {
            let run = TextRun {
                len:              content.len(),
                font:             style.font(),
                color:            text_color.into(),
                background_color: None,
                underline:        None,
                strikethrough:    None,
            };

            if let Some(marked_range) = self.marked_range.as_ref() {
                if marked_range.start > pos {
                    runs.push(TextRun {
                        len: marked_range.start - pos,
                        ..run.clone()
                    });
                }
                runs.push(TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color:     Some(run.color),
                        thickness: px(1.0),
                        wavy:      false,
                    }),
                    ..run.clone()
                });
                if marked_range.end < content.len() {
                    runs.push(TextRun {
                        len: content.len() - marked_range.end,
                        ..run.clone()
                    });
                }
            } else {
                runs.push(run);
            }
        } else {
            // Create runs from highlight spans
            for span in highlight_spans {
                // Fill gap before this span
                if span.start > pos {
                    runs.push(TextRun {
                        len:              span.start - pos,
                        font:             style.font(),
                        color:            text_color.into(),
                        background_color: None,
                        underline:        None,
                        strikethrough:    None,
                    });
                }

                // Add highlighted span
                let mut run = TextRun {
                    len:              span.end - span.start,
                    font:             style.font(),
                    color:            span.color.into(),
                    background_color: None,
                    underline:        None,
                    strikethrough:    None,
                };

                // Check if this span overlaps with marked range
                if let Some(marked_range) = self.marked_range.as_ref() {
                    if span.start < marked_range.end && span.end > marked_range.start {
                        run.underline = Some(UnderlineStyle {
                            color:     Some(run.color),
                            thickness: px(1.0),
                            wavy:      false,
                        });
                    }
                }

                runs.push(run);
                pos = span.end;
            }

            // Fill remaining content
            if pos < content.len() {
                runs.push(TextRun {
                    len:              content.len() - pos,
                    font:             style.font(),
                    color:            text_color.into(),
                    background_color: None,
                    underline:        None,
                    strikethrough:    None,
                });
            }
        }

        runs.into_iter().filter(|run| run.len > 0).collect()
    }
    pub fn with_highlighter(highlighter: H, cx: &mut Context<Self>) -> Self
    {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            mode: ValidationMode::AllowAll,
            max_length: None,
            size_variant: SizeVariant::default(),
            visual: Visual::default(),
            highlighter,
            scroll_offset: px(0.0),
        }
    }

    // ====================
    // Builder methods.
    // ====================

    pub fn with_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self
    {
        self.placeholder = placeholder.into();
        self
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self
    {
        self.max_length = Some(max_length);
        self
    }

    pub fn with_content(mut self, content: SharedString, cx: &mut Context<Self>) -> Self
    {
        let content_len = content.len();
        self.content = content;
        self.selected_range = content_len..content_len;
        self.set_move_cursor_to(self.get_next_boundary(self.selected_range.end), cx);
        self
    }

    pub fn with_validation_mode(mut self, mode: ValidationMode) -> Self
    {
        self.mode = mode;
        self
    }

    pub fn with_size_variant(mut self, size: SizeVariant) -> Self
    {
        self.size_variant = size;
        self
    }

    pub fn with_border_style(mut self, style: BorderStyle) -> Self
    {
        self.visual.border.style = style;
        self
    }

    pub fn with_border_radius(mut self, radius: BorderRadius) -> Self
    {
        self.visual.border.radius = radius;
        self
    }

    pub fn with_border_width(mut self, width: Pixels) -> Self
    {
        self.visual.border.width = width;
        self
    }

    pub fn with_border(mut self, border: BorderOpts) -> Self
    {
        self.visual.border = border;
        self
    }

    pub fn with_color_variant(mut self, variant: ColorVariant) -> Self
    {
        self.visual.color_variant = variant;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self
    {
        self.visual.disabled = disabled;
        self
    }

    pub fn with_full_width(mut self, full_width: bool) -> Self
    {
        self.visual.full_width = full_width;
        self
    }

    pub fn with_full_height(mut self, full_height: bool) -> Self
    {
        self.visual.full_height = full_height;
        self
    }

    pub fn with_visual(mut self, visual: Visual) -> Self
    {
        self.visual = visual;
        self
    }

    pub fn with_text_color_focused(mut self, color: Rgba) -> Self
    {
        self.visual.text_color_focused = Some(color);
        self
    }

    pub fn with_text_color_unfocused(mut self, color: Rgba) -> Self
    {
        self.visual.text_color_unfocused = Some(color);
        self
    }

    pub fn with_padding_x(mut self, padding_x: Pixels) -> Self
    {
        self.visual.padding.x = Some(padding_x);
        self
    }

    pub fn with_padding_y(mut self, padding_y: Pixels) -> Self
    {
        self.visual.padding.y = Some(padding_y);
        self
    }

    pub fn with_padding(mut self, padding_x: Pixels, padding_y: Pixels) -> Self
    {
        self.visual.padding.x = Some(padding_x);
        self.visual.padding.y = Some(padding_y);
        self
    }

    pub fn with_no_padding(mut self) -> Self
    {
        self.visual.padding.x = Some(px(0.0));
        self.visual.padding.y = Some(px(0.0));
        self
    }

    // ====================
    // Event handlers.
    // ====================

    fn on_left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>)
    {
        if self.selected_range.is_empty() {
            self.set_move_cursor_to(self.get_previous_boundary(self.get_cursor_offset()), cx);
        } else {
            self.set_move_cursor_to(self.selected_range.start, cx)
        }
    }

    fn on_right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>)
    {
        if self.selected_range.is_empty() {
            self.set_move_cursor_to(self.get_next_boundary(self.selected_range.end), cx);
        } else {
            self.set_move_cursor_to(self.selected_range.end, cx)
        }
    }

    fn on_select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>)
    {
        self.set_select_to(self.get_previous_boundary(self.get_cursor_offset()), cx);
    }

    fn on_select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>)
    {
        self.set_select_to(self.get_next_boundary(self.get_cursor_offset()), cx);
    }

    fn on_select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>)
    {
        self.set_move_cursor_to(0, cx);
        self.set_select_to(self.content.len(), cx)
    }

    fn on_home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>)
    {
        self.set_move_cursor_to(0, cx);
    }

    fn on_end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>)
    {
        self.set_move_cursor_to(self.content.len(), cx);
    }

    fn on_backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>)
    {
        if self.selected_range.is_empty() {
            self.set_select_to(self.get_previous_boundary(self.get_cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn on_delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>)
    {
        if self.selected_range.is_empty() {
            self.set_select_to(self.get_next_boundary(self.get_cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn on_enter(&mut self, _: &Enter, _: &mut Window, cx: &mut Context<Self>)
    {
        cx.emit(TextInputEvent::Confirmed);
    }

    fn on_escape(&mut self, _: &Escape, window: &mut Window, cx: &mut Context<Self>)
    {
        window.blur();
        cx.emit(TextInputEvent::Cancelled);
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    )
    {
        self.is_selecting = true;
        let index = self.get_index_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.set_select_to(index, cx);
        } else {
            self.set_move_cursor_to(index, cx)
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>)
    {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>)
    {
        if self.is_selecting {
            let index = self.get_index_for_mouse_position(event.position);
            self.set_select_to(index, cx);
        }
    }

    fn on_paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>)
    {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            let filtered_text: String = text
                .replace('\n', "")
                .chars()
                .filter(|&c| TextUtil::validate(self.mode, c, &self.content))
                .collect();
            self.replace_text_in_range(None, &filtered_text, window, cx);
        }
    }

    fn on_copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>)
    {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                (&self.content[self.selected_range.clone()]).to_string(),
            ));
        }
    }

    fn on_cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>)
    {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                (&self.content[self.selected_range.clone()]).to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    // ====================
    // State manipulation.
    // ====================

    fn set_move_cursor_to(&mut self, offset: usize, cx: &mut Context<Self>)
    {
        self.selected_range = offset..offset;
        self.update_scroll_offset(cx);
        cx.notify()
    }

    fn update_scroll_offset(&mut self, _cx: &mut Context<Self>)
    {
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return;
        };

        let cursor = self.get_cursor_offset();
        let cursor_x = line.x_for_index(cursor);

        // Get padding and border width
        let (padding_x, _) = match self.size_variant {
            SizeVariant::Small => (px(2.0), px(1.0)),
            SizeVariant::Medium => (px(3.0), px(2.0)),
            SizeVariant::Large => (px(4.0), px(3.0)),
        };
        let border_width = self.visual.border.width;
        let text_area_width = bounds.size.width - (padding_x * 2.0) - (border_width * 2.0);

        // Calculate the visible cursor position
        let visible_cursor_x = cursor_x - self.scroll_offset;

        // If cursor is out of view on the right, scroll to show it
        if visible_cursor_x > text_area_width - px(10.0) {
            self.scroll_offset = cursor_x - text_area_width + px(10.0);
        }
        // If cursor is out of view on the left, scroll to show it
        else if visible_cursor_x < px(10.0) {
            self.scroll_offset = cursor_x - px(10.0);
        }

        // Ensure scroll offset doesn't go negative and doesn't scroll beyond content
        let max_scroll = (line.width - text_area_width).max(px(0.0));
        self.scroll_offset = self.scroll_offset.clamp(px(0.0), max_scroll);
    }

    fn set_select_to(&mut self, offset: usize, cx: &mut Context<Self>)
    {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify()
    }

    // ====================
    // Utility functions.
    // ====================

    fn get_index_for_mouse_position(&self, position: Point<Pixels>) -> usize
    {
        if self.content.is_empty() {
            return 0;
        }

        // If we have layout and bounds, use them for precise positioning
        if let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref()) {
            // Get padding and border width to calculate correct text position
            let (padding_x, _) = match self.size_variant {
                SizeVariant::Small => (px(2.0), px(1.0)),
                SizeVariant::Medium => (px(3.0), px(2.0)),
                SizeVariant::Large => (px(4.0), px(3.0)),
            };
            let border_width = self.visual.border.width;

            // Calculate the inner position relative to the text area
            let inner_position =
                position - bounds.origin - point(padding_x + border_width, px(0.0));

            // Apply scroll offset to get the actual text position
            let text_position = inner_position + point(self.scroll_offset, px(0.0));

            // Use the line's index_for_x method to find the closest character position
            return line
                .index_for_x(text_position.x)
                .unwrap_or(self.content.len());
        }

        // Fallback for first click: estimate position based on text length and click position
        // This is a rough approximation but better than always positioning at the end
        let estimated_char_width = px(8.0); // Rough estimate of character width
        let estimated_text_width = self.content.len() as f32 * estimated_char_width.0;

        // Get padding and border width for bounds estimation
        let (padding_x, _) = match self.size_variant {
            SizeVariant::Small => (px(2.0), px(1.0)),
            SizeVariant::Medium => (px(3.0), px(2.0)),
            SizeVariant::Large => (px(4.0), px(3.0)),
        };
        let border_width = self.visual.border.width;

        // Estimate the click position relative to text start
        let click_x = position.x - padding_x - border_width;
        let percentage = (click_x.0 / estimated_text_width).clamp(0.0, 1.0);

        (self.content.len() as f32 * percentage) as usize
    }

    fn get_cursor_offset(&self) -> usize
    {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn get_offset_from_utf16(&self, offset: usize) -> usize
    {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn get_offset_to_utf16(&self, offset: usize) -> usize
    {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn get_range_to_utf16(&self, range: &Range<usize>) -> Range<usize>
    {
        self.get_offset_to_utf16(range.start)..self.get_offset_to_utf16(range.end)
    }

    fn get_range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize>
    {
        self.get_offset_from_utf16(range_utf16.start)..self.get_offset_from_utf16(range_utf16.end)
    }

    fn get_previous_boundary(&self, offset: usize) -> usize
    {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn get_next_boundary(&self, offset: usize) -> usize
    {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }
}

/// Input handler implememntation for TextInput.
///
impl<H> EntityInputHandler for TextInput<H>
where
    H: Highlighter,
{
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String>
    {
        let range = self.get_range_from_utf16(&range_utf16);
        actual_range.replace(self.get_range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection>
    {
        Some(UTF16Selection {
            range:    self.get_range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>>
    {
        self.marked_range
            .as_ref()
            .map(|range| self.get_range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>)
    {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    )
    {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.get_range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let mut filtered_text: String = new_text
            .chars()
            .filter(|&c| TextUtil::validate(self.mode, c, &self.content))
            .collect();

        if let Some(max_len) = self.max_length {
            let current_len = self.content.len() - (range.end - range.start);
            let available_space = max_len.saturating_sub(current_len);
            if filtered_text.len() > available_space {
                filtered_text = filtered_text.chars().take(available_space).collect();
            }
        }

        self.content =
            (self.content[0..range.start].to_owned() + &filtered_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + filtered_text.len()..range.start + filtered_text.len();
        self.marked_range.take();
        cx.emit(TextInputEvent::Edited);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    )
    {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.get_range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let mut filtered_text: String = new_text
            .chars()
            .filter(|&c| TextUtil::validate(self.mode, c, &self.content))
            .collect();

        if let Some(max_len) = self.max_length {
            let current_len = self.content.len() - (range.end - range.start);
            let available_space = max_len.saturating_sub(current_len);
            if filtered_text.len() > available_space {
                filtered_text = filtered_text.chars().take(available_space).collect();
            }
        }

        self.content =
            (self.content[0..range.start].to_owned() + &filtered_text + &self.content[range.end..])
                .into();

        if !filtered_text.is_empty() {
            self.marked_range = Some(range.start..range.start + filtered_text.len());
        } else {
            self.marked_range = None;
        }

        cx.emit(TextInputEvent::Edited);
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.get_range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| {
                range.start + filtered_text.len()..range.start + filtered_text.len()
            });

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>>
    {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.get_range_from_utf16(&range_utf16);

        // Get padding and border width to calculate correct text position
        let (padding_x, padding_y) = match self.size_variant {
            SizeVariant::Small => (px(2.0), px(1.0)),
            SizeVariant::Medium => (px(3.0), px(2.0)),
            SizeVariant::Large => (px(4.0), px(3.0)),
        };
        let border_width = self.visual.border.width;

        Some(Bounds::from_corners(
            point(
                bounds.left() + padding_x + border_width + last_layout.x_for_index(range.start)
                    - self.scroll_offset,
                bounds.top() + padding_y + border_width,
            ),
            point(
                bounds.left() + padding_x + border_width + last_layout.x_for_index(range.end)
                    - self.scroll_offset,
                bounds.bottom() - padding_y - border_width,
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize>
    {
        let last_layout = self.last_layout.as_ref()?;
        let bounds = self.last_bounds.as_ref()?;

        // Get padding and border width to calculate correct text position
        let (padding_x, _) = match self.size_variant {
            SizeVariant::Small => (px(2.0), px(1.0)),
            SizeVariant::Medium => (px(3.0), px(2.0)),
            SizeVariant::Large => (px(4.0), px(3.0)),
        };
        let border_width = self.visual.border.width;

        // Adjust relative_x to account for padding, border, and scroll offset
        let relative_x = point.x - bounds.left() - padding_x - border_width + self.scroll_offset;
        let utf8_index = last_layout.index_for_x(relative_x)?;
        Some(self.get_offset_to_utf16(utf8_index))
    }
}

struct TextElement<H>
where
    H: Highlighter,
{
    input: Entity<TextInput<H>>,
}

struct PrepaintState
{
    line:      Option<ShapedLine>,
    cursor:    Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl<H> IntoElement for TextElement<H>
where
    H: Highlighter,
{
    type Element = Self;

    fn into_element(self) -> Self::Element
    {
        self
    }
}

impl<H> Element for TextElement<H>
where
    H: Highlighter,
{
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId>
    {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>>
    {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState)
    {
        let input = self.input.read(cx);
        let padding_x = input.visual.padding.x.unwrap_or(match input.size_variant {
            SizeVariant::Small => px(2.0),
            SizeVariant::Medium => px(3.0),
            SizeVariant::Large => px(4.0),
        });
        let full_height = input.visual.full_height;

        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = if full_height {
            relative(1.).into()
        } else {
            match input.size_variant {
                SizeVariant::Small => px(20.).into(),
                SizeVariant::Medium => px(24.).into(),
                SizeVariant::Large => px(32.).into(),
            }
        };
        style.padding.left = padding_x.into();
        style.padding.right = padding_x.into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState
    {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.get_cursor_offset();
        let style = window.text_style();

        // Calculate padding based on custom values or size variant
        let padding_x = input.visual.padding.x.unwrap_or(match input.size_variant {
            SizeVariant::Small => px(2.0),
            SizeVariant::Medium => px(3.0),
            SizeVariant::Large => px(4.0),
        });

        let padding_y = input.visual.padding.y.unwrap_or(match input.size_variant {
            SizeVariant::Small => px(1.0),
            SizeVariant::Medium => px(2.0),
            SizeVariant::Large => px(3.0),
        });
        let border_width = input.visual.border.width;

        let text_color = text_primary();
        let runs = input.create_text_runs(&content, text_color, window);

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(content.clone(), font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);

        // Calculate character width for block cursor
        let char_width = if cursor < content.len() {
            let next_cursor_pos = line.x_for_index(cursor + 1);
            next_cursor_pos - cursor_pos
        } else {
            px(8.0) // fallback width for end of line
        };

        let scroll_offset = self.input.read(cx).scroll_offset;
        let is_focused = self.input.read(cx).focus_handle.is_focused(window);

        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                // Only show cursor when focused
                if is_focused {
                    Some(fill(
                        Bounds::new(
                            point(
                                bounds.left() + padding_x + border_width + cursor_pos
                                    - scroll_offset,
                                bounds.top() + padding_y + border_width,
                            ),
                            size(
                                char_width,
                                bounds.size.height - (padding_y * 2.0) - (border_width * 2.0),
                            ),
                        ),
                        accent_blue(),
                    ))
                } else {
                    None
                },
            )
        } else {
            let start_x = line.x_for_index(selected_range.start);
            let end_x = line.x_for_index(selected_range.end);
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + padding_x + border_width + start_x - scroll_offset,
                            bounds.top() + padding_y + border_width,
                        ),
                        point(
                            bounds.left() + padding_x + border_width + end_x - scroll_offset,
                            bounds.bottom() - padding_y - border_width,
                        ),
                    ),
                    selection_active(),
                )),
                None,
            )
        };
        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut gpui::App,
    )
    {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        let input = self.input.read(cx);
        let is_focused = focus_handle.is_focused(window);

        let padding_x = input.visual.padding.x.unwrap_or(match input.size_variant {
            SizeVariant::Small => px(2.0),
            SizeVariant::Medium => px(3.0),
            SizeVariant::Large => px(4.0),
        });

        let padding_y = input.visual.padding.y.unwrap_or(match input.size_variant {
            SizeVariant::Small => px(1.0),
            SizeVariant::Medium => px(2.0),
            SizeVariant::Large => px(3.0),
        });

        let border_width = input.visual.border.width;

        let text_bounds = Bounds::new(
            point(
                bounds.left() + padding_x + border_width - self.input.read(cx).scroll_offset,
                bounds.top() + padding_y + border_width,
            ),
            size(
                bounds.size.width - (padding_x * 2.0) - (border_width * 2.0),
                bounds.size.height - (padding_y * 2.0) - (border_width * 2.0),
            ),
        );

        // Only handle input when focused
        if is_focused {
            let handler = ElementInputHandler::new(bounds, self.input.clone());
            window.handle_input(&focus_handle, handler, cx);
        }

        if let Some(selection_quad) = prepaint.selection.take() {
            window.paint_quad(selection_quad)
        }

        let line = prepaint.line.take().expect("line should be present");

        // Calculate vertical centering offset using font size
        let line_height = window.line_height();
        let vertical_offset = (text_bounds.size.height - line_height).max(px(0.0)) / 2.0;
        let centered_origin = point(text_bounds.origin.x, text_bounds.origin.y + vertical_offset);

        line.paint(centered_origin, text_bounds.size.height, window, cx)
            .expect("line painting should succeed");

        // Paint cursor only when focused
        if is_focused {
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        }

        // Always update layout and bounds for consistent mouse positioning
        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl<H> Render for TextInput<H>
where
    H: Highlighter,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let is_focused = self.focus_handle.is_focused(window);
        let is_disabled = self.visual.disabled;

        let text_size: Box<dyn Fn(Div) -> Div> = match self.size_variant {
            SizeVariant::Small => Box::new(|el| el.text_xs()),
            SizeVariant::Medium => Box::new(|el| el.text_sm()),
            SizeVariant::Large => Box::new(|el| el.text_lg()),
        };

        let padding_x = self.visual.padding.x.unwrap_or(match self.size_variant {
            SizeVariant::Small => px(2.0),
            SizeVariant::Medium => px(3.0),
            SizeVariant::Large => px(4.0),
        });

        let padding_y = self.visual.padding.y.unwrap_or(match self.size_variant {
            SizeVariant::Small => px(1.0),
            SizeVariant::Medium => px(2.0),
            SizeVariant::Large => px(3.0),
        });

        // Store border width to ensure consistent sizing
        let border_width = self.visual.border.width;

        // Border radius
        let border_radius: Box<dyn Fn(Div) -> Div> = match self.visual.border.radius {
            BorderRadius::None => Box::new(|el| el),
            BorderRadius::Small => Box::new(|el| el.rounded_sm()),
            BorderRadius::Medium => Box::new(|el| el.rounded_md()),
            BorderRadius::Large => Box::new(|el| el.rounded_lg()),
            BorderRadius::Full => Box::new(|el| el.rounded_full()),
        };

        // Use custom text colors if provided, otherwise use color variant defaults
        let text_color_for_state = if is_focused {
            self.visual.text_color_focused
        } else {
            self.visual.text_color_unfocused
        };

        // Color variant configurations
        let (bg_color, border_color, text_color_normal, text_color_placeholder) =
            match self.visual.color_variant {
                ColorVariant::Default => (
                    if is_disabled {
                        surface_muted()
                    } else {
                        surface_elevated()
                    },
                    if is_focused {
                        border_focus()
                    } else if is_disabled {
                        border_muted()
                    } else {
                        border_default()
                    },
                    text_color_for_state.unwrap_or(if is_disabled {
                        text_muted()
                    } else {
                        text_default()
                    }),
                    text_muted(),
                ),
                ColorVariant::Primary => (
                    if is_disabled {
                        primary_muted()
                    } else {
                        primary()
                    },
                    if is_focused {
                        primary_emphasis()
                    } else if is_disabled {
                        primary_muted()
                    } else {
                        primary()
                    },
                    text_color_for_state.unwrap_or(text_on_primary()),
                    Rgba {
                        r: text_on_primary().r,
                        g: text_on_primary().g,
                        b: text_on_primary().b,
                        a: 0.6,
                    },
                ),
                ColorVariant::Secondary => (
                    if is_disabled {
                        secondary_muted()
                    } else {
                        surface_elevated()
                    },
                    if is_focused {
                        secondary_emphasis()
                    } else if is_disabled {
                        secondary_muted()
                    } else {
                        secondary()
                    },
                    text_color_for_state.unwrap_or(text_on_secondary()),
                    Rgba {
                        r: text_on_secondary().r,
                        g: text_on_secondary().g,
                        b: text_on_secondary().b,
                        a: 0.6,
                    },
                ),
                ColorVariant::Ghost => (
                    transparent_black(),
                    if is_focused {
                        border_focus()
                    } else {
                        transparent_black()
                    },
                    text_color_for_state.unwrap_or(if is_disabled {
                        text_muted()
                    } else {
                        text_default()
                    }),
                    text_muted(),
                ),
                ColorVariant::Subtle => (
                    Rgba {
                        r: surface_elevated().r,
                        g: surface_elevated().g,
                        b: surface_elevated().b,
                        a: 0.15,
                    },
                    if is_focused {
                        border_focus()
                    } else {
                        Rgba {
                            r: border_default().r,
                            g: border_default().g,
                            b: border_default().b,
                            a: 0.5,
                        }
                    },
                    text_color_for_state.unwrap_or(if is_disabled {
                        text_muted()
                    } else {
                        text_default()
                    }),
                    text_muted(),
                ),
            };

        // Base element
        let mut element = div()
            .px(padding_x)
            .py(padding_y)
            .bg(bg_color)
            .text_color(text_color_normal);

        // Apply width and height
        element = if self.visual.full_width {
            element.w_full()
        } else {
            element
        };

        element = if self.visual.full_height {
            element.h_full()
        } else {
            element
        };

        // Apply border - always apply border width to maintain consistent sizing
        element = if matches!(self.visual.border.style, BorderStyle::None) {
            // For no border, use transparent border to maintain spacing
            element
                .border(border_width)
                .border_color(transparent_black())
        } else {
            element.border(border_width).border_color(border_color)
        };

        element = border_radius(element);
        element = text_size(element);
        element = element.when(
            is_focused
                && !is_disabled
                && !matches!(
                    self.visual.color_variant,
                    ColorVariant::Ghost | ColorVariant::Subtle
                ),
            |el| {
                el.shadow(vec![BoxShadow {
                    color:         shadow_medium(),
                    offset:        point(px(0.), px(0.)),
                    blur_radius:   px(0.),
                    spread_radius: px(0.), // Changed from px(1.) to px(0.) to prevent size change
                }])
            },
        );

        element = element.when(is_disabled, |el| el.cursor(CursorStyle::Arrow).opacity(0.6));
        element = element.when(!is_disabled, |el| el.cursor(CursorStyle::IBeam));
        element = element.when(!is_focused && !is_disabled, |el| {
            el.hover(|hover_el| {
                let hover_border_color = match self.visual.color_variant {
                    ColorVariant::Default => border_active(),
                    ColorVariant::Primary => primary_emphasis(),
                    ColorVariant::Secondary => secondary_emphasis(),
                    ColorVariant::Ghost => border_active(),
                    ColorVariant::Subtle => border_active(),
                };
                hover_el.border_color(hover_border_color)
            })
        });

        with_default_font(
            element
                .key_context("TextInput")
                .track_focus(&self.focus_handle(cx))
                .when(!is_disabled, |el| {
                    el.on_action(cx.listener(Self::on_backspace))
                        .on_action(cx.listener(Self::on_delete))
                        .on_action(cx.listener(Self::on_left))
                        .on_action(cx.listener(Self::on_right))
                        .on_action(cx.listener(Self::on_select_left))
                        .on_action(cx.listener(Self::on_select_right))
                        .on_action(cx.listener(Self::on_select_all))
                        .on_action(cx.listener(Self::on_home))
                        .on_action(cx.listener(Self::on_end))
                        .on_action(cx.listener(Self::on_paste))
                        .on_action(cx.listener(Self::on_cut))
                        .on_action(cx.listener(Self::on_copy))
                        .on_action(cx.listener(Self::on_enter))
                        .on_action(cx.listener(Self::on_escape))
                        .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                        .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
                        .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
                        .on_mouse_move(cx.listener(Self::on_mouse_move))
                })
                .child(
                    div()
                        .overflow_hidden() // Clip text that extends beyond bounds
                        .relative() // Make this relative so we can overlay the placeholder
                        .child(
                            // Always use TextElement for consistent rendering with scrolling and input handling
                            TextElement {
                                input: cx.entity().clone(),
                            }
                            .into_any(),
                        )
                        .when(
                            self.content.is_empty() && !self.placeholder.is_empty(),
                            |el| {
                                // Overlay the placeholder on top when content is empty
                                el.child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .items_center()
                                        .pt(px(1.0))
                                        .pb(px(0.5))
                                        .pl(px(5.0))
                                        .text_color(text_color_placeholder)
                                        .child(self.placeholder.clone()),
                                )
                            },
                        ),
                ),
        )
    }
}

impl<H> Focusable for TextInput<H>
where
    H: Highlighter,
{
    fn focus_handle(&self, _: &App) -> FocusHandle
    {
        self.focus_handle.clone()
    }
}

impl<H> gpui::EventEmitter<TextInputEvent> for TextInput<H> where H: Highlighter {}
