use gpui::{Hsla, Rgba, hsla, rgb};

pub fn background_primary() -> Rgba
{
    rgb(0x1c1d20)
}

pub fn background_darker() -> Rgba
{
    rgb(0x161719)
}

pub fn surface_elevated() -> Rgba
{
    rgb(0x3a3c40)
}

pub fn surface_overlay() -> Rgba
{
    rgb(0x3a3c40)
}

pub fn border_default() -> Rgba
{
    rgb(0x383a3f)
}

pub fn border_subtle() -> Rgba
{
    rgb(0x2a2c30)
}

pub fn border_focus() -> Rgba
{
    rgb(0x90908d)
}

pub fn border_separator() -> Rgba
{
    rgb(0x323439)
}

pub fn border_active() -> Rgba
{
    rgb(0x5ba7f7)
}

pub fn button_primary() -> Rgba
{
    rgb(0x4f94d4)
}

pub fn button_primary_hover() -> Rgba
{
    rgb(0x5ba7f7)
}

pub fn button_secondary() -> Rgba
{
    rgb(0x353942)
}

pub fn button_secondary_hover() -> Rgba
{
    rgb(0x404550)
}

pub fn button_success() -> Rgba
{
    rgb(0x36373b)
}

pub fn button_success_hover() -> Rgba
{
    rgb(0x3f4045)
}

pub fn button_danger() -> Rgba
{
    rgb(0x36373b)
}

pub fn button_danger_hover() -> Rgba
{
    rgb(0x3f4045)
}

pub fn selection_active() -> Rgba
{
    rgb(0x264f78)
}

pub fn selection_inactive() -> Rgba
{
    rgb(0x353942)
}

pub fn hover_overlay() -> Rgba
{
    Rgba {
        r: 0x40 as f32 / 255.0,
        g: 0x45 as f32 / 255.0,
        b: 0x50 as f32 / 255.0,
        a: 0.3,
    }
}

pub fn hover_subtle() -> Rgba
{
    Rgba {
        r: 0x35 as f32 / 255.0,
        g: 0x39 as f32 / 255.0,
        b: 0x42 as f32 / 255.0,
        a: 0.5,
    }
}

pub fn text_primary() -> Rgba
{
    rgb(0xe6e7eb)
}

pub fn text_secondary() -> Rgba
{
    rgb(0x9ca0a8)
}

pub fn text_muted() -> Rgba
{
    rgb(0x6b7280)
}

pub fn text_disabled() -> Rgba
{
    rgb(0x4b5563)
}

pub fn text_accent() -> Rgba
{
    rgb(0x5ba7f7)
}

pub fn status_unsaved() -> Rgba
{
    rgb(0x90908d)
}

pub fn status_saved() -> Rgba
{
    rgb(0x90908d)
}

pub fn status_error() -> Rgba
{
    rgb(0x90908d)
}

pub fn status_warning() -> Rgba
{
    rgb(0x90908d)
}

pub fn panel_file_browser() -> Rgba
{
    rgb(0x1c1d20)
}

pub fn panel_asset_editor() -> Rgba
{
    rgb(0x252629)
}

pub fn panel_toolbar() -> Rgba
{
    rgb(0x2a2c30)
}

pub fn accent_blue() -> Rgba
{
    rgb(0x4f94d4)
}

pub fn accent_blue_light() -> Rgba
{
    rgb(0x5ba7f7)
}

// Shadow colors for depth and elevation
pub fn shadow_light() -> Hsla
{
    hsla(0.0, 0.0, 0.0, 0.1)
}

pub fn shadow_medium() -> Hsla
{
    hsla(0.0, 0.0, 0.0, 0.2)
}

pub fn shadow_heavy() -> Hsla
{
    hsla(0.0, 0.0, 0.0, 0.3)
}

// Subtle highlight colors
pub fn highlight_subtle() -> Rgba
{
    Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.05,
    }
}

pub fn highlight_border() -> Rgba
{
    Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.1,
    }
}

// Additional colors for TextInput variants
pub fn surface_muted() -> Rgba
{
    rgb(0x2a2c30)
}

pub fn border_muted() -> Rgba
{
    rgb(0x2d2f32)
}

pub fn text_default() -> Rgba
{
    text_primary()
}

// Primary variant colors
pub fn primary() -> Rgba
{
    rgb(0x4f94d4)
}

pub fn primary_muted() -> Rgba
{
    rgb(0x3a6fa3)
}

pub fn primary_emphasis() -> Rgba
{
    rgb(0x5ba7f7)
}

pub fn text_on_primary() -> Rgba
{
    rgb(0xffffff)
}

// Secondary variant colors
pub fn secondary() -> Rgba
{
    rgb(0x525669)
}

pub fn secondary_muted() -> Rgba
{
    rgb(0x3a3c50)
}

pub fn secondary_emphasis() -> Rgba
{
    rgb(0x6b6f87)
}

pub fn text_on_secondary() -> Rgba
{
    rgb(0xffffff)
}

// Success variant colors
pub fn success() -> Rgba
{
    rgb(0x4f9d4f)
}

pub fn success_muted() -> Rgba
{
    rgb(0x3a733a)
}

pub fn success_emphasis() -> Rgba
{
    rgb(0x5fb85f)
}

pub fn text_on_success() -> Rgba
{
    rgb(0xffffff)
}

// Warning variant colors
pub fn warning() -> Rgba
{
    rgb(0xd4a94f)
}

pub fn warning_muted() -> Rgba
{
    rgb(0xa3833a)
}

pub fn warning_emphasis() -> Rgba
{
    rgb(0xf7c55b)
}

pub fn text_on_warning() -> Rgba
{
    rgb(0x1c1d20)
}

// Error variant colors
pub fn error() -> Rgba
{
    rgb(0xd44f4f)
}

pub fn error_muted() -> Rgba
{
    rgb(0xa33a3a)
}

pub fn error_emphasis() -> Rgba
{
    rgb(0xf75b5b)
}

pub fn text_on_error() -> Rgba
{
    rgb(0xffffff)
}

// Transparent colors
pub fn transparent_black() -> Rgba
{
    Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    }
}

// Dropdown specific colors
pub fn dropdown_hover() -> Rgba
{
    rgb(0x264f78) // Bluish hover background
}

pub fn dropdown_selected() -> Rgba
{
    rgb(0x2a2c30) // Darker background for selected item
}

pub fn icon_default() -> Rgba
{
    rgb(0xe4e4e7)
}

// Syntax highlighting colors - One Dark theme
pub fn syntax_number() -> Rgba
{
    rgb(0xD19A66) // Orange for numbers (One Dark)
}

pub fn syntax_string() -> Rgba
{
    rgb(0x98C379) // Green for strings (One Dark)
}

pub fn syntax_function() -> Rgba
{
    rgb(0x61AFEF) // Blue for functions (One Dark)
}

pub fn syntax_builtin() -> Rgba
{
    rgb(0xE06C75) // Red for built-ins (One Dark)
}

pub fn syntax_keyword() -> Rgba
{
    rgb(0xC678DD) // Purple for keywords (One Dark)
}

pub fn syntax_operator() -> Rgba
{
    rgb(0x56B6C2) // Cyan for operators (One Dark)
}

pub fn syntax_identifier() -> Rgba
{
    rgb(0xABB2BF) // Light gray for identifiers (One Dark)
}

pub fn syntax_punctuation() -> Rgba
{
    rgb(0x5C6370) // Dark gray for punctuation (One Dark)
}

pub fn syntax_error() -> Rgba
{
    rgb(0xE06C75) // Red for errors (One Dark)
}

// Expression validation colors
pub fn expr_valid() -> Rgba
{
    rgb(0x4ec9b0) // Teal for valid expressions
}

pub fn expr_invalid() -> Rgba
{
    rgb(0xf44747) // Red for invalid expressions
}

pub fn expr_warning() -> Rgba
{
    rgb(0xddb700) // Yellow for warnings
}

// Auto-complete colors
pub fn autocomplete_bg() -> Rgba
{
    rgb(0x1e1f22) // Darker background for dropdown
}

pub fn autocomplete_hover() -> Rgba
{
    rgb(0x264f78) // Blue hover background
}

pub fn autocomplete_selected() -> Rgba
{
    rgb(0x094771) // Darker blue for selected
}

pub fn autocomplete_match() -> Rgba
{
    rgb(0x5ba7f7) // Bright blue for matching text
}

pub fn autocomplete_type() -> Rgba
{
    rgb(0x6b7280) // Gray for type info
}

// Additional semantic colors
pub fn text_info() -> Rgba
{
    rgb(0x3b82f6) // Blue for info
}

pub fn text_success() -> Rgba
{
    rgb(0x10b981) // Green for success
}

pub fn text_warning() -> Rgba
{
    rgb(0xf59e0b) // Amber for warning
}

pub fn text_danger() -> Rgba
{
    rgb(0xef4444) // Red for danger
}

pub fn surface_info() -> Rgba
{
    Rgba {
        r: 0x3b as f32 / 255.0,
        g: 0x82 as f32 / 255.0,
        b: 0xf6 as f32 / 255.0,
        a: 0.1,
    }
}

pub fn surface_danger() -> Rgba
{
    Rgba {
        r: 0xef as f32 / 255.0,
        g: 0x44 as f32 / 255.0,
        b: 0x44 as f32 / 255.0,
        a: 1.0,
    }
}

pub fn border_info() -> Rgba
{
    Rgba {
        r: 0x3b as f32 / 255.0,
        g: 0x82 as f32 / 255.0,
        b: 0xf6 as f32 / 255.0,
        a: 1.0,
    }
}

pub fn border_danger() -> Rgba
{
    Rgba {
        r: 0xef as f32 / 255.0,
        g: 0x44 as f32 / 255.0,
        b: 0x44 as f32 / 255.0,
        a: 1.0,
    }
}

// Expression input background
pub fn expr_input_bg() -> Rgba
{
    rgb(0x1a1b1e) // Slightly darker than default background
}

// Error panel colors - Red theme (syntax errors)
pub fn error_panel_red_bg() -> Rgba
{
    background_darker() // Use dark background instead of bright red
}

pub fn error_panel_red_border() -> Rgba
{
    rgb(0xFF7979) // Light red border
}

pub fn error_panel_red_icon() -> Rgba
{
    rgb(0xC0392B) // Dark red icon
}

pub fn error_panel_red_glow() -> Hsla
{
    hsla(0.0, 0.79, 0.57, 0.27) // Red glow (E74C3C with 27% opacity)
}

// Error panel colors - Orange theme (invalid values)
pub fn error_panel_orange_bg() -> Rgba
{
    background_darker() // Use dark background instead of bright orange
}

pub fn error_panel_orange_border() -> Rgba
{
    rgb(0xFFB84D) // Light orange border
}

pub fn error_panel_orange_icon() -> Rgba
{
    rgb(0xD68910) // Dark orange icon
}

pub fn error_panel_orange_glow() -> Hsla
{
    hsla(37.0 / 360.0, 0.89, 0.51, 0.27) // Orange glow (F39C12 with 27% opacity)
}

// Error panel colors - Purple theme (unknown references)
pub fn error_panel_purple_bg() -> Rgba
{
    background_darker() // Use dark background instead of bright purple
}

pub fn error_panel_purple_border() -> Rgba
{
    rgb(0xBB8FCC) // Light purple border
}

pub fn error_panel_purple_icon() -> Rgba
{
    rgb(0x7D3C98) // Dark purple icon
}

pub fn error_panel_purple_glow() -> Hsla
{
    hsla(282.0 / 360.0, 0.39, 0.53, 0.27) // Purple glow (9B59B6 with 27% opacity)
}

// Error panel colors - Blue theme (type errors)
pub fn error_panel_blue_bg() -> Rgba
{
    background_darker() // Use dark background instead of bright blue
}

pub fn error_panel_blue_border() -> Rgba
{
    rgb(0x5DADE2) // Light blue border
}

pub fn error_panel_blue_icon() -> Rgba
{
    rgb(0x2874A6) // Dark blue icon
}

pub fn success_panel_border() -> Rgba
{
    rgb(0x10b981) // Green border for success
}

pub fn info_panel_border() -> Rgba
{
    rgb(0x3b82f6) // Blue border for info
}

pub fn error_panel_blue_glow() -> Hsla
{
    hsla(204.0 / 360.0, 0.70, 0.53, 0.27) // Blue glow (3498DB with 27% opacity)
}

// Error panel common colors
pub fn error_panel_icon_bg() -> Rgba
{
    rgb(0xFFFFFF) // White icon background
}

pub fn error_panel_text() -> Rgba
{
    rgb(0xFFFFFF) // White text for contrast
}

pub fn error_panel_shadow() -> Hsla
{
    hsla(0.0, 0.0, 0.0, 0.27) // Soft shadow (27% opacity black)
}

// Save button specific colors
pub fn save_button_bg() -> Rgba
{
    rgb(0x2d5a2d) // Dark green background
}

pub fn save_button_bg_hover() -> Rgba
{
    rgb(0x3a6f3a) // Lighter green on hover
}

pub fn save_button_bg_active() -> Rgba
{
    rgb(0x1e3d1e) // Darker green when pressed
}

pub fn save_button_border() -> Rgba
{
    rgb(0x4a7c4a) // Green border
}

pub fn save_button_border_hover() -> Rgba
{
    rgb(0x5a8c5a) // Lighter green border on hover
}

pub fn save_button_text() -> Rgba
{
    rgb(0x7dd87d) // Light green text
}

pub fn save_button_text_disabled() -> Rgba
{
    rgb(0x4a5a4a) // Muted green text when disabled
}

pub fn save_button_icon() -> Rgba
{
    rgb(0x7dd87d) // Light green icon
}

pub fn save_button_icon_disabled() -> Rgba
{
    rgb(0x5a6a5a) // Muted green icon when disabled
}

pub fn save_button_bg_disabled() -> Rgba
{
    rgb(0x2a3a2a) // Darker green background when disabled
}

pub fn save_button_border_disabled() -> Rgba
{
    rgb(0x3a4a3a) // Muted green border when disabled
}

pub fn save_button_glow() -> Hsla
{
    hsla(120.0 / 360.0, 0.6, 0.5, 0.3) // Green glow effect
}

// Revert button specific colors
pub fn revert_button_bg() -> Rgba
{
    rgb(0x5a2d2d) // Dark red background
}

pub fn revert_button_bg_hover() -> Rgba
{
    rgb(0x6f3a3a) // Lighter red on hover
}

pub fn revert_button_bg_active() -> Rgba
{
    rgb(0x3d1e1e) // Darker red when pressed
}

pub fn revert_button_border() -> Rgba
{
    rgb(0x7c4a4a) // Red border
}

pub fn revert_button_border_hover() -> Rgba
{
    rgb(0x8c5a5a) // Lighter red border on hover
}

pub fn revert_button_text() -> Rgba
{
    rgb(0xd87d7d) // Light red text
}

pub fn revert_button_text_disabled() -> Rgba
{
    rgb(0x5a4a4a) // Muted red text when disabled
}

pub fn revert_button_icon() -> Rgba
{
    rgb(0xd87d7d) // Light red icon
}

pub fn revert_button_icon_disabled() -> Rgba
{
    rgb(0x6a5a5a) // Muted red icon when disabled
}

pub fn revert_button_bg_disabled() -> Rgba
{
    rgb(0x3a2a2a) // Darker red background when disabled
}

pub fn revert_button_border_disabled() -> Rgba
{
    rgb(0x4a3a3a) // Muted red border when disabled
}

pub fn revert_button_glow() -> Hsla
{
    hsla(0.0 / 360.0, 0.6, 0.5, 0.3) // Red glow effect
}
