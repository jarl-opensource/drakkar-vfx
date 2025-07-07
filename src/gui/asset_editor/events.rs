use gpui::EventEmitter;

use crate::gui::asset_editor::editor::AssetEditor;

/// Event for AssetEditor object.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetEditorEvent
{
    Save,
}

impl EventEmitter<AssetEditorEvent> for AssetEditor {}
