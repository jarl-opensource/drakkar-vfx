use std::path::PathBuf;

// ====================
// Editor.
// ====================
use crate::gui::file_browser::browser::FileBrowser;
use crate::gui::file_browser::item::{BufferError, FileItem};
use crate::gui::models::state::AssetState;

/// Events sent by FileEntry objects.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEntryEvent
{
    Select
    {
        ix: usize
    },
    MoveUp
    {
        ix: usize
    },
    MoveDown
    {
        ix: usize
    },
    RenameStart
    {
        ix: usize
    },
    RenameConfirm
    {
        ix: usize, new_name: String
    },
    Copy
    {
        ix: usize
    },
    Delete
    {
        ix: usize
    },
}

impl gpui::EventEmitter<FileEntryEvent> for FileItem {}

/// Events sent by FileBrowser objects.
///
#[derive(Debug, Clone)]
pub enum FileBrowserEvent
{
    BufferStateSelected
    {
        path:        PathBuf,
        asset_state: AssetState,
    },
    AssetChanged
    {
        path:        PathBuf,
        asset_state: AssetState,
    },
    BufferLoadError
    {
        path: PathBuf, error: BufferError
    },
}

impl gpui::EventEmitter<FileBrowserEvent> for FileBrowser {}

/// Events sent by StatusBar objects.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusBarEvent
{
    RefreshRequested,
}

impl gpui::EventEmitter<StatusBarEvent> for FileBrowser {}
