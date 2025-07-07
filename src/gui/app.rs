use std::path::PathBuf;

// ====================
// Deps.
// ====================
use clap::Parser;
// ====================
// GPUI.
// ====================
use gpui::prelude::*;
use gpui::{
    App,
    Application,
    Bounds,
    BoxShadow,
    Context,
    Entity,
    IntoElement,
    KeyBinding,
    ParentElement,
    SharedString,
    Styled,
    TitlebarOptions,
    Window,
    WindowBounds,
    WindowKind,
    WindowOptions,
    div,
    point,
    px,
    size,
};
use tracing::{debug, error};
use tracing_subscriber::FmtSubscriber;

// ====================
// Editor.
// ====================
use crate::gui::app_status_bar::AppStatusBar;
use crate::gui::asset_editor::editor::{AssetEditor, AssetUpdated};
use crate::gui::client::ViewerSyncClient;
use crate::gui::file_browser::browser::FileBrowser;
use crate::gui::file_browser::events::{FileBrowserEvent, StatusBarEvent};
use crate::gui::models::state::FromHanabi;
use crate::gui::server_wrapper::ServerWrapper;
use crate::gui::styling::colors::*;
use crate::gui::styling::fonts::*;
use crate::gui::utils::asset_source::GuiAssets;

// ====================
// Actions.
// ====================
mod actions
{
    gpui::actions!(gui_app, [Quit]);
}

#[derive(Default, Parser)]
#[command(name = "drakkar-vfx", about = "Drakkar VFX Particle Editor")]
pub struct GuiCliArgs
{
    #[clap(long, short = 'V')]
    pub version:           bool,
    #[clap(long, default_value = "false")]
    pub skip_start_viewer: bool,
    #[clap(long)]
    pub assets_root:       Option<PathBuf>,
}

impl GuiCliArgs
{
    pub fn display_version()
    {
        let version = env!("CARGO_PKG_VERSION");
        let git_sha = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
        let git_commit_date = option_env!("VERGEN_GIT_COMMIT_TIMESTAMP").unwrap_or("unknown");
        let rust_version = option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown");
        let target = option_env!("BUILD_TARGET").unwrap_or("unknown");
        let build_date = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown");

        println!("Drakkar VFX {}", version);
        println!("Build Information:");
        println!("  Git SHA: {}", git_sha);
        println!("  Git Commit Date: {}", git_commit_date);
        println!("  Rust Version: {}", rust_version);
        println!("  Target: {}", target);
        println!("  Build Date: {}", build_date);
    }
}

pub struct GuiAppState
{
    pub server_wrapper:    ServerWrapper,
    pub sync_client:       ViewerSyncClient,
    pub file_browser:      Entity<FileBrowser>,
    pub asset_editor:      Entity<AssetEditor>,
    pub bottom_status_bar: Entity<AppStatusBar>,
}

impl GuiAppState
{
    fn new(cx: &mut Context<Self>, args: GuiCliArgs) -> Self
    {
        let assets_root = args.assets_root.unwrap_or_else(|| {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(manifest_dir).join("assets")
        });

        let mut server_wrapper = ServerWrapper::new();
        if !args.skip_start_viewer {
            match server_wrapper.start() {
                Ok(()) => {
                    debug!("Viewer server started successfully")
                }
                Err(e) => {
                    error!("Failed to start viewer server: {}", e)
                }
            }
        }

        let asset_editor = cx.new(|cx| AssetEditor::new(cx));
        let file_browser = cx.new(|cx| FileBrowser::new(cx, assets_root.clone()));
        let bottom_status_bar = cx.new(|_| AppStatusBar::new());
        let file_browser_weak_ref = file_browser.downgrade();

        file_browser.update(cx, move |file_browser, cx| {
            file_browser.weak_ref = file_browser_weak_ref;
            file_browser.set_update_file_list(cx, "", true);
        });

        cx.subscribe(&file_browser, Self::on_file_browser_event)
            .detach();

        cx.subscribe(&file_browser, Self::on_status_bar_event)
            .detach();

        cx.subscribe(&asset_editor, Self::on_asset_changed).detach();

        Self {
            server_wrapper,
            file_browser,
            asset_editor,
            sync_client: ViewerSyncClient::new(),
            bottom_status_bar,
        }
    }

    // ====================
    // Event handlers.
    // ====================

    fn on_file_browser_event(
        this: &mut Self,
        _: Entity<FileBrowser>,
        ev: &FileBrowserEvent,
        cx: &mut Context<Self>,
    )
    {
        match ev {
            FileBrowserEvent::BufferStateSelected { path, asset_state } => {
                this.sync_client.send_open_asset_file(path);
                this.asset_editor.update(cx, |editor, cx| {
                    editor.on_buffer_state_selected(asset_state.clone(), cx);
                });
            }
            FileBrowserEvent::AssetChanged {
                path: _,
                asset_state,
            } => {
                this.file_browser.update(cx, |browser, cx| {
                    if let Some(selected_ix) = browser.selected_file {
                        if let Some(file_item) = browser.filtered_files.get(selected_ix) {
                            file_item.update(cx, |item, _| {
                                item.update_buffer(asset_state.clone());
                            });
                        }
                    }
                });
            }
            FileBrowserEvent::BufferLoadError { path, error } => {
                this.asset_editor.update(cx, |editor, cx| {
                    editor.show_error(&path, error.to_string(), cx);
                });
            }
        }
    }

    fn on_asset_changed(
        this: &mut Self,
        _: Entity<AssetEditor>,
        ev: &AssetUpdated,
        cx: &mut Context<Self>,
    )
    {
        debug!("Asset changed, sending to viewer");
        this.sync_client.send_open_asset(ev.effect_asset.clone());

        // Update the buffer in the file browser if we have a selected file
        this.file_browser.update(cx, |browser, cx| {
            if let Some(selected_ix) = browser.selected_file {
                if let Some(file_item) = browser.filtered_files.get(selected_ix) {
                    if let Ok(asset_state) = FromHanabi::asset_state(&ev.effect_asset) {
                        file_item.update(cx, |item, _| {
                            item.update_buffer(asset_state);
                        });
                    }
                }
            }
        });
    }

    fn on_status_bar_event(
        this: &mut Self,
        _: Entity<FileBrowser>,
        ev: &StatusBarEvent,
        cx: &mut Context<Self>,
    )
    {
        match ev {
            StatusBarEvent::RefreshRequested => {
                this.file_browser.update(cx, |browser, cx| {
                    browser.set_update_file_list(cx, "", true);
                    browser.set_update_scm_statuses(cx);
                });
            }
        }
    }
}

impl Drop for GuiAppState
{
    fn drop(&mut self)
    {
        if let Err(e) = self.server_wrapper.stop() {
            error!("Failed to stop server wrapper: {}", e);
        }
        std::process::exit(0);
    }
}

impl Render for GuiAppState
{
    fn render(&mut self, _window: &mut Window, _: &mut Context<Self>) -> impl IntoElement
    {
        self.server_wrapper.drain_all_messages();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(background_primary())
            .child(
                div()
                    .id("top-status-bar-container")
                    .h(px(24.0))
                    .child(self.bottom_status_bar.clone()),
            )
            .child(
                div()
                    .id("file-browser-container")
                    .h(px(350.0))
                    .bg(panel_file_browser())
                    .border_b_1()
                    .border_color(border_separator())
                    .shadow(vec![BoxShadow {
                        color:         shadow_medium(),
                        offset:        point(px(0.), px(2.)),
                        blur_radius:   px(6.),
                        spread_radius: px(0.),
                    }])
                    .child(self.file_browser.clone()),
            )
            .child(
                div()
                    .id("models-editor-container")
                    .size_full()
                    .flex_1()
                    .flex_shrink_0()
                    .child(self.asset_editor.clone()),
            )
    }
}

/// Entry point.
///
pub fn gui_main()
{
    let args = GuiCliArgs::parse();

    // Check if version was requested
    if args.version {
        GuiCliArgs::display_version();
        return;
    }

    let rust_log = std::env::var("RUST_LOG")
        .ok()
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .unwrap_or_else(|| "info".to_owned());

    tracing::subscriber::set_global_default(
        FmtSubscriber::builder().with_env_filter(rust_log).finish(),
    )
    .expect("Failed to set up tracing subscriber");

    let working_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));
    debug!("Starting particle editor GUI: {:?}", working_directory);

    Application::new()
        .with_assets(GuiAssets {})
        .run(|cx: &mut App| {
            cx.activate(true);
            cx.text_system()
                .add_fonts(vec![font_bytes().into()])
                .expect("Failed to load default font");
            let _ = cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            });

            cx.on_action(|_: &actions::Quit, cx| {
                cx.quit();
            });

            cx.bind_keys([
                KeyBinding::new("cmd-q", actions::Quit, None),
                KeyBinding::new("ctrl-q", actions::Quit, None),
                KeyBinding::new(
                    "up",
                    crate::gui::primitives::dropdown_input::actions::Up,
                    Some("Dropdown"),
                ),
                KeyBinding::new(
                    "down",
                    crate::gui::primitives::dropdown_input::actions::Down,
                    Some("Dropdown"),
                ),
                KeyBinding::new(
                    "enter",
                    crate::gui::primitives::dropdown_input::actions::Enter,
                    Some("Dropdown"),
                ),
                KeyBinding::new(
                    "escape",
                    crate::gui::primitives::dropdown_input::actions::Escape,
                    Some("Dropdown"),
                ),
                KeyBinding::new(
                    "backspace",
                    crate::gui::primitives::text_input::Backspace,
                    None,
                ),
                KeyBinding::new("delete", crate::gui::primitives::text_input::Delete, None),
                KeyBinding::new("left", crate::gui::primitives::text_input::Left, None),
                KeyBinding::new("right", crate::gui::primitives::text_input::Right, None),
                KeyBinding::new(
                    "shift-left",
                    crate::gui::primitives::text_input::SelectLeft,
                    None,
                ),
                KeyBinding::new(
                    "shift-right",
                    crate::gui::primitives::text_input::SelectRight,
                    None,
                ),
                KeyBinding::new("home", crate::gui::primitives::text_input::Home, None),
                KeyBinding::new("end", crate::gui::primitives::text_input::End, None),
                KeyBinding::new("enter", crate::gui::primitives::text_input::Enter, None),
                KeyBinding::new("escape", crate::gui::primitives::text_input::Escape, None),
                KeyBinding::new("cmd-a", crate::gui::primitives::text_input::SelectAll, None),
                KeyBinding::new("cmd-v", crate::gui::primitives::text_input::Paste, None),
                KeyBinding::new("cmd-c", crate::gui::primitives::text_input::Copy, None),
                KeyBinding::new("cmd-x", crate::gui::primitives::text_input::Cut, None),
                KeyBinding::new(
                    "up",
                    crate::gui::file_browser::item::actions::Up,
                    Some("FileEntry"),
                ),
                KeyBinding::new(
                    "down",
                    crate::gui::file_browser::item::actions::Down,
                    Some("FileEntry"),
                ),
                KeyBinding::new(
                    "enter",
                    crate::gui::file_browser::item::actions::Enter,
                    Some("FileEntry"),
                ),
            ]);

            let window_size = size(px(600.0), px(960.0));
            let window_bounds = Bounds::centered(None, window_size, cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("Drakkar VFX".to_string())),
                    ..Default::default()
                }),
                window_min_size: Some(gpui::Size {
                    width:  px(600.),
                    height: px(960.),
                }),
                kind: WindowKind::Normal,
                ..Default::default()
            };

            cx.spawn(async move |cx| {
                let window = cx
                    .open_window(options, |_, cx| cx.new(|cx| GuiAppState::new(cx, args)))
                    .expect("failed to open window");
                window
                    .update(cx, |_, window, _| {
                        window.activate_window();
                    })
                    .expect("failed to update window");
            })
            .detach();
        });

    debug!("Application shutting down gracefully");
}
