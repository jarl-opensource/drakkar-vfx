use anyhow::Result;
use gpui::{AssetSource, SharedString};

pub struct GuiAssets {}

impl AssetSource for GuiAssets
{
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>>
    {
        std::fs::read(path)
            .map(Into::into)
            .map_err(Into::into)
            .map(Some)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>>
    {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| {
                Some(SharedString::from(
                    entry.ok()?.path().to_string_lossy().to_string(),
                ))
            })
            .collect::<Vec<_>>())
    }
}
