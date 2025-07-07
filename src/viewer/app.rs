// ====================
// Bevy stuff.
// ====================
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::gizmos::prelude::Gizmos;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::camera::ScalingMode;
use bevy::render::mesh::shape;
use bevy::render::render_resource::WgpuFeatures;
use bevy::render::settings::WgpuSettings;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::ui::Val;
use bevy::window::WindowResolution;
// ====================
// Particles.
// ====================
#[cfg(not(feature = "jarl"))]
use bevy_hanabi::{
    EffectAsset,
    EffectSpawner,
    ForceFieldModifier,
    HanabiPlugin,
    ParticleEffect,
    ParticleEffectBundle,
};
#[cfg(feature = "jarl")]
use jarl_particles::prelude::*;
#[cfg(feature = "jarl")]
use jarl_particles::{
    EffectAsset,
    EffectSpawner,
    ForceFieldModifier,
    HanabiPlugin,
    ParticleCountStats,
    ParticleEffect,
};

// ====================
// Editor.
// ====================
use crate::viewer::server::{ServerPlugin, ViewerCommandEvent};
const DEFAULT_FONT: &str = crate::common::VIEWER_FONT;

/// Component to track force field sources for gizmo rendering
#[derive(Component)]
pub struct ForceFieldSource
{
    pub position:          Vec3,
    pub max_radius:        f32,
    pub min_radius:        f32,
    pub mass:              f32,
    pub force_exponent:    f32,
    pub conform_to_sphere: bool,
}

/// Component to mark force field gizmo entities
#[derive(Component)]
pub struct ForceFieldGizmo;

/// Component marker for visual bounding box entities
#[derive(Component)]
pub struct VisualBbox
{
    pub min: Vec3,
    pub max: Vec3,
}

pub fn viewer_main()
{
    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "wgpu=error,naga=warn".to_string(),
                    ..default()
                })
                .set(RenderPlugin { wgpu_settings })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Drakkar VFX :: Viewer".to_string(),
                        position: WindowPosition::Automatic,
                        resolution: WindowResolution::new(800.0, 800.0),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(HanabiPlugin)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(ServerPlugin)
        .init_resource::<CameraController>()
        .init_resource::<ViewerSettings>()
        .init_resource::<ParticleStats>()
        .init_resource::<BackgroundColor>()
        .init_resource::<ViewerState>()
        .add_systems(Startup, sys_setup)
        .add_systems(
            Update,
            (
                sys_camera_controller_system,
                sys_input_system,
                sys_particle_stats,
                sys_ui_text,
                sys_update_grid_gizmos,
                sys_update_grid,
                sys_handle_server_commands,
                sys_update_background,
                sys_handle_buttons,
                sys_button_hover_effects,
                bevy::window::close_on_esc,
            ),
        )
        .add_systems(Update, sys_render_force_field_gizmos)
        .run();
}

#[derive(Component)]
pub struct GridLine;

#[derive(Component)]
pub enum UiText
{
    ParticleCount,
    Bbox,
    Fps,
    Controls,
    SpawnerInfo,
    TimingInfo,
    EmptyState,
    ErrorState,
}

#[derive(Component)]
pub struct ParticleEffectEntity;

#[derive(Component)]
pub enum ButtonType
{
    DarkBackground,
    LightBackground,
    Gizmos,
    Grid,
}

#[derive(Resource)]
pub struct ViewerState
{
    pub has_asset:     bool,
    pub error_message: Option<String>,
}

impl Default for ViewerState
{
    fn default() -> Self
    {
        Self {
            has_asset:     false,
            error_message: None,
        }
    }
}

fn sys_camera_controller_system(
    kb: Res<Input<KeyCode>>,
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    mut camera_controller: ResMut<CameraController>,
    time: Res<Time>,
)
{
    const CAMERA_SPEED: f32 = 200.0;
    const ZOOM_SPEED: f32 = 2.0;
    const SMOOTHING_FACTOR: f32 = 8.0;

    let mut dir = Vec2::ZERO;
    if kb.pressed(KeyCode::W) || kb.pressed(KeyCode::Up) {
        dir.y += 1.0;
    }
    if kb.pressed(KeyCode::S) || kb.pressed(KeyCode::Down) {
        dir.y -= 1.0;
    }
    if kb.pressed(KeyCode::A) || kb.pressed(KeyCode::Left) {
        dir.x -= 1.0;
    }
    if kb.pressed(KeyCode::D) || kb.pressed(KeyCode::Right) {
        dir.x += 1.0;
    }

    if dir.length() > 0.0 {
        dir = dir.normalize();
        let zoom = camera_controller.zoom;
        camera_controller.position += dir * CAMERA_SPEED * zoom * time.delta_seconds();
    }

    let mut zoom_delta = 0.0;
    if kb.pressed(KeyCode::Q) {
        zoom_delta += ZOOM_SPEED * time.delta_seconds();
    }
    if kb.pressed(KeyCode::E) {
        zoom_delta -= ZOOM_SPEED * time.delta_seconds();
    }

    if zoom_delta != 0.0 {
        camera_controller.zoom *= (1.0_f32 + zoom_delta).max(0.1);
        camera_controller.zoom = camera_controller
            .zoom
            .clamp(camera_controller.min_zoom, camera_controller.max_zoom);
    }

    for wheel in scroll.iter() {
        let zoom_delta = wheel.y * ZOOM_SPEED * time.delta_seconds();
        camera_controller.zoom *= (1.0_f32 - zoom_delta).max(0.1);
        camera_controller.zoom = camera_controller
            .zoom
            .clamp(camera_controller.min_zoom, camera_controller.max_zoom);
    }

    if let Ok((mut camera_transform, mut projection)) = camera_query.get_single_mut() {
        let target = camera_controller.position.extend(0.0);
        let current = camera_transform.translation;
        let smoothed = current.lerp(target, SMOOTHING_FACTOR * time.delta_seconds());
        camera_transform.translation = smoothed;
        projection.scale = camera_controller.zoom;
    }
}

fn sys_input_system(
    kb: Res<Input<KeyCode>>,
    mut viewer_settings: ResMut<ViewerSettings>,
    mut camera_controller: ResMut<CameraController>,
)
{
    if kb.just_pressed(KeyCode::G) {
        viewer_settings.show_grid = !viewer_settings.show_grid;
    }
    if kb.just_pressed(KeyCode::B) {
        viewer_settings.show_bbox = !viewer_settings.show_bbox;
    }
    if kb.just_pressed(KeyCode::I) {
        viewer_settings.show_stats = !viewer_settings.show_stats;
    }
    if kb.just_pressed(KeyCode::R) {
        camera_controller.position = Vec2::ZERO;
        camera_controller.zoom = 1.0;
    }
}

fn sys_particle_stats(
    mut particle_stats: ResMut<ParticleStats>,

    #[cfg(feature = "jarl")] particle_count_stats: Res<ParticleCountStats>,

    time: Res<Time>,
    particle_query: Query<(Entity, &EffectSpawner)>,
)
{
    let current_time = time.elapsed_seconds_f64();
    let delta_time = time.delta_seconds();

    particle_stats.particle_count = 0;
    particle_stats.bbox_size = Vec2::new(32.0, 32.0);

    if let Some((_entity, spawner)) = particle_query.iter().next() {
        #[cfg(feature = "jarl")]
        {
            particle_stats.particle_count = particle_count_stats.get_count(_entity);
        }

        let spawn_count = spawner.spawn_count();
        let spawner_config = spawner.spawner();
        if spawner_config.is_once() {
            particle_stats.spawner_type = "Once".to_string();
        } else {
            particle_stats.spawner_type = "Rate".to_string();
        }
        particle_stats.effect_age += delta_time;
        particle_stats.total_spawned = particle_stats.total_spawned.saturating_add(spawn_count);
    } else {
        #[cfg(feature = "jarl")]
        {
            particle_stats.particle_count = particle_count_stats.get_total_count();
        }
        particle_stats.spawner_type = "None".to_string();
    }
    particle_stats.last_update_time = current_time;
}

fn sys_ui_text(
    viewer_settings: Res<ViewerSettings>,
    particle_stats: Res<ParticleStats>,
    viewer_state: Res<ViewerState>,
    time: Res<Time>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    mut ui_text_query: Query<(&mut Text, &UiText)>,
)
{
    for (mut text, ui_text) in ui_text_query.iter_mut() {
        match ui_text {
            UiText::ParticleCount => {
                text.sections[0].value = if viewer_settings.show_stats && viewer_state.has_asset {
                    format!("Particles: {}", particle_stats.particle_count)
                } else {
                    String::new()
                };
            }
            UiText::SpawnerInfo => {
                text.sections[0].value = if viewer_settings.show_stats && viewer_state.has_asset {
                    format!(
                        "Spawner: {} | Total: {}",
                        particle_stats.spawner_type, particle_stats.total_spawned
                    )
                } else {
                    String::new()
                };
            }
            UiText::TimingInfo => {
                text.sections[0].value = if viewer_settings.show_stats && viewer_state.has_asset {
                    let current_time = time.elapsed_seconds_f64();
                    let sin_time = current_time.sin();
                    format!("Time: {:.1}s | Sin(time): {:.3}", current_time, sin_time)
                } else {
                    String::new()
                };
            }
            UiText::Bbox => {
                text.sections[0].value = if viewer_settings.show_bbox && viewer_state.has_asset {
                    format!(
                        "BBox: {:.1}x{:.1}",
                        particle_stats.bbox_size.x, particle_stats.bbox_size.y
                    )
                } else {
                    String::new()
                };
            }
            UiText::Fps => {
                text.sections[0].value = if viewer_settings.show_stats {
                    if let Some(fps) =
                        diagnostics.get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
                    {
                        if let Some(value) = fps.smoothed() {
                            format!("FPS: {:.1}", value)
                        } else {
                            "FPS: --".to_string()
                        }
                    } else {
                        "FPS: --".to_string()
                    }
                } else {
                    String::new()
                };
            }
            UiText::Controls => {
                text.sections[0].value =
                    "WASD: Move | Q/E: Zoom | G: Grid | B: BBox | I: Info & Stats | R: Reset | Esc: Exit"
                        .to_string();
            }
            UiText::EmptyState => {
                text.sections[0].value = if !viewer_state.has_asset
                    && viewer_state.error_message.is_none()
                {
                    "No particle effect loaded.\nOpen a .hanabi file from the editor to view it here.".to_string()
                } else {
                    String::new()
                };
            }
            UiText::ErrorState => {
                text.sections[0].value = if let Some(ref error) = viewer_state.error_message {
                    format!("Error loading effect:\n{}", error)
                } else {
                    String::new()
                };
            }
        }
    }
}

fn sys_update_grid(
    viewer_settings: Res<ViewerSettings>,
    grid_query: Query<Entity, With<GridLine>>,
    grid_label_query: Query<Entity, With<GridLabel>>,
    mut cmds: Commands,
)
{
    let visibility = if viewer_settings.show_grid {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for entity in grid_query.iter() {
        cmds.entity(entity).insert(visibility);
    }
    for entity in grid_label_query.iter() {
        cmds.entity(entity).insert(visibility);
    }
}

fn sys_handle_server_commands(
    mut events: EventReader<ViewerCommandEvent>,
    mut cmds: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut viewer_state: ResMut<ViewerState>,
    query_effects: Query<Entity, With<ParticleEffectEntity>>,
    query_force_fields: Query<Entity, With<ForceFieldSource>>,
)
{
    for event in events.iter() {
        match event {
            ViewerCommandEvent::OpenAsset { asset } => {
                let mut asset = asset.clone();
                if asset.spawner.period.single() > 3.0 {
                    asset.spawner.period = 1.0.into();
                }

                spawn_effect(
                    &mut cmds,
                    &mut effects,
                    &asset,
                    &query_effects,
                    &query_force_fields,
                );
                viewer_state.has_asset = true;
                viewer_state.error_message = None;
            }

            ViewerCommandEvent::OpenAssetFile { path: file_path } => {
                match std::fs::read_to_string(file_path) {
                    Ok(content) => match ron::from_str::<EffectAsset>(&content) {
                        Ok(mut effect) => {
                            if effect.spawner.period.single() > 3.0 {
                                effect.spawner.period = 1.0.into();
                            }

                            spawn_effect(
                                &mut cmds,
                                &mut effects,
                                &effect,
                                &query_effects,
                                &query_force_fields,
                            );
                            viewer_state.has_asset = true;
                            viewer_state.error_message = None;
                        }
                        Err(e) => {
                            error!("Failed to parse effect file {}: {}", file_path, e);
                            viewer_state.has_asset = false;
                            viewer_state.error_message = Some(format!("Parse error: {}", e));
                        }
                    },
                    Err(e) => {
                        error!("Failed to read effect file {}: {}", file_path, e);
                        viewer_state.has_asset = false;
                        viewer_state.error_message = Some(format!("File read error: {}", e));
                    }
                }
            }
        }
    }
}

fn spawn_effect(
    cmds: &mut Commands,
    effects: &mut Assets<EffectAsset>,
    asset: &EffectAsset,
    query_effects: &Query<Entity, With<ParticleEffectEntity>>,
    query_force_fields: &Query<Entity, With<ForceFieldSource>>,
)
{
    effects.clear();
    for entity in query_effects.iter() {
        cmds.entity(entity).despawn();
    }
    for entity in query_force_fields.iter() {
        cmds.entity(entity).despawn();
    }
    let handle = effects.add(asset.clone());
    cmds.spawn((
        ParticleEffectBundle {
            effect: ParticleEffect::new(handle).with_z_layer_2d(Some(0.0)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            visibility: Visibility::Visible,
            ..Default::default()
        },
        ParticleEffectEntity,
    ));
    for modifier in &asset.update_modifiers {
        if let Some(force_field_modifier) = modifier.as_any().downcast_ref::<ForceFieldModifier>() {
            for source in &force_field_modifier.sources {
                if source.mass > 0.0 {
                    cmds.spawn((
                        ForceFieldSource {
                            position:          source.position,
                            max_radius:        source.max_radius,
                            min_radius:        source.min_radius,
                            mass:              source.mass,
                            force_exponent:    source.force_exponent,
                            conform_to_sphere: source.conform_to_sphere,
                        },
                        Transform::from_translation(source.position),
                        Visibility::Visible,
                    ));
                }
            }
        }
    }
}

fn sys_setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
)
{
    let projection = OrthographicProjection {
        scale: 1.0,
        near: -200.0,
        far: 200.0,
        scaling_mode: ScalingMode::WindowSize(3.0),
        ..Default::default()
    };
    cmds.spawn((
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            projection: projection.clone(),
            camera: Camera {
                hdr: true,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb_u8(4, 4, 10)),
                ..default()
            },
            tonemapping: Tonemapping::None,
            ..default()
        },
        BloomSettings::default(),
        UiCameraConfig { show_ui: true },
    ));
    setup_ui(&mut cmds, &asset_server);
    setup_grid(&mut cmds, &mut meshes, &mut materials, &asset_server);
}

fn setup_ui(cmds: &mut Commands, asset_server: &AssetServer)
{
    let text_style = TextStyle {
        font_size: 14.0,
        color: Color::rgba(0.8, 0.8, 0.8, 0.6),
        font: asset_server.load(DEFAULT_FONT),
        ..default()
    };

    let text_style_small = TextStyle {
        font_size: 10.0,
        color: Color::rgba(0.8, 0.8, 0.8, 1.0),
        font: asset_server.load(DEFAULT_FONT),
        ..default()
    };

    let empty_state_style = TextStyle {
        font_size: 16.0,
        color: Color::rgba(0.6, 0.6, 0.6, 0.8),
        font: asset_server.load(DEFAULT_FONT),
        ..default()
    };

    let error_state_style = TextStyle {
        font_size: 14.0,
        color: Color::rgba(1.0, 0.3, 0.3, 0.9),
        font: asset_server.load(DEFAULT_FONT),
        ..default()
    };

    cmds.spawn((
        TextBundle::from_section("Particles: 0", text_style.clone()).with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        }),
        UiText::ParticleCount,
    ));

    cmds.spawn((
        TextBundle::from_section("BBox: 0x0", text_style.clone()).with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(30.0),
            ..default()
        }),
        UiText::Bbox,
    ));

    cmds.spawn((
        TextBundle::from_section("Spawner: Rate | Total: 0", text_style.clone()).with_style(
            Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(50.0),
                ..default()
            },
        ),
        UiText::SpawnerInfo,
    ));

    cmds.spawn((
        TextBundle::from_section("Time: 0.0s | Sin(time): 0.000", text_style.clone()).with_style(
            Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(70.0),
                ..default()
            },
        ),
        UiText::TimingInfo,
    ));

    cmds.spawn((
        TextBundle::from_section("FPS: --", text_style).with_style(Style {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        }),
        UiText::Fps,
    ));

    cmds.spawn((
        TextBundle::from_section("Controls:", text_style_small).with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        }),
        UiText::Controls,
    ));

    cmds.spawn((
        TextBundle::from_section("No effect loaded", empty_state_style).with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            bottom: Val::Px(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }),
        UiText::EmptyState,
    ));

    cmds.spawn((
        TextBundle::from_section("", error_state_style).with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            bottom: Val::Px(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }),
        UiText::ErrorState,
    ));

    cmds.spawn((
        ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(60.0),
                height: Val::Px(25.0),
                right: Val::Px(80.0),
                bottom: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb_u8(54, 57, 64).into(),
            border_color: Color::rgb_u8(100, 100, 100).into(),
            ..default()
        },
        ButtonType::LightBackground,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Gray",
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                font: asset_server.load(DEFAULT_FONT),
                ..default()
            },
        ));
    });

    cmds.spawn((
        ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(60.0),
                height: Val::Px(25.0),
                right: Val::Px(10.0),
                bottom: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb_u8(50, 50, 50).into(),
            border_color: Color::rgb_u8(100, 100, 100).into(),
            ..default()
        },
        ButtonType::DarkBackground,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Dark",
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                font: asset_server.load(DEFAULT_FONT),
                ..default()
            },
        ));
    });

    cmds.spawn((
        ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(60.0),
                height: Val::Px(25.0),
                right: Val::Px(150.0),
                bottom: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb_u8(60, 60, 60).into(),
            border_color: Color::rgb_u8(100, 100, 100).into(),
            ..default()
        },
        ButtonType::Gizmos,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Gizmos",
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                font: asset_server.load(DEFAULT_FONT),
                ..default()
            },
        ));
    });

    cmds.spawn((
        ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(60.0),
                height: Val::Px(25.0),
                right: Val::Px(220.0),
                bottom: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb_u8(60, 60, 60).into(),
            border_color: Color::rgb_u8(100, 100, 100).into(),
            ..default()
        },
        ButtonType::Grid,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Grid",
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                font: asset_server.load(DEFAULT_FONT),
                ..default()
            },
        ));
    });
}

fn setup_grid(
    cmds: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    asset_server: &AssetServer,
)
{
    let grid_material = materials.add(ColorMaterial::from(Color::rgba(0.3, 0.3, 0.3, 0.2)));
    let sub_grid_material = materials.add(ColorMaterial::from(Color::rgba(0.2, 0.2, 0.2, 0.15)));
    let line_mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(0.5, 2000.0))));
    let line_mesh_h = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(2000.0, 0.5))));
    let sub_line_mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(0.25, 2000.0))));
    let sub_line_mesh_h = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(2000.0, 0.25))));
    let grid_size = 32.0;
    let subdivisions = 4;
    let small_grid_size = grid_size / subdivisions as f32;
    let range = 1024.0;
    let mut x = -range;
    while x <= range {
        cmds.spawn((
            MaterialMesh2dBundle {
                mesh: line_mesh.clone().into(),
                material: grid_material.clone(),
                transform: Transform::from_xyz(x, 0.0, -1.0),
                visibility: Visibility::Hidden,
                ..default()
            },
            GridLine,
        ));
        x += grid_size;
    }
    let mut y = -range;
    while y <= range {
        cmds.spawn((
            MaterialMesh2dBundle {
                mesh: line_mesh_h.clone().into(),
                material: grid_material.clone(),
                transform: Transform::from_xyz(0.0, y, -1.0),
                visibility: Visibility::Hidden,
                ..default()
            },
            GridLine,
        ));
        y += grid_size;
    }
    let mut x = -range;
    while x <= range {
        if (x / grid_size).fract().abs() > 0.01 {
            cmds.spawn((
                MaterialMesh2dBundle {
                    mesh: sub_line_mesh.clone().into(),
                    material: sub_grid_material.clone(),
                    transform: Transform::from_xyz(x, 0.0, -1.0),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                GridLine,
            ));
        }
        x += small_grid_size;
    }
    let mut y = -range;
    while y <= range {
        if (y / grid_size).fract().abs() > 0.01 {
            cmds.spawn((
                MaterialMesh2dBundle {
                    mesh: sub_line_mesh_h.clone().into(),
                    material: sub_grid_material.clone(),
                    transform: Transform::from_xyz(0.0, y, -1.0),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                GridLine,
            ));
        }
        y += small_grid_size;
    }
    let label_text_style = TextStyle {
        font_size: gizmo_constants::GRID_LABEL_FONT_SIZE,
        color: Color::rgba(
            gizmo_constants::GRID_LABEL_COLOR.0,
            gizmo_constants::GRID_LABEL_COLOR.1,
            gizmo_constants::GRID_LABEL_COLOR.2,
            gizmo_constants::GRID_LABEL_COLOR.3,
        ),
        font: asset_server.load("Roboto.ttf"),
        ..default()
    };
    let mut label_x = -range;
    while label_x <= range {
        let mut label_y = -range;
        while label_y <= range {
            let label_text = format!("({:.0},{:.0})", label_x, label_y);
            let label_xyz = Vec3::new(
                label_x + gizmo_constants::GRID_LABEL_OFFSET_X,
                label_y + gizmo_constants::GRID_LABEL_OFFSET_Y,
                0.0,
            );
            cmds.spawn((
                Text2dBundle {
                    text: Text::from_section(label_text, label_text_style.clone())
                        .with_alignment(TextAlignment::Center),
                    transform: Transform {
                        translation: label_xyz,
                        scale: Vec3::splat(0.13),
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    ..default()
                },
                GridLabel,
            ));
            label_y += grid_size;
        }
        label_x += grid_size;
    }
}

#[derive(Resource)]
pub struct CameraController
{
    pub position: Vec2,
    pub zoom:     f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraController
{
    fn default() -> Self
    {
        Self {
            position: Vec2::ZERO,
            zoom:     0.8,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }
}

/// Settings for the particle editor viewer
#[derive(Resource)]
pub struct ViewerSettings
{
    // Visual elements
    pub show_grid:         bool,
    pub show_bbox:         bool,
    pub grid_size:         f32,
    pub grid_subdivisions: u32,

    // Stats display toggle
    pub show_stats: bool,

    // Stats layout
    pub stats_panel_width: f32,
    pub compact_stats:     bool,
}

impl Default for ViewerSettings
{
    fn default() -> Self
    {
        Self {
            show_grid:         false,
            show_bbox:         true,
            grid_size:         32.0,
            grid_subdivisions: 4,
            show_stats:        true,
            stats_panel_width: 250.0,
            compact_stats:     false,
        }
    }
}

// Statistics about the particle effect being displayed
#[derive(Resource)]
pub struct ParticleStats
{
    // Basic particle info
    pub particle_count: u32,
    pub bbox_size:      Vec2,

    // Spawner information
    pub total_spawned: u32,    // Total particles spawned since start
    pub spawner_type:  String, // "Once", "Rate", "Burst", etc.

    // Effect timing
    pub effect_age:         f32, // How long effect has been running (seconds)
    pub estimated_lifetime: f32, // Estimated particle lifetime (seconds)

    // Update timing
    pub last_update_time: f64, // When stats were last updated
}

impl Default for ParticleStats
{
    fn default() -> Self
    {
        Self {
            particle_count:     0,
            bbox_size:          Vec2::new(32.0, 32.0),
            total_spawned:      0,
            spawner_type:       String::from("Unknown"),
            effect_age:         0.0,
            estimated_lifetime: 5.0,
            last_update_time:   0.0,
        }
    }
}

/// Component marker for grid coordinate labels
#[derive(Component)]
pub struct GridLabel;

/// State for tracking bbox rendering to avoid recreation every frame
#[derive(Default)]
pub struct BboxState
{
    material: Option<Handle<ColorMaterial>>,
    meshes:   Option<BboxMeshes>,
}

/// Cached meshes for bbox rendering
struct BboxMeshes
{
    top:    Handle<Mesh>,
    bottom: Handle<Mesh>,
    left:   Handle<Mesh>,
    right:  Handle<Mesh>,
}

/// Creates a line mesh between two 2D points
pub fn create_line_mesh(start: Vec2, end: Vec2) -> Mesh
{
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineList);
    let vertices = vec![[start.x, start.y, 0.0], [end.x, end.y, 0.0]];
    let indices = vec![0u32, 1u32];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0]];
    let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh
}

/// System that manages visual bounding box rendering
pub fn sys_update_grid_gizmos(
    mut cmds: Commands,
    viewer_settings: Res<ViewerSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    visual_bbox_query: Query<Entity, With<VisualBbox>>,
    mut bbox_state: Local<BboxState>,
)
{
    if viewer_settings.show_bbox {
        if visual_bbox_query.is_empty() {
            if bbox_state.material.is_none() {
                let line_color = Color::rgba(
                    gizmo_constants::BBOX_COLOR.0,
                    gizmo_constants::BBOX_COLOR.1,
                    gizmo_constants::BBOX_COLOR.2,
                    gizmo_constants::BBOX_COLOR.3,
                );
                bbox_state.material = Some(materials.add(ColorMaterial::from(line_color)));
            }

            if bbox_state.meshes.is_none() {
                let half_size = gizmo_constants::BBOX_HALF_SIZE;

                let top_mesh = create_line_mesh(
                    Vec2::new(-half_size, half_size),
                    Vec2::new(half_size, half_size),
                );
                let bottom_mesh = create_line_mesh(
                    Vec2::new(-half_size, -half_size),
                    Vec2::new(half_size, -half_size),
                );
                let left_mesh = create_line_mesh(
                    Vec2::new(-half_size, -half_size),
                    Vec2::new(-half_size, half_size),
                );
                let right_mesh = create_line_mesh(
                    Vec2::new(half_size, -half_size),
                    Vec2::new(half_size, half_size),
                );

                let bbox_meshes = BboxMeshes {
                    top:    meshes.add(top_mesh),
                    bottom: meshes.add(bottom_mesh),
                    left:   meshes.add(left_mesh),
                    right:  meshes.add(right_mesh),
                };
                bbox_state.meshes = Some(bbox_meshes);
            }

            let material = bbox_state.material.as_ref().unwrap();
            let meshes = bbox_state.meshes.as_ref().unwrap();
            let parent = cmds
                .spawn((
                    SpatialBundle::from_transform(Transform::from_xyz(
                        0.0,
                        0.0,
                        gizmo_constants::BBOX_Z_ORDER,
                    )),
                    VisualBbox {
                        min: Vec3::new(
                            -gizmo_constants::BBOX_HALF_SIZE,
                            -gizmo_constants::BBOX_HALF_SIZE,
                            0.0,
                        ),
                        max: Vec3::new(
                            gizmo_constants::BBOX_HALF_SIZE,
                            gizmo_constants::BBOX_HALF_SIZE,
                            0.0,
                        ),
                    },
                ))
                .id();

            cmds.spawn((MaterialMesh2dBundle {
                mesh: meshes.top.clone().into(),
                material: material.clone(),
                ..default()
            },))
                .set_parent(parent);

            cmds.spawn((MaterialMesh2dBundle {
                mesh: meshes.bottom.clone().into(),
                material: material.clone(),
                ..default()
            },))
                .set_parent(parent);

            cmds.spawn((MaterialMesh2dBundle {
                mesh: meshes.left.clone().into(),
                material: material.clone(),
                ..default()
            },))
                .set_parent(parent);

            cmds.spawn((MaterialMesh2dBundle {
                mesh: meshes.right.clone().into(),
                material: material.clone(),
                ..default()
            },))
                .set_parent(parent);
        }
    } else {
        if !visual_bbox_query.is_empty() {
            for entity in visual_bbox_query.iter() {
                cmds.entity(entity).despawn_recursive();
            }
        }
    }
}

/// System to update background color based on UI button interactions
pub fn sys_update_background(
    mut camera_query: Query<&mut Camera2d>,
    background_color: Res<BackgroundColor>,
)
{
    if let Ok(mut camera) = camera_query.get_single_mut() {
        camera.clear_color = ClearColorConfig::Custom(background_color.color);
    }
}

/// System to handle background color button clicks
pub fn sys_handle_buttons(
    mut background_color: ResMut<BackgroundColor>,
    mut viewer_settings: ResMut<ViewerSettings>,
    button_query: Query<(&Interaction, &ButtonType), Changed<Interaction>>,
)
{
    for (interaction, button_type) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            match button_type {
                ButtonType::DarkBackground => {
                    *background_color = BackgroundColor::dark();
                }
                ButtonType::LightBackground => {
                    *background_color = BackgroundColor::light();
                }
                ButtonType::Gizmos => {
                    viewer_settings.show_bbox = !viewer_settings.show_bbox;
                }
                ButtonType::Grid => {
                    viewer_settings.show_grid = !viewer_settings.show_grid;
                }
            }
        }
    }
}

/// System to handle button hover effects
pub fn sys_button_hover_effects(
    mut button_query: Query<(&Interaction, &mut bevy::ui::BackgroundColor, &ButtonType)>,
)
{
    for (interaction, mut background_color, button_type) in button_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                // Lighter background color on hover
                match button_type {
                    ButtonType::DarkBackground => {
                        background_color.0 = Color::rgb_u8(70, 70, 70);
                    }
                    ButtonType::LightBackground => {
                        background_color.0 = Color::rgb_u8(74, 77, 84);
                    }
                    ButtonType::Gizmos => {
                        background_color.0 = Color::rgb_u8(80, 80, 80);
                    }
                    ButtonType::Grid => {
                        background_color.0 = Color::rgb_u8(80, 80, 80);
                    }
                }
            }
            Interaction::None => {
                // Reset to default background color
                match button_type {
                    ButtonType::DarkBackground => {
                        background_color.0 = Color::rgb_u8(50, 50, 50);
                    }
                    ButtonType::LightBackground => {
                        background_color.0 = Color::rgb_u8(54, 57, 64);
                    }
                    ButtonType::Gizmos => {
                        background_color.0 = Color::rgb_u8(60, 60, 60);
                    }
                    ButtonType::Grid => {
                        background_color.0 = Color::rgb_u8(60, 60, 60);
                    }
                }
            }
            Interaction::Pressed => {
                // Slightly darker when pressed
                match button_type {
                    ButtonType::DarkBackground => {
                        background_color.0 = Color::rgb_u8(40, 40, 40);
                    }
                    ButtonType::LightBackground => {
                        background_color.0 = Color::rgb_u8(44, 47, 54);
                    }
                    ButtonType::Gizmos => {
                        background_color.0 = Color::rgb_u8(50, 50, 50);
                    }
                    ButtonType::Grid => {
                        background_color.0 = Color::rgb_u8(50, 50, 50);
                    }
                }
            }
        }
    }
}

/// Constants for gizmo rendering
pub mod gizmo_constants
{
    pub const BBOX_SIZE: f32 = 32.0;
    pub const BBOX_HALF_SIZE: f32 = BBOX_SIZE / 2.0;
    pub const BBOX_COLOR: (f32, f32, f32, f32) = (0.0, 1.0, 0.3, 0.35);
    pub const BBOX_Z_ORDER: f32 = 1.0;

    pub const GRID_LABEL_FONT_SIZE: f32 = 15.0;
    pub const GRID_LABEL_SCALE: f32 = 0.13;
    pub const GRID_LABEL_COLOR: (f32, f32, f32, f32) = (0.6, 0.6, 0.6, 0.12);
    pub const GRID_LABEL_OFFSET_X: f32 = 7.0;
    pub const GRID_LABEL_OFFSET_Y: f32 = 2.5;
    pub const MAX_GRID_SIZE_FOR_LABELS: f32 = 200.0;
}

/// Background color for the viewer
#[derive(Resource)]
pub struct BackgroundColor
{
    pub color: Color,
}

impl Default for BackgroundColor
{
    fn default() -> Self
    {
        Self {
            color: Color::rgb_u8(4, 4, 10),
        }
    }
}

impl BackgroundColor
{
    pub fn dark() -> Self
    {
        Self {
            color: Color::rgb_u8(4, 4, 10),
        }
    }

    pub fn light() -> Self
    {
        Self {
            color: Color::rgb_u8(54, 57, 64),
        }
    }
}

/// System to render force field gizmos
pub fn sys_render_force_field_gizmos(
    viewer_settings: Res<ViewerSettings>,
    force_field_query: Query<&ForceFieldSource>,
    mut gizmos: Gizmos,
)
{
    if !viewer_settings.show_bbox {
        return;
    }

    for force_field in force_field_query.iter() {
        if force_field.mass.abs() <= 0.0 {
            continue;
        }

        let position = force_field.position.truncate();
        gizmos.circle_2d(
            position,
            force_field.max_radius,
            Color::rgba(0.8, 0.4, 1.0, 0.07),
        );

        if force_field.min_radius > 0.0 {
            gizmos.circle_2d(
                position,
                force_field.min_radius,
                Color::rgba(0.6, 0.2, 0.8, 0.07),
            );
        }
    }
}
