use std::collections::HashMap;
use std::path::PathBuf;

// ====================
// Deps.
// ====================
use bevy_hanabi::{EffectAsset, SimulationCondition, SimulationSpace};
// ====================
// GPUI.
// ====================
use gpui::prelude::*;
use gpui::{
    AnyElement,
    BoxShadow,
    Context,
    Entity,
    EventEmitter,
    IntoElement,
    ParentElement,
    Rgba,
    Styled,
    Subscription,
    Window,
    div,
    px,
};
use tracing::debug;

// ====================
// Editor.
// ====================
use crate::gui::asset_editor::error_panel::ErrorPanel;
use crate::gui::blocks::{KeyValueBlock, NewValueStrategy, ScalarBlock, SequenceBlock};
use crate::gui::inspectors::enumeration::EnumInspector;
use crate::gui::inspectors::float::FloatInspector;
use crate::gui::inspectors::force_field::ForceFieldSourceInspector;
use crate::gui::inspectors::init_modifier::InitModifierInspector;
use crate::gui::inspectors::integer::IntegerInspector;
use crate::gui::inspectors::render_modifier::RenderModifierInspector;
use crate::gui::inspectors::spawner::{SpawnerData, SpawnerInspector};
use crate::gui::inspectors::text::TextInspector;
use crate::gui::inspectors::time_color::{TimeColor, TimeColorInspector};
use crate::gui::inspectors::time_vec2::{TimeVec2, TimeVec2Inspector};
use crate::gui::inspectors::update_modifier::UpdateModifierInspector;
use crate::gui::inspectors::{Inspector, InspectorEvent};
use crate::gui::models::color::HdrColor;
use crate::gui::models::modifier::{XInitModifier, XRenderModifier, XUpdateModifier};
use crate::gui::models::state::{AssetState, ToHanabi};
use crate::gui::section::BlockSection;
use crate::gui::styling::colors::*;

/// Event emitted when the asset has been modified
#[derive(Clone)]
pub struct AssetUpdated
{
    pub effect_asset: EffectAsset,
}

/// State of the asset editor - either successfully loaded or showing an error
#[derive(Debug, Clone)]
enum EditorState
{
    Empty,
    Loaded,
    Error,
}

/// Asset editor panel for editing particle effect assets.
///
/// Displays and allows editing of particle effect properties
/// when an models file is selected in the file browser.
///
pub struct AssetEditor
{
    state:        Entity<AssetState>,
    editor_state: EditorState,
    error_panel:  Option<Entity<ErrorPanel>>,

    // Section expanded states - persisted across asset changes
    section_states: HashMap<String, bool>,

    // General section
    name:       Option<Entity<ScalarBlock<TextInspector>>>,
    capacity:   Option<Entity<ScalarBlock<IntegerInspector>>>,
    z_layer_2d: Option<Entity<ScalarBlock<FloatInspector>>>,
    sim_space:  Option<Entity<ScalarBlock<EnumInspector<SimulationSpace>>>>,
    sim_cond:   Option<Entity<ScalarBlock<EnumInspector<SimulationCondition>>>>,
    section_1:  Option<Entity<BlockSection>>,

    // Spawner properties.
    spawner:   Option<Entity<SpawnerInspector>>,
    section_2: Option<Entity<BlockSection>>,

    // Render Modifiers.
    size_over_time:   Option<Entity<SequenceBlock<TimeVec2Inspector>>>,
    color_over_time:  Option<Entity<SequenceBlock<TimeColorInspector>>>,
    render_modifiers: Option<Entity<SequenceBlock<RenderModifierInspector>>>,
    section_3:        Option<Entity<BlockSection>>,

    // Init Modifiers.
    section_4:      Option<Entity<BlockSection>>,
    init_modifiers: Option<Entity<SequenceBlock<InitModifierInspector>>>,

    // Update Modifiers.
    section_5:        Option<Entity<BlockSection>>,
    update_modifiers: Option<Entity<SequenceBlock<UpdateModifierInspector>>>,

    // Force Fields.
    section_6:    Option<Entity<BlockSection>>,
    force_fields: Option<Entity<SequenceBlock<ForceFieldSourceInspector>>>,

    // Properties.
    section_7:  Option<Entity<BlockSection>>,
    properties: Option<Entity<KeyValueBlock>>,

    // Event subscriptions
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<AssetUpdated> for AssetEditor {}

impl AssetEditor
{
    /// Create a new empty models editor.
    ///
    pub fn new(cx: &mut Context<Self>) -> Self
    {
        let state = cx.new(|_| AssetState::default());

        Self {
            state,
            editor_state: EditorState::Empty,
            error_panel: None,
            section_states: HashMap::new(),
            section_1: None,
            name: None,
            capacity: None,
            z_layer_2d: None,
            sim_space: None,
            sim_cond: None,
            section_2: None,
            spawner: None,
            size_over_time: None,
            color_over_time: None,
            render_modifiers: None,
            section_3: None,
            init_modifiers: None,
            section_4: None,
            update_modifiers: None,
            section_5: None,
            section_6: None,
            force_fields: None,
            properties: None,
            section_7: None,
            _subscriptions: Vec::new(),
        }
    }

    /// Get the expanded state for a section, defaulting to true if not set
    fn get_section_expanded_state(&self, section_name: &str) -> bool
    {
        self.section_states
            .get(section_name)
            .copied()
            .unwrap_or(true)
    }

    /// Helper method to convert current state to EffectAsset and emit AssetChanged event
    fn emit_asset_change(&mut self, cx: &mut Context<Self>)
    {
        match ToHanabi::effect_asset(self.state.read(cx)) {
            Ok(effect_asset) => {
                cx.emit(AssetUpdated { effect_asset });
            }
            Err(err) => {
                tracing::error!("Failed to convert state to EffectAsset: {}", err);
            }
        }
    }

    // ====================
    // State manipulation.
    // ====================

    /// Set the expanded state for a section
    fn set_section_expanded_state(&mut self, section_name: &str, expanded: bool)
    {
        self.section_states
            .insert(section_name.to_string(), expanded);
    }

    /// Save current section states before clearing
    fn set_section_states(&mut self, cx: &mut Context<Self>)
    {
        if let Some(section_1) = &self.section_1 {
            let expanded = section_1.read(cx).is_expanded();
            self.set_section_expanded_state("General Properties", expanded);
        }
        if let Some(section_2) = &self.section_2 {
            let expanded = section_2.read(cx).is_expanded();
            self.set_section_expanded_state("Spawner", expanded);
        }
        if let Some(section_3) = &self.section_3 {
            let expanded = section_3.read(cx).is_expanded();
            self.set_section_expanded_state("Render Modifiers", expanded);
        }
        if let Some(section_4) = &self.section_4 {
            let expanded = section_4.read(cx).is_expanded();
            self.set_section_expanded_state("Init Modifiers", expanded);
        }
        if let Some(section_5) = &self.section_5 {
            let expanded = section_5.read(cx).is_expanded();
            self.set_section_expanded_state("Update Modifiers", expanded);
        }
        if let Some(section_6) = &self.section_6 {
            let expanded = section_6.read(cx).is_expanded();
            self.set_section_expanded_state("Force Fields", expanded);
        }
        if let Some(section_7) = &self.section_7 {
            let expanded = section_7.read(cx).is_expanded();
            self.set_section_expanded_state("Properties", expanded);
        }
    }

    // ====================
    // Event handlers.
    // ====================

    /// Load asset state from buffer (in-memory version)
    pub fn on_buffer_state_selected(&mut self, asset_state: AssetState, cx: &mut Context<Self>)
    {
        // Clear previous state
        self.clear_editor_fields(cx);
        self.initialize_editor_from_state(&asset_state, cx);
        self.editor_state = EditorState::Loaded;

        self.state.update(cx, |state, _cx| {
            *state = asset_state;
        });

        cx.notify();
    }

    /// Show an error panel with the given file path and message
    pub fn show_error(&mut self, file_path: &PathBuf, message: String, cx: &mut Context<Self>)
    {
        self.editor_state = EditorState::Error;
        self.error_panel = Some(cx.new(|_| ErrorPanel::new(file_path, message.clone())));
    }

    /// Clear all editor fields
    fn clear_editor_fields(&mut self, cx: &mut Context<Self>)
    {
        // Save current section states before clearing
        self.set_section_states(cx);
        self._subscriptions.clear();

        self.section_1 = None;
        self.name = None;
        self.capacity = None;
        self.z_layer_2d = None;
        self.sim_space = None;
        self.sim_cond = None;

        self.section_2 = None;
        self.spawner = None;

        self.section_3 = None;
        self.size_over_time = None;
        self.color_over_time = None;
        self.render_modifiers = None;

        self.section_4 = None;
        self.init_modifiers = None;

        self.section_5 = None;
        self.update_modifiers = None;

        self.section_6 = None;
        self.force_fields = None;

        self.render_modifiers = None;

        self.section_7 = None;
        self.properties = None;

        self.error_panel = None;
    }

    /// Initialize all editor fields from the converted AssetState
    fn initialize_editor_from_state(&mut self, state: &AssetState, cx: &mut Context<Self>)
    {
        // Section 1 – General Properties
        self.name = Some(cx.new(|cx| {
            ScalarBlock::<TextInspector>::new("Name", state.name.clone(), cx).with_index(0)
        }));
        self.capacity = Some(cx.new(|cx| {
            ScalarBlock::<IntegerInspector>::new("Capacity", state.capacity, cx).with_index(1)
        }));
        self.z_layer_2d = Some(cx.new(|cx| {
            ScalarBlock::<FloatInspector>::new("Z Layer 2D", state.z_layer_2d, cx).with_index(2)
        }));
        self.sim_space = Some(cx.new(|cx| {
            ScalarBlock::<EnumInspector<SimulationSpace>>::new(
                "Sim. Space",
                state.simulation_space,
                cx,
            )
            .with_index(3)
        }));
        self.sim_cond = Some(cx.new(|cx| {
            ScalarBlock::<EnumInspector<SimulationCondition>>::new(
                "Sim. Cond",
                state.simulation_condition,
                cx,
            )
            .with_index(4)
        }));

        // Section 2 – Spawner Properties
        self.spawner = Some(cx.new(|cx| SpawnerInspector::new(cx, state.spawner.clone())));

        // Section 3 – Render Modifiers
        let size_data = if state.size_over_time.is_empty() {
            vec![TimeVec2::new(0.0, 8.0, 8.0)]
        } else {
            state
                .size_over_time
                .iter()
                .map(|(time, size)| TimeVec2::new(*time, size.x, size.y))
                .collect()
        };

        let color_data = if state.color_over_time.is_empty() {
            vec![TimeColor::new(0.0, HdrColor::default())]
        } else {
            state
                .color_over_time
                .iter()
                .map(|(time, color)| TimeColor::new(*time, color.clone()))
                .collect()
        };

        self.size_over_time =
            Some(cx.new(|cx| {
                SequenceBlock::<TimeVec2Inspector>::new("Size Over Time", size_data, cx)
            }));
        self.color_over_time = Some(cx.new(|cx| {
            SequenceBlock::<TimeColorInspector>::new("Color Over Time", color_data, cx)
                .with_new_val(NewValueStrategy::MostRecentOr(TimeColor::new(
                    1.0,
                    HdrColor::default(),
                )))
        }));

        // Create render modifiers
        let render_modifiers_data = if state.render_modifiers.is_empty() {
            vec![XRenderModifier::default()]
        } else {
            state.render_modifiers.clone()
        };
        self.render_modifiers =
            Some(cx.new(|cx| SequenceBlock::new("", render_modifiers_data, cx)));

        // Section 4 – Init Modifiers
        let init_modifiers_data = if state.init_modifiers.is_empty() {
            vec![XInitModifier::default()]
        } else {
            state.init_modifiers.clone()
        };
        self.init_modifiers = Some(cx.new(|cx| SequenceBlock::new("", init_modifiers_data, cx)));

        // Section 5 – Update Modifiers
        let update_modifiers_data = if state.update_modifiers.is_empty() {
            vec![XUpdateModifier::default()]
        } else {
            state.update_modifiers.clone()
        };
        self.update_modifiers =
            Some(cx.new(|cx| SequenceBlock::new("", update_modifiers_data, cx)));

        // Section 6 – Force Fields
        let force_fields_data = if state.force_fields.is_empty() {
            vec![crate::gui::models::modifier::XForceFieldSource::default()]
        } else {
            state.force_fields.clone()
        };
        self.force_fields = Some(cx.new(|cx| SequenceBlock::new("", force_fields_data, cx)));

        // Section 7 – Properties
        self.properties = Some(cx.new(|cx| KeyValueBlock::new("", state.properties.clone(), cx)));

        // Create section entities with saved expanded states
        self.section_1 = Some(cx.new(|cx| {
            BlockSection::new("General Properties", cx)
                .with_expanded(self.get_section_expanded_state("General Properties"))
        }));
        self.section_2 = Some(cx.new(|cx| {
            BlockSection::new("Spawner", cx)
                .with_expanded(self.get_section_expanded_state("Spawner"))
        }));
        self.section_3 = Some(cx.new(|cx| {
            BlockSection::new("Render Modifiers", cx)
                .with_expanded(self.get_section_expanded_state("Render Modifiers"))
        }));
        self.section_4 = Some(cx.new(|cx| {
            BlockSection::new("Init Modifiers", cx)
                .with_expanded(self.get_section_expanded_state("Init Modifiers"))
        }));
        self.section_5 = Some(cx.new(|cx| {
            BlockSection::new("Update Modifiers", cx)
                .with_expanded(self.get_section_expanded_state("Update Modifiers"))
        }));
        self.section_6 = Some(cx.new(|cx| {
            BlockSection::new("Force Fields", cx)
                .with_expanded(self.get_section_expanded_state("Force Fields"))
        }));
        self.section_7 = Some(cx.new(|cx| {
            BlockSection::new("Properties", cx)
                .with_expanded(self.get_section_expanded_state("Properties"))
        }));

        // Subscribe to all inspector events for debugging
        self.subscribe_to_all_events(cx);
    }

    /// Subscribe to events from all created entities
    fn subscribe_to_all_events(&mut self, cx: &mut Context<Self>)
    {
        // Section 1 – General Properties
        if let Some(ref name) = self.name {
            let subscription =
                cx.subscribe(name, |this, _entity, event: &InspectorEvent<String>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!("Inspector event from 'Name': {:?}", v);
                    this.state.update(cx, |state, _cx| {
                        state.name = v.clone();
                    });
                    this.emit_asset_change(cx);
                });
            self._subscriptions.push(subscription);
        }
        if let Some(ref capacity) = self.capacity {
            let subscription = cx.subscribe(
                capacity,
                |this, _entity, event: &InspectorEvent<i32>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!("Inspector event from 'Capacity': {:?}", v);
                    this.state.update(cx, |state, _cx| {
                        state.capacity = *v;
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }
        if let Some(ref z_layer_2d) = self.z_layer_2d {
            let subscription = cx.subscribe(
                z_layer_2d,
                |this, _entity, event: &InspectorEvent<f32>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!("[AssetEditor] Event from 'Z Layer 2D': {:?}", v);
                    this.state.update(cx, |state, _cx| {
                        state.z_layer_2d = *v;
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }
        if let Some(ref sim_space) = self.sim_space {
            let subscription = cx.subscribe(
                sim_space,
                |this, _entity, event: &InspectorEvent<SimulationSpace>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!("[AssetEditor] Event from 'Sim. Space': {:?}", v);
                    this.state.update(cx, |state, _cx| {
                        state.simulation_space = *v;
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }
        if let Some(ref sim_cond) = self.sim_cond {
            let subscription = cx.subscribe(
                sim_cond,
                |this, _entity, event: &InspectorEvent<SimulationCondition>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!("[AssetEditor] Event from 'Sim. Cond': {:?}", v);
                    this.state.update(cx, |state, _cx| {
                        state.simulation_condition = *v;
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }

        // Section 2 – Spawner Properties
        if let Some(ref spawner) = self.spawner {
            let subscription = cx.subscribe(
                spawner,
                |this, _entity, event: &InspectorEvent<SpawnerData>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!("[AssetEditor] Event from 'Spawner': {:?}", v);
                    this.state.update(cx, |state, _cx| {
                        state.spawner = v.clone();
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }

        // Section 3 – Render Modifiers
        if let Some(ref size_over_time) = self.size_over_time {
            let subscription = cx.subscribe(
                size_over_time,
                |this, _entity, event: &InspectorEvent<Vec<TimeVec2>>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!(
                        "[AssetEditor] Event from 'Size Over Time': {} items",
                        v.len()
                    );
                    this.state.update(cx, |state, _cx| {
                        state.size_over_time = v.iter().map(|tv| (tv.t, tv.v)).collect();
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }
        if let Some(ref color_over_time) = self.color_over_time {
            let subscription = cx.subscribe(
                color_over_time,
                |this, _entity, event: &InspectorEvent<Vec<TimeColor>>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!(
                        "[AssetEditor] Event from 'Color Over Time': {} items",
                        v.len()
                    );
                    this.state.update(cx, |state, _cx| {
                        state.color_over_time =
                            v.iter().map(|tc| (tc.t, tc.color.clone())).collect();
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }

        // Section 4 – Init Modifiers
        if let Some(ref init_modifiers) = self.init_modifiers {
            let subscription = cx.subscribe(
                init_modifiers,
                |this, _entity, event: &InspectorEvent<Vec<XInitModifier>>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!(
                        "[AssetEditor] Event from 'Init Modifiers': {} items",
                        v.len()
                    );
                    this.state.update(cx, |state, _cx| {
                        state.init_modifiers = v.clone();
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }

        // Section 5 – Update Modifiers
        if let Some(ref update_modifiers) = self.update_modifiers {
            let subscription = cx.subscribe(
                update_modifiers,
                |this, _entity, event: &InspectorEvent<Vec<XUpdateModifier>>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!(
                        "[AssetEditor] Event from 'Update Modifiers': {} items",
                        v.len()
                    );
                    this.state.update(cx, |state, _cx| {
                        state.update_modifiers = v.clone();
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }

        // Section 6 – Force Fields
        if let Some(ref force_fields) = self.force_fields {
            let subscription = cx.subscribe(
                force_fields,
                |this,
                 _entity,
                 event: &InspectorEvent<Vec<crate::gui::models::modifier::XForceFieldSource>>,
                 cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!("[AssetEditor] Event from 'Force Fields': {} items", v.len());
                    this.state.update(cx, |state, _cx| {
                        state.force_fields = v.clone();
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }

        // Section 3 – Render Modifiers
        if let Some(ref render_modifiers) = self.render_modifiers {
            let subscription = cx.subscribe(
                render_modifiers,
                |this, _entity, event: &InspectorEvent<Vec<XRenderModifier>>, cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!(
                        "[AssetEditor] Event from 'Render Modifiers': {} items",
                        v.len()
                    );
                    this.state.update(cx, |state, _cx| {
                        state.render_modifiers = v.clone();
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }

        // Section 7 – Properties
        if let Some(ref properties) = self.properties {
            let subscription = cx.subscribe(
                properties,
                |this,
                 _entity,
                 event: &InspectorEvent<Vec<crate::gui::models::key_value::KeyValueEntry>>,
                 cx| {
                    let InspectorEvent::Updated { v } = event;
                    debug!("[AssetEditor] Event from 'Properties': {} items", v.len());
                    this.state.update(cx, |state, _cx| {
                        state.properties = v.clone();
                    });
                    this.emit_asset_change(cx);
                },
            );
            self._subscriptions.push(subscription);
        }
    }

    // ====================
    // UI components.
    // ====================

    /// Utility functions.
    ///

    /// Render the properties panel.
    ///
    fn render_props_editor(&mut self, cx: &mut Context<Self>) -> impl IntoElement
    {
        let scrollable = div()
            .id("props-editor-scrollable-area")
            .size_full()
            .flex_1()
            .flex_shrink_0()
            .overflow_y_scroll();

        scrollable.child(
            div()
                .id("props-editor")
                .h(px(8000.0))
                .px_2()
                .py_2()
                .bg(Rgba {
                    r: background_primary().r,
                    g: background_primary().g,
                    b: background_primary().b,
                    a: 0.5,
                })
                .when(self.section_1.is_some(), |parent| {
                    // Update section children
                    if let Some(section_1) = &self.section_1 {
                        section_1.update(cx, |this, _cx| {
                            this.clear();
                            this.try_add_child(&self.name);
                            this.try_add_child(&self.capacity);
                            this.try_add_child(&self.z_layer_2d);
                            this.try_add_child(&self.sim_space);
                            this.try_add_child(&self.sim_cond);
                        });
                    }

                    if let Some(section_2) = &self.section_2 {
                        section_2.update(cx, |this, _cx| {
                            this.clear();
                            this.try_add_child(&self.spawner);
                        });
                    }

                    if let Some(section_3) = &self.section_3 {
                        section_3.update(cx, |this, _cx| {
                            this.clear();
                            this.try_add_child(&self.size_over_time);
                            this.try_add_child(&self.color_over_time);
                            this.try_add_child(&self.render_modifiers);
                        });
                    }

                    if let Some(section_4) = &self.section_4 {
                        section_4.update(cx, |this, _cx| {
                            this.clear();
                            this.try_add_child(&self.init_modifiers);
                        });
                    }

                    if let Some(section_5) = &self.section_5 {
                        section_5.update(cx, |this, _cx| {
                            this.clear();
                            this.try_add_child(&self.update_modifiers);
                        });
                    }

                    if let Some(section_6) = &self.section_6 {
                        section_6.update(cx, |this, _cx| {
                            this.clear();
                            this.try_add_child(&self.force_fields);
                        });
                    }

                    if let Some(section_7) = &self.section_7 {
                        section_7.update(cx, |this, _cx| {
                            this.clear();
                            this.try_add_child(&self.properties);
                        });
                    }

                    parent
                        .children(self.section_1.clone())
                        .children(self.section_2.clone())
                        .children(self.section_3.clone())
                        .children(self.section_4.clone())
                        .children(self.section_5.clone())
                        .children(self.section_6.clone())
                        .children(self.section_7.clone())
                }),
        )
    }
}

// ====================
// Rendering.
// ====================

impl Render for AssetEditor
{
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .id("models-editor")
            .size_full()
            .flex_1()
            .flex_shrink_0()
            .bg(panel_asset_editor())
            .child(match &self.editor_state {
                EditorState::Empty => self.render_empty_state(cx).into_any_element(),
                EditorState::Loaded => self.render_props_editor(cx).into_any_element(),
                EditorState::Error => self.render_error_panel(cx).into_any_element(),
            })
    }
}

impl AssetEditor
{
    /// Render the empty state when no asset is loaded
    fn render_empty_state(&self, _cx: &mut Context<Self>) -> impl IntoElement
    {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_start()
            .px_6()
            .pt_16()
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
                                color:         shadow_medium(),
                                offset:        gpui::point(px(0.), px(2.)),
                                blur_radius:   px(6.),
                                spread_radius: px(0.),
                            }])
                            .child(
                                div()
                                    .text_2xl()
                                    .text_color(text_secondary())
                                    .font_weight(gpui::FontWeight::LIGHT)
                                    .child("📄"),
                            ),
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(text_primary())
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_center()
                            .child("No Asset Selected"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(text_secondary())
                            .text_center()
                            .max_w(px(280.0))
                            .child("Select a particle effect asset file to begin editing."),
                    ),
            )
    }

    /// Render the error panel
    ///
    fn render_error_panel(&self, _cx: &mut Context<Self>) -> AnyElement
    {
        if let Some(error_panel) = &self.error_panel {
            div()
                .size_full()
                .child(error_panel.clone())
                .into_any_element()
        } else {
            self.render_empty_state(_cx).into_any_element()
        }
    }
}
