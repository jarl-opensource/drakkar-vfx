use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use bevy_hanabi::EffectAsset;

use crate::gui::file_browser::item::*;
use crate::gui::scm::Scm;

pub struct FsUtil;

impl FsUtil
{
    pub fn list_asset_files_in_dir(
        directory: &PathBuf,
        filter: &str,
        scm: &mut Scm,
    ) -> Vec<FileEntryData>
    {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(filter) {
                        let path = entry.path();
                        let mut flags = FileItemFlags::regular();
                        flags.scm = scm.get_scm_status(&path);
                        files.push(FileEntryData {
                            name: name.to_string(),
                            path: path.clone(),
                            flags,
                        });
                    }
                }
            }
        }

        files
    }

    /// Get the modification timestamp of a file
    pub fn get_file_timestamp(file_path: &PathBuf) -> Option<SystemTime>
    {
        if let Ok(metadata) = fs::metadata(file_path) {
            if let Ok(modified_time) = metadata.modified() {
                return Some(modified_time);
            }
        }
        None
    }

    /// Check if a file has been modified since a given timestamp
    pub fn is_file_modified_since(file_path: &PathBuf, since: SystemTime) -> bool
    {
        if let Some(current_timestamp) = Self::get_file_timestamp(file_path) {
            return current_timestamp > since;
        }
        false
    }

    /// Read file content as string
    pub fn read_file_content(file_path: &PathBuf) -> Option<String>
    {
        fs::read_to_string(file_path).ok()
    }

    /// Parse file content as EffectAsset
    pub fn parse_effect_asset(content: &str) -> Option<EffectAsset>
    {
        match ron::from_str::<EffectAsset>(content) {
            Ok(asset) => Some(asset),
            Err(error) => {
                eprintln!("RON parsing error: {}", error);
                None
            }
        }
    }

    /// Generate a unique filename for duplication
    /// Returns the new filename without the path
    pub fn generate_unique_filename(
        original_name: &str,
        extension: &str,
        directory: &PathBuf,
    ) -> String
    {
        let base_name = original_name.trim_end_matches(&format!(".{}", extension));
        let mut counter = 1;
        let mut new_name = format!("{}_copy_{}.{}", base_name, counter, extension);
        let mut new_path = directory.join(&new_name);

        while new_path.exists() {
            counter += 1;
            new_name = format!("{}_copy_{}.{}", base_name, counter, extension);
            new_path = directory.join(&new_name);
        }

        new_name
    }

    /// Duplicate a file with a unique name
    /// Returns the new file path on success, None on failure
    pub fn duplicate_file(original_path: &PathBuf, extension: &str) -> Option<PathBuf>
    {
        let original_name = original_path.file_name()?.to_str()?;
        let directory = original_path.parent()?.to_path_buf();

        let new_name = Self::generate_unique_filename(original_name, extension, &directory);
        let new_path = directory.join(&new_name);

        match fs::copy(original_path, &new_path) {
            Ok(_) => Some(new_path),
            Err(_) => None,
        }
    }

    /// Delete a file from the filesystem
    /// Returns true on success, false on failure
    pub fn delete_file(file_path: &PathBuf) -> bool
    {
        fs::remove_file(file_path).is_ok()
    }

    /// Rename a file on the filesystem
    /// Returns the new path on success, None on failure
    pub fn rename_file(old_path: &PathBuf, new_name: &str) -> Option<PathBuf>
    {
        let new_path = old_path.parent()?.join(new_name);

        match fs::rename(old_path, &new_path) {
            Ok(_) => Some(new_path),
            Err(_) => None,
        }
    }

    /// Create a new file with given content
    /// Returns true on success, false on failure
    pub fn create_file_with_content(file_path: &PathBuf, content: &str) -> bool
    {
        // Ensure the parent directory exists
        if let Some(parent) = file_path.parent() {
            if let Err(_) = fs::create_dir_all(parent) {
                return false;
            }
        }

        fs::write(file_path, content).is_ok()
    }

    /// Generate a unique filename for new file creation
    /// Returns the new filename without the path
    pub fn generate_unique_new_filename(
        base_name: &str,
        extension: &str,
        directory: &PathBuf,
    ) -> String
    {
        let mut counter = 1;
        let mut new_name = format!("{}_{}.{}", base_name, counter, extension);
        let mut new_path = directory.join(&new_name);

        while new_path.exists() {
            counter += 1;
            new_name = format!("{}_{}.{}", base_name, counter, extension);
            new_path = directory.join(&new_name);
        }

        new_name
    }
}
