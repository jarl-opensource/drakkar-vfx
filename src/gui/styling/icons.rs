use std::sync::Arc;

use gpui::{Image, ImageSource, Img, Styled, Svg, img, svg};

use crate::gui::styling::colors::icon_default;

/// Consts.
///
mod consts
{
    pub(crate) const ROOT: &str = env!("CARGO_MANIFEST_DIR");

    pub(crate) const ASSET_FILE_ICON_PATH: &str = "assets/file-box.svg";
    pub(crate) const CHEVRON_RIGHT_ICON_PATH: &str = "assets/chevron-right.svg";
    pub(crate) const CHEVRON_DOWN_ICON_PATH: &str = "assets/chevron-down.svg";
    pub(crate) const CHEVRON_UP_ICON_PATH: &str = "assets/chevron-up.svg";
    pub(crate) const CHECK_ICON_PATH: &str = "assets/check.svg";
    pub(crate) const TRASH_ICON_PATH: &str = "assets/trash.svg";
    pub(crate) const CIRCLE_PLUS_ICON_PATH: &str = "assets/circle-plus.svg";
    pub(crate) const SYMBOL_ARRAY_ICON_PATH: &str = "assets/symbol-array.svg";
    pub(crate) const SYMBOL_COLOR_ICON_PATH: &str = "assets/symbol-color.svg";
    pub(crate) const SYMBOL_FIELD_ICON_PATH: &str = "assets/symbol-field.svg";
    pub(crate) const SYMBOL_NUMERIC_ICON_PATH: &str = "assets/symbol-numeric.svg";
    pub(crate) const SYMBOL_STRING_ICON_PATH: &str = "assets/symbol-key.svg";
    pub(crate) const SQUARE_FUNCTION_ICON_PATH: &str = "assets/square-function.svg";
    pub(crate) const RECTANGLE_ELLIPSIS_ICON_PATH: &str = "assets/rectangle-ellipsis.svg";
    pub(crate) const OCTAGON_ALERT_ICON_PATH: &str = "assets/octagon-alert.svg";
    pub(crate) const SYMBOL_EVENT_ICON_PATH: &str = "assets/symbol-event.svg";
    pub(crate) const CODEPEN_ICON_PATH: &str = "assets/codepen.svg";
    pub(crate) const GIT_COMMIT_HORIZONTAL_ICON_PATH: &str = "assets/git-commit-horizontal.svg";
    pub(crate) const COPY_ICON_PATH: &str = "assets/copy.svg";
    pub(crate) const REFRESH_CCW_ICON_PATH: &str = "assets/refresh-ccw.svg";
    pub(crate) const FOLDER_SYNC_ICON_PATH: &str = "assets/folder-sync.svg";
    pub(crate) const SAVE_ICON_PATH: &str = "assets/save.svg";
    pub(crate) const HISTORY_ICON_PATH: &str = "assets/history.svg";
    pub(crate) const DISCORD_ICON_PATH: &str = "assets/discord.svg";
    pub(crate) const GITHUB_ICON_PATH: &str = "assets/github.svg";
    pub(crate) const YOUTUBE_ICON_PATH: &str = "assets/youtube.svg";

    pub(crate) const ASSET_FILE_ICON: &[u8] = include_bytes!("../../../assets/file-box.svg");
    pub(crate) const CHEVRON_RIGHT_ICON: &[u8] =
        include_bytes!("../../../assets/chevron-right.svg");
    pub(crate) const CHEVRON_DOWN_ICON: &[u8] = include_bytes!("../../../assets/chevron-down.svg");
    pub(crate) const CHEVRON_UP_ICON: &[u8] = include_bytes!("../../../assets/chevron-up.svg");
    pub(crate) const CHECK_ICON: &[u8] = include_bytes!("../../../assets/check.svg");
    pub(crate) const TRASH_ICON: &[u8] = include_bytes!("../../../assets/trash.svg");
    pub(crate) const CIRCLE_PLUS_ICON: &[u8] = include_bytes!("../../../assets/circle-plus.svg");
    pub(crate) const SYMBOL_ARRAY_ICON: &[u8] = include_bytes!("../../../assets/symbol-array.svg");
    pub(crate) const SYMBOL_COLOR_ICON: &[u8] = include_bytes!("../../../assets/symbol-color.svg");
    pub(crate) const SYMBOL_FIELD_ICON: &[u8] = include_bytes!("../../../assets/symbol-field.svg");
    pub(crate) const SYMBOL_NUMERIC_ICON: &[u8] =
        include_bytes!("../../../assets/symbol-numeric.svg");
    pub(crate) const SYMBOL_STRING_ICON: &[u8] =
        include_bytes!("../../../assets/symbol-string.svg");
    pub(crate) const SQUARE_FUNCTION_ICON: &[u8] =
        include_bytes!("../../../assets/square-function.svg");
    pub(crate) const RECTANGLE_ELLIPSIS_ICON: &[u8] =
        include_bytes!("../../../assets/rectangle-ellipsis.svg");
    pub(crate) const OCTAGON_ALERT_ICON: &[u8] =
        include_bytes!("../../../assets/octagon-alert.svg");
    pub(crate) const SYMBOL_EVENT_ICON: &[u8] = include_bytes!("../../../assets/symbol-event.svg");
    pub(crate) const CODEPEN_ICON: &[u8] = include_bytes!("../../../assets/codepen.svg");
    pub(crate) const GIT_COMMIT_HORIZONTAL_ICON: &[u8] =
        include_bytes!("../../../assets/git-commit-horizontal.svg");
    pub(crate) const COPY_ICON: &[u8] = include_bytes!("../../../assets/copy.svg");
    pub(crate) const REFRESH_CCW_ICON: &[u8] = include_bytes!("../../../assets/refresh-ccw.svg");
    pub(crate) const FOLDER_SYNC_ICON: &[u8] = include_bytes!("../../../assets/folder-sync.svg");
    pub(crate) const SAVE_ICON: &[u8] = include_bytes!("../../../assets/save.svg");
    pub(crate) const HISTORY_ICON: &[u8] = include_bytes!("../../../assets/history.svg");
    pub(crate) const DISCORD_ICON: &[u8] = include_bytes!("../../../assets/discord.svg");
    pub(crate) const GITHUB_ICON: &[u8] = include_bytes!("../../../assets/github.svg");
    pub(crate) const YOUTUBE_ICON: &[u8] = include_bytes!("../../../assets/youtube.svg");
}

// ====================
// Types.
// ====================

#[derive(Clone, Debug)]
pub enum ProductIcon
{
    Asset,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    Check,
    Trash,
    CirclePlus,
    SymbolArray,
    SymbolColor,
    SymbolField,
    SymbolNumeric,
    SymbolString,
    SquareFunction,
    RectangleEllipsis,
    OctagonAlert,
    SymbolEvent,
    Codepen,
    GitCommitHorizontal,
    Copy,
    RefreshCcw,
    FolderSync,
    Save,
    History,
    Discord,
    Github,
    Youtube,
}

impl ProductIcon
{
    pub fn to_img(&self) -> gpui::Img
    {
        match self {
            ProductIcon::Asset => asset_icon_img(),
            ProductIcon::ChevronRight => chevron_right_icon_img(),
            ProductIcon::ChevronDown => chevron_down_icon_img(),
            ProductIcon::ChevronUp => chevron_up_icon_img(),
            ProductIcon::Check => check_icon_img(),
            ProductIcon::Trash => trash_icon_img(),
            ProductIcon::CirclePlus => circle_plus_icon_img(),
            ProductIcon::SymbolArray => symbol_array_icon_img(),
            ProductIcon::SymbolColor => symbol_color_icon_img(),
            ProductIcon::SymbolField => symbol_field_icon_img(),
            ProductIcon::SymbolNumeric => symbol_numeric_icon_img(),
            ProductIcon::SymbolString => symbol_string_icon_img(),
            ProductIcon::SquareFunction => square_function_icon_img(),
            ProductIcon::RectangleEllipsis => rectangle_ellipsis_icon_img(),
            ProductIcon::OctagonAlert => octagon_alert_icon_img(),
            ProductIcon::SymbolEvent => symbol_event_icon_img(),
            ProductIcon::Codepen => codepen_icon_img(),
            ProductIcon::GitCommitHorizontal => git_commit_horizontal_icon_img(),
            ProductIcon::Copy => copy_icon_img(),
            ProductIcon::RefreshCcw => refresh_ccw_icon_img(),
            ProductIcon::FolderSync => folder_sync_icon_img(),
            ProductIcon::Save => save_icon_img(),
            ProductIcon::History => history_icon_img(),
            ProductIcon::Discord => discord_icon_img(),
            ProductIcon::Github => github_icon_img(),
            ProductIcon::Youtube => youtube_icon_img(),
        }
    }

    pub fn to_svg(&self) -> gpui::Svg
    {
        match self {
            ProductIcon::Asset => asset_icon_svg(),
            ProductIcon::ChevronRight => chevron_right_icon_svg(),
            ProductIcon::ChevronDown => chevron_down_icon_svg(),
            ProductIcon::ChevronUp => chevron_up_icon_svg(),
            ProductIcon::Check => check_icon_svg(),
            ProductIcon::Trash => trash_icon_svg(),
            ProductIcon::CirclePlus => circle_plus_icon_svg(),
            ProductIcon::SymbolArray => symbol_array_icon_svg(),
            ProductIcon::SymbolColor => symbol_color_icon_svg(),
            ProductIcon::SymbolField => symbol_field_icon_svg(),
            ProductIcon::SymbolNumeric => symbol_numeric_icon_svg(),
            ProductIcon::SymbolString => symbol_string_icon_svg(),
            ProductIcon::SquareFunction => square_function_icon_svg(),
            ProductIcon::RectangleEllipsis => rectangle_ellipsis_icon_svg(),
            ProductIcon::OctagonAlert => octagon_alert_icon_svg(),
            ProductIcon::SymbolEvent => symbol_event_icon_svg(),
            ProductIcon::Codepen => codepen_icon_svg(),
            ProductIcon::GitCommitHorizontal => git_commit_horizontal_icon_svg(),
            ProductIcon::Copy => copy_icon_svg(),
            ProductIcon::RefreshCcw => refresh_ccw_icon_svg(),
            ProductIcon::FolderSync => folder_sync_icon_svg(),
            ProductIcon::Save => save_icon_svg(),
            ProductIcon::History => history_icon_svg(),
            ProductIcon::Discord => discord_icon_svg(),
            ProductIcon::Github => github_icon_svg(),
            ProductIcon::Youtube => youtube_icon_svg(),
        }
    }
}

#[inline(always)]
fn asset_icon_img() -> Img
{
    let bytes = consts::ASSET_FILE_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn chevron_right_icon_img() -> Img
{
    let bytes = consts::CHEVRON_RIGHT_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn chevron_down_icon_img() -> Img
{
    let bytes = consts::CHEVRON_DOWN_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn chevron_up_icon_img() -> Img
{
    let bytes = consts::CHEVRON_UP_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn check_icon_img() -> Img
{
    let bytes = consts::CHECK_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn trash_icon_img() -> Img
{
    let bytes = consts::TRASH_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn circle_plus_icon_img() -> Img
{
    let bytes = consts::CIRCLE_PLUS_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn symbol_array_icon_img() -> Img
{
    let bytes = consts::SYMBOL_ARRAY_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn symbol_color_icon_img() -> Img
{
    let bytes = consts::SYMBOL_COLOR_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn symbol_field_icon_img() -> Img
{
    let bytes = consts::SYMBOL_FIELD_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn symbol_numeric_icon_img() -> Img
{
    let bytes = consts::SYMBOL_NUMERIC_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn symbol_string_icon_img() -> Img
{
    let bytes = consts::SYMBOL_STRING_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn symbol_numeric_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::SYMBOL_NUMERIC_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn asset_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::ASSET_FILE_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn chevron_right_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::CHEVRON_RIGHT_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn chevron_down_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::CHEVRON_DOWN_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn chevron_up_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::CHEVRON_UP_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn check_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::CHECK_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn trash_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::TRASH_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn circle_plus_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::CIRCLE_PLUS_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn symbol_array_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::SYMBOL_ARRAY_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn square_function_icon_img() -> Img
{
    let bytes = consts::SQUARE_FUNCTION_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn rectangle_ellipsis_icon_img() -> Img
{
    let bytes = consts::RECTANGLE_ELLIPSIS_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn octagon_alert_icon_img() -> Img
{
    let bytes = consts::OCTAGON_ALERT_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn symbol_event_icon_img() -> Img
{
    let bytes = consts::SYMBOL_EVENT_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn git_commit_horizontal_icon_img() -> Img
{
    let bytes = consts::GIT_COMMIT_HORIZONTAL_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn copy_icon_img() -> Img
{
    let bytes = consts::COPY_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn codepen_icon_img() -> Img
{
    let bytes = consts::CODEPEN_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn square_function_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::SQUARE_FUNCTION_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn rectangle_ellipsis_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::RECTANGLE_ELLIPSIS_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn octagon_alert_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::OCTAGON_ALERT_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn symbol_event_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::SYMBOL_EVENT_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn symbol_color_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::SYMBOL_COLOR_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn symbol_field_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::SYMBOL_FIELD_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn symbol_string_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::SYMBOL_STRING_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn codepen_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::CODEPEN_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn git_commit_horizontal_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::GIT_COMMIT_HORIZONTAL_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn copy_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::COPY_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn refresh_ccw_icon_img() -> Img
{
    let bytes = consts::REFRESH_CCW_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn folder_sync_icon_img() -> Img
{
    let bytes = consts::FOLDER_SYNC_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn refresh_ccw_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::REFRESH_CCW_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn folder_sync_icon_svg() -> Svg
{
    svg()
        .path(format!(
            "{}/{}",
            consts::ROOT,
            consts::FOLDER_SYNC_ICON_PATH
        ))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn save_icon_img() -> Img
{
    let bytes = consts::SAVE_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn save_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::SAVE_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn history_icon_img() -> Img
{
    let bytes = consts::HISTORY_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn history_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::HISTORY_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn discord_icon_img() -> Img
{
    let bytes = consts::DISCORD_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn discord_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::DISCORD_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn github_icon_img() -> Img
{
    let bytes = consts::GITHUB_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn youtube_icon_img() -> Img
{
    let bytes = consts::YOUTUBE_ICON.to_vec();
    img(ImageSource::Image(Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        bytes,
    ))))
    .size_4()
}

#[inline(always)]
fn github_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::GITHUB_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}

#[inline(always)]
fn youtube_icon_svg() -> Svg
{
    svg()
        .path(format!("{}/{}", consts::ROOT, consts::YOUTUBE_ICON_PATH))
        .size_4()
        .text_color(icon_default())
}
