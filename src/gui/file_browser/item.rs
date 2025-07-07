use std::collections::VecDeque;
use std::path::PathBuf;

// ====================
// GPUI.
// ====================
use gpui::prelude::*;
use gpui::{
    ClickEvent,
    Context,
    Entity,
    FocusHandle,
    Focusable,
    IntoElement,
    ParentElement,
    Pixels,
    SharedString,
    Styled,
    WeakEntity,
    Window,
    div,
    px,
    rgb,
};

// ====================
// Editor.
// ====================
use crate::gui::file_browser::browser::FileBrowser;
use crate::gui::file_browser::events::FileEntryEvent;
use crate::gui::models::state::{AssetState, FromHanabi, ToHanabi};
use crate::gui::primitives::button::*;
use crate::gui::primitives::events::TextInputEvent;
use crate::gui::primitives::text_input::*;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::*;
use crate::gui::utils::fs::FsUtil;
use crate::gui::utils::scm::*;

// ====================
// Actions.
// ====================

pub mod actions
{
    use gpui::actions;
    actions!(file_entry, [Up, Down, Enter]);
}

use actions::{Down, Enter, Up};

/// Error types for buffer operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum BufferError
{
    #[error("File error for {}: {message}", path.display())]
    File
    {
        path: PathBuf, message: String
    },
    #[error("Parse error for {}: {message}", path.display())]
    Parse
    {
        path: PathBuf, message: String
    },
    #[error("Conversion error for {}: {message}", path.display())]
    Conversion
    {
        path: PathBuf, message: String
    },
}

/// Result type for buffer state operations
pub type BufferStateResult = Result<AssetState, BufferError>;

/// In-memory buffer for asset state with history
#[derive(Clone, Debug)]
pub struct AssetBuffer
{
    /// History of asset states (ring buffer)
    pub history:         VecDeque<AssetState>,
    /// Index of current state in history (relative to front)
    pub current_state_i: usize,
    /// Whether the buffer has unsaved changes
    pub is_modified:     bool,
    /// File timestamp when buffer was last loaded from disk
    pub disk_timestamp:  Option<std::time::SystemTime>,
}

const MAX_HISTORY: usize = 64;

impl AssetBuffer
{
    pub fn new(initial: AssetState) -> Self
    {
        let mut history = VecDeque::with_capacity(MAX_HISTORY);
        history.push_back(initial);
        Self {
            history,
            current_state_i: 0,
            is_modified: false,
            disk_timestamp: None,
        }
    }

    /// Get the current asset state
    pub fn current_state(&self) -> &AssetState
    {
        self.history
            .get(self.current_state_i)
            .expect("current_state_i out of bounds")
    }

    /// Get mutable reference to current asset state
    pub fn current_state_mut(&mut self) -> &mut AssetState
    {
        self.history
            .get_mut(self.current_state_i)
            .expect("current_state_i out of bounds")
    }

    /// Add a new state to the buffer, trimming history if needed
    pub fn add_state(&mut self, new_state: AssetState)
    {
        // Truncate any redo history
        while self.history.len() > self.current_state_i + 1 {
            self.history.pop_back();
        }
        self.history.push_back(new_state);
        self.current_state_i += 1;

        // If over capacity, pop from front and adjust index
        if self.history.len() > MAX_HISTORY {
            self.history.pop_front();
            if self.current_state_i > 0 {
                self.current_state_i -= 1;
            }
        }

        self.is_modified = true;
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool
    {
        self.current_state_i > 0
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool
    {
        self.current_state_i < self.history.len() - 1
    }

    /// Undo to previous state
    pub fn undo(&mut self) -> bool
    {
        if self.can_undo() {
            self.current_state_i -= 1;
            true
        } else {
            false
        }
    }

    /// Redo to next state
    pub fn redo(&mut self) -> bool
    {
        if self.can_redo() {
            self.current_state_i += 1;
            true
        } else {
            false
        }
    }

    /// Mark buffer as saved (not modified)
    pub fn mark_saved(&mut self)
    {
        self.is_modified = false;
    }

    /// Update disk timestamp to current file timestamp
    pub fn update_disk_timestamp(&mut self, file_path: &std::path::Path)
    {
        self.disk_timestamp = FsUtil::get_file_timestamp(&file_path.to_path_buf());
    }

    /// Check if file on disk has been modified since buffer was loaded
    pub fn is_disk_modified(&self, file_path: &std::path::Path) -> bool
    {
        if let Some(buffer_timestamp) = self.disk_timestamp {
            return FsUtil::is_file_modified_since(&file_path.to_path_buf(), buffer_timestamp);
        }
        false
    }
}

/// Entry representing single models in the browser.
///
#[derive(Clone, Debug)]
pub struct FileItem
{
    pub ix:           usize,
    pub name:         SharedString,
    pub path:         PathBuf,
    pub flags:        FileItemFlags,
    pub parent:       WeakEntity<FileBrowser>,
    pub focus_handle: FocusHandle,
    pub text_input:   Entity<TextInput>,
    pub needs_focus:  bool,

    /// In-memory buffer for asset state
    pub buffer: Option<AssetBuffer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileItemFlags
{
    pub is_new:      bool,
    pub is_selected: bool,
    pub is_renaming: bool,
    pub scm:         ScmStatus,
}

impl FileItemFlags
{
    pub fn regular() -> Self
    {
        Self {
            is_new:      false,
            is_selected: false,
            is_renaming: false,
            scm:         ScmStatus::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileEntryData
{
    pub name:  String,
    pub path:  PathBuf,
    pub flags: FileItemFlags,
}

impl FileItem
{
    pub fn new(
        cx: &mut Context<Self>,
        ix: usize,
        name: String,
        path: PathBuf,
        flags: FileItemFlags,
        parent: WeakEntity<FileBrowser>,
    ) -> Self
    {
        let focus_handle = cx.focus_handle();
        let name = SharedString::new(name);
        let name_ref = name.clone();
        let text_input = cx.new(|cx| {
            TextInput::new(cx)
                .with_content(name, cx)
                .with_border_radius(BorderRadius::None)
                .with_border_width(px(0.0))
                .with_border_style(BorderStyle::None)
                .with_full_width(true)
                .with_text_color_focused(text_primary())
                .with_text_color_unfocused(text_secondary())
                .with_color_variant(ColorVariant::Ghost)
                .with_size_variant(SizeVariant::Medium)
                .with_padding(Pixels(0.0), Pixels(0.0))
        });

        cx.subscribe(&text_input, Self::on_text_input_event)
            .detach();

        FileItem {
            ix,
            name: name_ref,
            path,
            flags,
            parent,
            focus_handle,
            text_input,
            needs_focus: false,
            buffer: None,
        }
    }

    /// Load buffer state from disk.
    pub fn load_buffer(&self) -> BufferStateResult
    {
        let content = FsUtil::read_file_content(&self.path).ok_or_else(|| BufferError::File {
            path:    self.path.clone(),
            message: format!("Failed to read file: {}", self.path.display()),
        })?;
        let asset = FsUtil::parse_effect_asset(&content).ok_or_else(|| BufferError::Parse {
            path:    self.path.clone(),
            message: format!(
                "Failed to parse effect asset from file: {}",
                self.path.display()
            ),
        })?;
        crate::gui::models::state::FromHanabi::asset_state(&asset).map_err(|e| {
            BufferError::Conversion {
                path:    self.path.clone(),
                message: e.to_string(),
            }
        })
    }

    /// Initialize buffer with asset state from disk
    pub fn init_buf(&mut self, asset_state: AssetState)
    {
        let mut buf = AssetBuffer::new(asset_state);

        // TODO: make other way to compare disk <> buffer versions.
        buf.update_disk_timestamp(&self.path);
        self.buffer = Some(buf);
    }

    /// Initialize buffer by loading from disk with error handling
    pub fn init_buf_from_disk(&mut self) -> BufferStateResult
    {
        match self.load_buffer() {
            Ok(asset_state) => {
                self.init_buf(asset_state.clone());
                Ok(asset_state)
            }
            Err(error) => Err(error),
        }
    }

    /// Get current buffer state
    pub fn get_buffer_state(&self) -> Option<&AssetState>
    {
        self.buffer.as_ref().map(|b| b.current_state())
    }

    /// Update buffer with new state
    pub fn update_buffer(&mut self, new_state: AssetState)
    {
        if let Some(buffer) = &mut self.buffer {
            buffer.add_state(new_state);
        }
    }

    /// Check if buffer differs from disk version
    pub fn check_disk_diff(&self) -> bool
    {
        if let Some(ref buffer) = self.buffer {
            if buffer.is_disk_modified(&self.path) {
                return true;
            }

            if let Some(content) = FsUtil::read_file_content(&self.path) {
                if let Some(asset) = FsUtil::parse_effect_asset(&content) {
                    if let Ok(disk_state) = FromHanabi::asset_state(&asset) {
                        return buffer.current_state() != &disk_state;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Persist current buffer state to disk and update all necessary flags
    pub fn persist(&mut self) -> Result<(), Box<dyn std::error::Error>>
    {
        if let Some(buffer) = &mut self.buffer {
            let asset = ToHanabi::effect_asset(buffer.current_state())?;
            let content = ron::ser::to_string_pretty(&asset, ron::ser::PrettyConfig::default())?;
            std::fs::write(&self.path, content)?;
            buffer.update_disk_timestamp(&self.path);
            buffer.mark_saved();
            self.flags.is_new = false;
            Ok(())
        } else {
            Err("No buffer to persist".into())
        }
    }

    /// Get the current status indicators for this file item using centralized cache
    pub fn get_status_indicators(&self, scm: &mut Scm) -> (bool, bool, ScmStatus)
    {
        let buffer_modified = self.buffer.as_ref().map(|b| b.is_modified).unwrap_or(false);
        let buffer_differs_disk = self.check_disk_diff();
        let scm_status = scm.get_scm_status(&self.path);
        (
            buffer_modified || buffer_differs_disk,
            scm_status == ScmStatus::Modified,
            scm_status,
        )
    }

    // ====================
    // Event handlers.
    // ====================

    /// Handle keyboard Up event – notify FileBrowser.
    ///
    fn on_up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>)
    {
        if !self.flags.is_renaming {
            cx.emit(FileEntryEvent::MoveUp { ix: self.ix });
        }
    }

    /// Handle keyboard Down event – notify FileBrowser.
    ///
    fn on_down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>)
    {
        if !self.flags.is_renaming {
            cx.emit(FileEntryEvent::MoveDown { ix: self.ix });
        }
    }

    /// Handle keyboard Enter event - enter renaming mode.
    ///
    fn on_enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>)
    {
        Self::set_enter_edit_mode(self, window, cx);
    }

    /// Handle to text input events.
    /// On Enter – accept.
    /// On Esc   – discard.
    ///
    fn on_text_input_event(
        this: &mut Self,
        _: Entity<TextInput>,
        ev: &TextInputEvent,
        cx: &mut Context<Self>,
    )
    {
        match ev {
            TextInputEvent::Confirmed => {
                let new_name = &this.text_input.read(cx).content[..];
                if !new_name.is_empty() {
                    cx.emit(FileEntryEvent::RenameConfirm {
                        ix:       this.ix,
                        new_name: new_name.to_owned(),
                    });
                    this.flags.is_renaming = false;
                }
                this.flags.is_renaming = false;
                this.flags.is_selected = true;
                this.needs_focus = true;
                cx.notify();
            }

            TextInputEvent::Cancelled => {
                this.text_input.update(cx, |input, cx| {
                    input.content = this.name.clone();
                    input.selected_range = 0..0;
                    cx.notify();
                });
                this.flags.is_renaming = false;
                this.flags.is_selected = true;
                this.needs_focus = true;
                cx.notify();
            }

            _ => {}
        }
    }

    /// Handle mouse clicks when not in renaming mode.
    ///
    fn on_mouse_click(this: &mut Self, ev: &ClickEvent, window: &mut Window, cx: &mut Context<Self>)
    {
        if this.flags.is_renaming {
            return;
        }

        if ev.down.click_count == 1 {
            Self::set_select(this, window, cx);
        } else if ev.down.click_count == 2 {
            Self::set_enter_edit_mode(this, window, cx);
        }
    }

    // ====================
    // State manipulation.
    // ====================

    /// Set to edit mode.
    ///
    fn set_enter_edit_mode(this: &mut Self, window: &mut Window, cx: &mut Context<Self>)
    {
        this.flags.is_renaming = true;
        this.flags.is_selected = false;
        this.text_input.update(cx, |this, cx| {
            window.focus(&mut this.focus_handle);
            this.selected_range = 0..this.content.len();
            cx.notify();
        });

        cx.emit(FileEntryEvent::RenameStart { ix: this.ix });
        cx.notify();
    }

    /// Set to select mode.
    ///
    fn set_select(this: &mut Self, window: &mut Window, cx: &mut Context<Self>)
    {
        this.flags.is_renaming = false;
        this.flags.is_selected = true;

        window.focus(&mut this.focus_handle);

        cx.emit(FileEntryEvent::Select { ix: this.ix });
        cx.notify();
    }
}

// ====================
// Rendering.
// ====================

impl Render for FileItem
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        let _flags = self.flags;
        let is_selected = self.flags.is_selected;
        let is_renaming = self.flags.is_renaming;

        if self.needs_focus {
            window.focus(&mut self.focus_handle);
            self.needs_focus = false;
        }

        // Extract values we need before mutable borrow
        let ix = self.ix;
        let focus_handle = self.focus_handle.clone();
        let text_input = self.text_input.clone();
        let name = self.name.clone();

        // Get status indicators using centralized cache
        let parent = self.parent.upgrade().unwrap();
        let (buffer_out_of_sync, disk_uncommitted, scm_status) =
            parent.update(cx, |parent, _| self.get_status_indicators(&mut parent.scm));
        let is_new = self.flags.is_new || scm_status == ScmStatus::New;

        let mut element = if is_selected {
            render_uniform_list_item_selected()
        } else {
            render_uniform_list_item_button_unselected()
        }
        .id(SharedString::from(format!("file_item_{}", ix)))
        .focusable()
        .on_click(cx.listener(Self::on_mouse_click))
        .track_focus(&focus_handle);

        if is_selected && !is_renaming {
            element = element
                .key_context("FileEntry")
                .on_action(cx.listener(Self::on_up))
                .on_action(cx.listener(Self::on_down))
                .on_action(cx.listener(Self::on_enter));
        }

        element.child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            ProductIcon::Asset
                                .to_svg()
                                .size_4()
                                .text_color(icon_default()),
                        )
                        .w_full()
                        .when(is_renaming, |el| {
                            el.child(
                                with_default_font(div())
                                    .w_full()
                                    .child(text_input.clone()),
                            )
                        })
                        .when(!is_renaming, |el| {
                            el.child(
                                with_default_font(div())
                                    .text_sm()
                                    .text_color(if is_selected {
                                        text_primary()
                                    } else {
                                        text_secondary()
                                    })
                                    .child(name.clone()),
                            )
                        }),
                )
                .when(!is_renaming, |el| {
                    el.child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                // Status indicators on the right
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .when(is_new, |el| {
                                        el.child(
                                            with_default_font(div())
                                                .text_xs()
                                                .text_color(rgb(0x4ade80)) // Green for new files
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .child("new")
                                        )
                                    })
                                    .when(buffer_out_of_sync, |el| {
                                        el.child(
                                            with_default_font(div())
                                                .text_xs()
                                                .text_color(rgb(0xff6b35)) // Orange for buffer out of sync
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .child("unsaved")
                                        )
                                    })
                                    .when(disk_uncommitted, |el| {
                                        el.child(
                                            with_default_font(div())
                                                .text_xs()
                                                .text_color(rgb(0xffd93d)) // Yellow for uncommitted changes
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .opacity(0.3)
                                                .child("uncommitted")
                                        )
                                    })
                            )
                            .child(
                                // Action buttons
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .opacity(if is_selected { 1.0 } else { 0.0 })
                                    .hover(|el| {
                                        el.opacity(1.0)
                                    })
                                    .child(
                                        div()
                                            .id("copy-button")
                                            .w(px(20.0))
                                            .h(px(20.0))
                                            .rounded_md()
                                            .bg(surface_elevated())
                                            .border_1()
                                            .border_color(border_subtle())
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .hover(|el| {
                                                el.border_color(border_active())
                                                    .border_1()
                                            })
                                            .on_click(cx.listener(|this: &mut Self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>| {
                                                cx.emit(FileEntryEvent::Copy { ix: this.ix });
                                            }))
                                            .child(
                                                ProductIcon::Copy
                                                    .to_svg()
                                                    .size_3()
                                                    .text_color(text_secondary())
                                            )
                                    )
                                    .child(
                                        div()
                                            .id("delete-button")
                                            .w(px(20.0))
                                            .h(px(20.0))
                                            .rounded_md()
                                            .bg(surface_elevated())
                                            .border_1()
                                            .border_color(border_subtle())
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .hover(|el| {
                                                el.border_color(border_active())
                                                    .border_1()
                                            })
                                            .on_click(cx.listener(|this: &mut Self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>| {
                                                cx.emit(FileEntryEvent::Delete { ix: this.ix });
                                            }))
                                            .child(
                                                ProductIcon::Trash
                                                    .to_svg()
                                                    .size_3()
                                                    .text_color(text_secondary())
                                            )
                                    )
                            )
                    )
                })

        )
    }
}

impl Focusable for FileItem
{
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle
    {
        self.focus_handle.clone()
    }
}
