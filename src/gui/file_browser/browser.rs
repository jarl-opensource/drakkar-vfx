use std::collections::HashMap;
use std::path::PathBuf;

// ====================
// GPUI.
// ====================
use gpui::prelude::*;
use gpui::{
    BoxShadow,
    ClickEvent,
    Div,
    Entity,
    IntoElement,
    ParentElement,
    Pixels,
    ScrollStrategy,
    SharedString,
    Styled,
    UniformListScrollHandle,
    WeakEntity,
    Window,
    div,
    point,
    px,
    uniform_list,
};
use tracing::error;

// ====================
// Editor.
// ====================
use crate::gui::file_browser::events::{FileBrowserEvent, FileEntryEvent, StatusBarEvent};
use crate::gui::file_browser::item::*;
use crate::gui::file_browser::status_bar::StatusBar;
use crate::gui::primitives::button::*;
use crate::gui::primitives::events::TextInputEvent;
use crate::gui::primitives::text_input::*;
use crate::gui::scm::*;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::styling::icons::ProductIcon;
use crate::gui::utils::fs::*;
use crate::gui::utils::text::*;

/// File browser widget for browsing and managing particle effect models files.
/// Displays a filtered list of .hanabi.ron files with SCM status indicators.
pub struct FileBrowser
{
    /// Complete list of all discovered models files
    pub all_files:      Vec<FileEntryData>,
    /// Currently visible files after applying name filter
    pub filtered_files: Vec<Entity<FileItem>>,

    /// Index of the currently selected file in filtered_files
    pub selected_file: Option<usize>,

    /// Whether the new file creation UI is active
    pub creating_new_file: bool,
    /// Root directory containing VFX assets
    pub vfx_directory:     PathBuf,
    /// Flag to trigger filesystem rescan
    pub needs_refresh:     bool,
    /// Flag to trigger new file creation
    pub needs_new_file:    bool,
    /// Index of item that needs focus on next render
    pub needs_focus:       Option<usize>,
    /// Source control management interface
    pub scm:               Scm,

    /// Text input for new file name entry
    pub new_file_text_input:         Entity<TextInput>,
    /// Text input for filtering displayed files
    pub file_name_filter_text_input: Entity<TextInput>,
    /// Weak reference to self for child components
    pub weak_ref:                    WeakEntity<Self>,
    /// Handle for controlling list scrolling
    pub scroll_handle:               UniformListScrollHandle,
    pub status_bar:                  Entity<StatusBar>,
    /// Bottom bar for additional controls
    pub bottom_bar:                  Entity<Div>,
    /// Timer for periodic SCM status updates
    pub scm_refresh_timer:           Option<std::time::Instant>,
}

impl FileBrowser
{
    const ASSET_FILE_EXTENSION: &'static str = "hanabi.ron";
    const DEFAULT_TEMPLATE: &'static str = include_str!("../../../assets/default.hanabi.ron");

    /// Creates a new file browser instance.
    /// Initializes UI components and sets up event subscriptions.
    pub fn new(cx: &mut Context<Self>, vfx_directory: PathBuf) -> Self
    {
        let new_file_text_input = cx.new(|cx| TextInput::new(cx));
        let file_name_filter_text_input = cx.new(|cx| {
            TextInput::new(cx)
                .with_placeholder(SharedString::from("Search effect models name .."))
                .with_validation_mode(ValidationMode::AllowAll)
                .with_max_length(32)
                .with_size_variant(SizeVariant::Medium)
                .with_border_radius(BorderRadius::Medium)
                .with_padding_x(Pixels(4.0))
                .with_text_color_focused(border_active())
                .with_text_color_unfocused(text_secondary())
                .with_full_width(true)
        });

        cx.subscribe(&file_name_filter_text_input, move |this, _, event, cx| {
            this.on_filter_input_event(event, cx);
        })
        .detach();

        let repository = Scm::new(&vfx_directory, vec![]);
        let is_git_repo = repository.is_detected();
        let scroll_handle = UniformListScrollHandle::new();
        let status_bar = cx.new(|_| StatusBar::new(vfx_directory.clone(), is_git_repo));
        let bottom_bar = cx.new(|_| div());
        let mut browser = Self {
            filtered_files: Vec::new(),
            all_files: Vec::new(),
            selected_file: None,
            creating_new_file: false,
            vfx_directory,
            needs_refresh: false,
            needs_new_file: false,
            needs_focus: None,
            new_file_text_input,
            file_name_filter_text_input,
            scm: repository,
            weak_ref: WeakEntity::new_invalid(),
            scroll_handle,
            status_bar: status_bar.clone(),
            bottom_bar,
            scm_refresh_timer: None,
        };

        cx.subscribe(&status_bar, Self::on_status_bar_event)
            .detach();

        // Initialize file list and SCM statuses
        browser.set_update_file_list(cx, "", true);
        browser.set_update_scm_statuses(cx);

        // Initialize SCM with file list
        browser.initialize_scm(cx);

        browser
    }

    // ====================
    // Event handlers.
    // ====================

    /// Handles events emitted by FileEntry children.
    /// Processes selection, rename, and navigation events.
    pub fn on_file_entry_event(
        this: &mut Self,
        _: Entity<FileItem>,
        ev: &FileEntryEvent,
        cx: &mut Context<Self>,
    )
    {
        match ev {
            FileEntryEvent::Select { ix } => {
                this.set_selected_item(*ix, cx);
                cx.notify();
            }

            FileEntryEvent::RenameStart { .. } => {
                this.selected_file = None;
                cx.notify();
            }

            FileEntryEvent::MoveUp { ix } => {
                if *ix > 0 {
                    this.set_selected_item(ix - 1, cx);
                }
            }

            FileEntryEvent::MoveDown { ix } => {
                if ix + 1 < this.filtered_files.len() {
                    this.set_selected_item(ix + 1, cx);
                }
            }

            FileEntryEvent::RenameConfirm { ix, new_name } => {
                this.set_new_file_name(*ix, new_name.clone(), cx);
            }
            FileEntryEvent::Copy { ix } => {
                this.duplicate_entry(*ix, cx);
            }
            FileEntryEvent::Delete { ix } => {
                this.delete_file(*ix, cx);
            }
        }
    }

    /// Handles events emitted by StatusBar.
    /// Processes refresh requests.
    pub fn on_status_bar_event(
        this: &mut Self,
        _: Entity<StatusBar>,
        ev: &StatusBarEvent,
        cx: &mut Context<Self>,
    )
    {
        match ev {
            StatusBarEvent::RefreshRequested => {
                this.scm.update_repository();
                let is_git_repo = this.scm.is_detected();
                this.status_bar.update(cx, |status_bar, _| {
                    status_bar.update_git_repo(is_git_repo);
                });
                this.set_update_file_list(cx, "", true);
                this.set_update_scm_statuses(cx);
                cx.notify();
            }
        }
    }

    /// Handles text input events from the file name filter.
    /// Updates filtered list on edit, restores focus on cancel.
    pub fn on_filter_input_event(&mut self, ev: &TextInputEvent, cx: &mut Context<Self>)
    {
        match ev {
            TextInputEvent::Edited => {
                let filter = self.file_name_filter_text_input.read(cx).content.clone();
                self.set_update_file_list(cx, &filter[..], false);
            }

            TextInputEvent::Cancelled => {
                if let Some(ix) = self.selected_file {
                    self.set_selected_item(ix, cx);
                }
            }

            _ => (),
        }
    }

    /// Handles click on the "New Effect" button.
    /// Creates a new effect file with default template content.
    pub fn on_new_file_click(
        this: &mut Self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    )
    {
        // Generate a unique filename
        let filename = FsUtil::generate_unique_new_filename(
            "new_effect",
            Self::ASSET_FILE_EXTENSION,
            &this.vfx_directory,
        );
        let file_path = this.vfx_directory.join(&filename);

        // Create the file with default template content
        if FsUtil::create_file_with_content(&file_path, Self::DEFAULT_TEMPLATE) {
            // Refresh the file list to include the new file
            this.set_update_file_list(cx, "", true);

            // Find and select the newly created file
            if let Some(new_file_index) = this
                .filtered_files
                .iter()
                .position(|entry| entry.read(cx).name == filename)
            {
                this.set_selected_item(new_file_index, cx);
            }

            // Refresh SCM statuses after file creation
            this.refresh_scm_statuses(cx);
        } else {
            error!("Failed to create new effect file: {}", file_path.display());
        }
    }

    /// Handles click on the "Revert" button.
    /// Clears history and reverts buffer to the version from disk.
    pub fn on_revert_click(
        this: &mut Self,
        _: &gpui::MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    )
    {
        if let Some(ix) = this.selected_file {
            if let Some(entry) = this.filtered_files.get(ix) {
                entry.update(cx, |item, _| {
                    let _ = item.init_buf_from_disk();
                });

                // Re-emit the select event to refresh the editor
                this.set_selected_item(ix, cx);
            }
        }
    }

    // ====================
    // State manipulation.
    // ====================

    /// Sets the selected file item and updates UI state.
    /// Deselects all other items and schedules focus for the selected item.
    pub fn set_selected_item(&mut self, ix: usize, cx: &mut Context<Self>)
    {
        self.selected_file = Some(ix);
        self.needs_focus = Some(ix);

        let mut selected = None;
        let mut selected_buffer_state = None;
        let mut selected_buffer_error = None;

        for entry in &mut self.filtered_files {
            entry.update(cx, |entry, _cx| {
                entry.flags.is_selected = ix == entry.ix;
                entry.flags.is_renaming = false;
                if entry.flags.is_selected {
                    selected = Some(entry.path.clone());
                    selected_buffer_state = entry.get_buffer_state().cloned();

                    // Check if buffer failed to load
                    if entry.buffer.is_none() {
                        selected_buffer_error = Some(entry.load_buffer().err().clone());
                    }

                    self.scroll_handle
                        .scroll_to_item(ix, ScrollStrategy::Center);
                }
            });
        }

        if let Some(error) = selected_buffer_error {
            if let Some(path) = selected.clone() {
                cx.emit(FileBrowserEvent::BufferLoadError {
                    path,
                    error: error.unwrap(),
                });
            }
        } else if let Some(asset_state) = selected_buffer_state {
            if let Some(path) = selected {
                cx.emit(FileBrowserEvent::BufferStateSelected { path, asset_state });
            }
        }

        cx.notify();
    }

    /// Updates the file list by scanning filesystem and/or applying name filter.
    /// Preserves selection state when possible.
    pub fn set_update_file_list(
        &mut self,
        cx: &mut Context<Self>,
        name_filter: &str,
        check_fs: bool,
    )
    {
        let name_filter_lower = name_filter.to_owned().to_lowercase();
        let mut selected_name: Option<String> = None;

        // Preserve buffers for existing files
        let mut buf_tmp: HashMap<PathBuf, AssetBuffer> = HashMap::new();
        for entry in &self.filtered_files {
            let file_item = entry.read(cx);
            if let Some(buffer) = &file_item.buffer {
                buf_tmp.insert(file_item.path.clone(), buffer.clone());
            }
        }

        if let Some(ix) = self.selected_file {
            if let Some(entry) = self.filtered_files.get(ix) {
                selected_name = Some(entry.read(cx).name.to_string());
            }
        }

        if check_fs {
            self.all_files.clear();
            self.all_files = FsUtil::list_asset_files_in_dir(
                &self.vfx_directory,
                Self::ASSET_FILE_EXTENSION,
                &mut self.scm,
            );
            self.all_files.sort_by(|a, b| a.name.cmp(&b.name));

            // Update SCM with new file list
            self.update_scm_file_list();
        }

        let mut selected = None;
        self.filtered_files = self
            .all_files
            .iter()
            .filter(|f| f.name.to_lowercase().contains(&name_filter_lower))
            .cloned()
            .enumerate()
            .map(|(ix, mut entry)| {
                if Some(&entry.name) == selected_name.as_ref() {
                    entry.flags.is_selected = true;
                    entry.flags.is_renaming = false;
                    selected = Some(ix);
                }
                cx.notify();
                cx.new(|cx| {
                    let mut file_item = FileItem::new(
                        cx,
                        ix,
                        entry.name,
                        entry.path.clone(),
                        entry.flags,
                        self.weak_ref.clone(),
                    );
                    // Restore buffer if available.
                    if let Some(buffer) = buf_tmp.get(&entry.path) {
                        file_item.buffer = Some(buffer.clone());
                    }
                    //
                    file_item
                })
            })
            .collect();

        self.selected_file = selected;
        self.filtered_files.iter().for_each(|entity| {
            cx.subscribe(entity, Self::on_file_entry_event).detach();
        });
        self.init_buffers(cx);
    }

    /// Initialize buffers for all file items by loading their asset states
    fn init_buffers(&mut self, cx: &mut Context<Self>)
    {
        for file_item in &self.filtered_files {
            if file_item.read(cx).buffer.is_some() {
                continue;
            }
            let path = file_item.read(cx).path.clone();
            match file_item.read(cx).load_buffer() {
                Ok(asset_state) => {
                    file_item.update(cx, |item, _| {
                        item.init_buf(asset_state);
                    });
                }
                Err(error) => {
                    tracing::error!("Failed to load buffer: {}", error);
                    cx.emit(FileBrowserEvent::BufferLoadError { path, error });
                }
            }
        }
    }

    /// Renames a file on disk and updates the corresponding entry.
    /// Shows error in logs if rename operation fails.
    pub fn set_new_file_name(&mut self, ix: usize, new_name: String, cx: &mut Context<Self>)
    {
        if let Some(entry) = self.filtered_files.get(ix) {
            let old_path = entry.read(cx).path.clone();
            if let Some(new_path) = FsUtil::rename_file(&old_path, &new_name) {
                // Cache will be updated by the polling thread

                entry.update(cx, |entry, _cx| {
                    entry.name = SharedString::new(new_name.clone());
                    entry.path = new_path.clone();
                });
                if let Some(file_entry) = self.all_files.iter_mut().find(|f| f.path == old_path) {
                    file_entry.name = new_name.clone();
                    file_entry.path = new_path.clone();
                }
                if let Some(new_ix) = self
                    .filtered_files
                    .iter()
                    .position(|entry| entry.read(cx).path == new_path)
                {
                    self.selected_file = Some(new_ix);
                    self.needs_focus = Some(new_ix);
                }
                // Refresh SCM statuses after file rename
                self.refresh_scm_statuses(cx);
            } else {
                error!("Failed to rename file: {}", old_path.display());
            }
        }
    }

    /// Updates SCM status flags for all visible file entries.
    pub fn set_update_scm_statuses(&mut self, cx: &mut Context<Self>)
    {
        for entry in self.filtered_files.iter() {
            entry.update(cx, |entry, _| {
                entry.flags.scm = self.scm.get_scm_status(&entry.path);
            });
        }
    }

    /// Force refresh SCM statuses and update UI
    pub fn refresh_scm_statuses(&mut self, cx: &mut Context<Self>)
    {
        self.set_update_scm_statuses(cx);
        cx.notify();
    }

    /// Initialize SCM with file list and start polling
    pub fn initialize_scm(&mut self, _cx: &mut Context<Self>)
    {
        let file_paths: Vec<PathBuf> = self.all_files.iter().map(|f| f.path.clone()).collect();
        self.scm = Scm::new(&self.vfx_directory, file_paths);
    }

    /// Update SCM file list
    pub fn update_scm_file_list(&mut self)
    {
        let file_paths: Vec<PathBuf> = self.all_files.iter().map(|f| f.path.clone()).collect();
        self.scm.update_tracked_files(file_paths);
    }

    /// Copies a file entry with a new name.
    pub fn duplicate_entry(&mut self, ix: usize, cx: &mut Context<Self>)
    {
        if let Some(entry) = self.filtered_files.get(ix) {
            let original_path = entry.read(cx).path.clone();
            if let Some(new_path) =
                FsUtil::duplicate_file(&original_path, Self::ASSET_FILE_EXTENSION)
            {
                self.set_update_file_list(cx, "", true);
                if let Some(ix) = self
                    .filtered_files
                    .iter()
                    .position(|entry| entry.read(cx).path == new_path)
                {
                    self.set_selected_item(ix, cx);
                }
                // Refresh SCM statuses after file duplication
                self.refresh_scm_statuses(cx);
            } else {
                error!("Failed to duplicate file: {}", original_path.display());
            }
        }
    }

    /// Deletes a file from disk and updates the file list.
    pub fn delete_file(&mut self, ix: usize, cx: &mut Context<Self>)
    {
        if let Some(entry) = self.filtered_files.get(ix) {
            let file_path = entry.read(cx).path.clone();

            if FsUtil::delete_file(&file_path) {
                self.set_update_file_list(cx, "", true);

                // Clear selection if the deleted file was selected
                if let Some(selected_ix) = self.selected_file {
                    if selected_ix == ix {
                        self.selected_file = None;
                    } else if selected_ix > ix {
                        self.selected_file = Some(selected_ix - 1);
                    }
                }

                // Refresh SCM statuses after file deletion
                self.refresh_scm_statuses(cx);
            } else {
                error!("Failed to delete file: {}", file_path.display());
            }
        }
    }

    // ====================
    // Rendering.
    // ====================

    /// Renders the bottom bar for additional controls.
    fn render_bottom_bar(&self, cx: &mut gpui::Context<Self>) -> impl IntoElement
    {
        // Determine selected file and its buffer state
        let (save_enabled, revert_enabled, selected_path) = if let Some(ix) = self.selected_file {
            if let Some(entry) = self.filtered_files.get(ix) {
                let file_item = entry.read(cx);
                let dirty = file_item
                    .buffer
                    .as_ref()
                    .map(|b| b.is_modified)
                    .unwrap_or(false);
                (dirty, dirty, Some(file_item.path.display().to_string()))
            } else {
                (false, false, None)
            }
        } else {
            (false, false, None)
        };

        div()
            .h(px(48.0))
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .bg(panel_toolbar())
            .border_t_1()
            .border_color(border_separator())
            .shadow(vec![
                BoxShadow {
                    color:         shadow_light(),
                    offset:        point(px(0.), px(-1.)),
                    blur_radius:   px(4.),
                    spread_radius: px(0.),
                },
                BoxShadow {
                    color:         shadow_medium(),
                    offset:        point(px(0.), px(-2.)),
                    blur_radius:   px(8.),
                    spread_radius: px(0.),
                },
            ])
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        with_default_font(div())
                            .text_xs()
                            .text_color(text_muted())
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("File:"),
                    )
                    .child(
                        with_default_font(div())
                            .text_xs()
                            .text_color(text_secondary())
                            .font_weight(gpui::FontWeight::NORMAL)
                            .child(selected_path.unwrap_or_else(|| "No file selected".to_string())),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .when(revert_enabled, |el| {
                                el.child(
                                    div()
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(|this: &mut Self, _ev, _window, cx| {
                                                Self::on_revert_click(this, _ev, _window, cx);
                                            }),
                                        )
                                        .h(px(32.0))
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .bg(if revert_enabled {
                                            revert_button_bg()
                                        } else {
                                            revert_button_bg_disabled()
                                        })
                                        .border_1()
                                        .border_color(if revert_enabled {
                                            revert_button_border()
                                        } else {
                                            revert_button_border_disabled()
                                        })
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .cursor_pointer()
                                        .opacity(if revert_enabled { 1.0 } else { 0.4 })
                                        .hover(|el| {
                                            if revert_enabled {
                                                el.bg(revert_button_bg_hover())
                                                    .border_color(revert_button_border_hover())
                                            } else {
                                                el
                                            }
                                        })
                                        .child(ProductIcon::History.to_svg().text_color(
                                            if revert_enabled {
                                                revert_button_icon()
                                            } else {
                                                revert_button_icon_disabled()
                                            },
                                        ))
                                        .child(
                                            with_default_font(div())
                                                .text_sm()
                                                .text_color(if revert_enabled {
                                                    revert_button_text()
                                                } else {
                                                    revert_button_text_disabled()
                                                })
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .child("Revert"),
                                        ),
                                )
                            })
                            .when(!revert_enabled, |el| {
                                el.child(
                                    div()
                                        .bg(surface_muted())
                                        .border_1()
                                        .border_color(border_subtle())
                                        .rounded_md()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .gap_2()
                                        .px_4()
                                        .py_2()
                                        .opacity(0.5)
                                        .child(
                                            ProductIcon::History
                                                .to_svg()
                                                .text_color(save_button_icon_disabled()),
                                        )
                                        .child(
                                            with_default_font(div())
                                                .text_sm()
                                                .text_color(save_button_text_disabled())
                                                .font_weight(gpui::FontWeight::NORMAL)
                                                .child("Revert"),
                                        ),
                                )
                            }),
                    )
                    .child(
                        div()
                            .when(save_enabled, |el| {
                                el.child(
                                    div()
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(|this: &mut Self, _ev, _window, cx| {
                                                if let Some(ix) = this.selected_file {
                                                    if let Some(entry) = this.filtered_files.get(ix)
                                                    {
                                                        entry.update(cx, |item, _| {
                                                            let _ = item.persist();
                                                        });
                                                    }
                                                    cx.notify();
                                                }
                                            }),
                                        )
                                        .h(px(32.0))
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .bg(if save_enabled {
                                            save_button_bg()
                                        } else {
                                            save_button_bg_disabled()
                                        })
                                        .border_1()
                                        .border_color(if save_enabled {
                                            save_button_border()
                                        } else {
                                            save_button_border_disabled()
                                        })
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .cursor_pointer()
                                        .opacity(if save_enabled { 1.0 } else { 0.4 })
                                        .hover(|el| {
                                            if save_enabled {
                                                el.bg(save_button_bg_hover())
                                                    .border_color(save_button_border_hover())
                                            } else {
                                                el
                                            }
                                        })
                                        .child(ProductIcon::Save.to_svg().text_color(
                                            if save_enabled {
                                                save_button_icon()
                                            } else {
                                                save_button_icon_disabled()
                                            },
                                        ))
                                        .child(
                                            with_default_font(div())
                                                .text_sm()
                                                .text_color(if save_enabled {
                                                    save_button_text()
                                                } else {
                                                    save_button_text_disabled()
                                                })
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .child("Save"),
                                        ),
                                )
                            })
                            .when(!save_enabled, |el| {
                                el.child(
                                    div()
                                        .bg(surface_muted())
                                        .border_1()
                                        .border_color(border_subtle())
                                        .rounded_md()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .gap_2()
                                        .px_4()
                                        .py_2()
                                        .opacity(0.5)
                                        .child(
                                            ProductIcon::Save
                                                .to_svg()
                                                .text_color(save_button_icon_disabled()),
                                        )
                                        .child(
                                            with_default_font(div())
                                                .text_sm()
                                                .text_color(save_button_text_disabled())
                                                .font_weight(gpui::FontWeight::NORMAL)
                                                .child("Save"),
                                        ),
                                )
                            }),
                    ),
            )
    }

    /// Renders the toolbar containing new file button and search input.
    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement
    {
        let file_name_filter_text_input = self.file_name_filter_text_input.clone();

        div()
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_3()
            .bg(panel_toolbar())
            .border_b_2()
            .border_color(border_default())
            .shadow(vec![BoxShadow {
                color:         shadow_light(),
                offset:        point(px(0.), px(1.)),
                blur_radius:   px(2.),
                spread_radius: px(0.),
            }])
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .w_full()
                    .child(
                        div()
                            .id("button_new")
                            .on_click(cx.listener(Self::on_new_file_click))
                            .child(render_toolbar_button_with_icon(
                                "New Effect",
                                Some(ProductIcon::CirclePlus),
                            )),
                    )
                    .child(div().w(px(4.)).h_full().bg(border_separator()).opacity(0.8))
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .bg(surface_elevated())
                            .border_t_1()
                            .rounded_lg()
                            .border_color(border_separator())
                            .shadow(vec![BoxShadow {
                                color:         shadow_light(),
                                offset:        point(px(0.), px(-1.)),
                                blur_radius:   px(2.),
                                spread_radius: px(0.),
                            }])
                            .child(file_name_filter_text_input.clone()),
                    ),
            )
    }

    /// Check and update SCM statuses periodically
    pub fn check_scm_refresh(&mut self, cx: &mut Context<Self>)
    {
        let now = std::time::Instant::now();
        let should_refresh = match self.scm_refresh_timer {
            Some(last_refresh) => now.duration_since(last_refresh).as_secs() >= 5, // Refresh every 5 seconds
            None => true,                                                          // First time
        };

        if should_refresh {
            self.refresh_scm_statuses(cx);
            self.scm_refresh_timer = Some(now);
        }
    }
}

impl Render for FileBrowser
{
    /// Renders the complete file browser UI.
    /// Handles focus restoration and displays file list or empty state.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        // Check for periodic SCM status updates
        self.check_scm_refresh(cx);

        if let Some(ix) = self.needs_focus.take() {
            if let Some(entry) = self.filtered_files.get(ix) {
                entry.update(cx, |this, _| {
                    this.flags.is_selected = true;
                    this.flags.is_renaming = false;
                    window.focus(&this.focus_handle);
                });
            }
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(panel_file_browser())
            .child(self.render_toolbar(cx))
            .child(
                div().flex().flex_col().h_full().child(
                    div()
                        .flex()
                        .flex_col()
                        .h_full()
                        .when(!self.filtered_files.is_empty(), |el| {
                            el.child(
                                uniform_list(
                                    "file_list",
                                    self.filtered_files.len(),
                                    cx.processor(move |this, range, _, _| {
                                        let mut items: Vec<Div> = Vec::new();
                                        for ix in range {
                                            let ix = ix as usize;
                                            if let Some(entry) = this.filtered_files.get(ix) {
                                                items.push(div().child(entry.clone()));
                                            }
                                        }
                                        items
                                    }),
                                )
                                .h_full()
                                .track_scroll(self.scroll_handle.clone()),
                            )
                        })
                        .when(self.filtered_files.is_empty(), |el| {
                            el.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .h_full()
                                    .px_6()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .gap_3()
                                            .child(
                                                div()
                                                    .w(px(60.0))
                                                    .h(px(60.0))
                                                    .rounded_full()
                                                    .bg(surface_elevated())
                                                    .border_1()
                                                    .border_color(border_subtle())
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .shadow(vec![BoxShadow {
                                                        color: shadow_medium(),
                                                        offset: point(px(0.), px(2.)),
                                                        blur_radius: px(6.),
                                                        spread_radius: px(0.),
                                                    }])
                                                    .child(
                                                        div()
                                                            .text_2xl()
                                                            .text_color(text_secondary())
                                                            .font_weight(gpui::FontWeight::LIGHT)
                                                            .child("📁")
                                                    )
                                            )
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .text_color(text_primary())
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_center()
                                                    .child("No Effect Files Found")
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(text_secondary())
                                                    .text_center()
                                                    .max_w(px(280.0))
                                                    .child("No .hanabi.ron files found in the VFX directory.")
                                            )
                                    )
                            )
                        }),
                )
            )
            .child(self.status_bar.clone())
            .child(self.render_bottom_bar(cx))
    }
}
