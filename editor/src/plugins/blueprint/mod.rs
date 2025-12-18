use crate::{
    fyrox::{
        asset::manager::ResourceManager,
        asset::ResourceData,
        core::{
            futures::executor::block_on,
            log::Log,
            make_relative_path,
            pool::{ErasedHandle, Handle, Pool},
            reflect::prelude::*,
            uuid::Uuid,
        },
        engine::Engine,
        graph::BaseSceneGraph,
        gui::{
            border::BorderBuilder,
            check_box::{CheckBoxBuilder, CheckBoxMessage},
            button::{ButtonBuilder, ButtonMessage},
            dock::{DockingManagerBuilder, DockingManagerMessage, TileBuilder, TileContent},
            dropdown_list::{DropdownListBuilder, DropdownListMessage},
            grid::{Column, GridBuilder, Row},
            list_view::{ListViewBuilder, ListViewMessage},
            message::{MessageDirection, MouseButton, UiMessage},
            numeric::{NumericUpDownBuilder, NumericUpDownMessage},
            popup::{Placement, PopupBuilder, PopupMessage},
            scroll_viewer::ScrollViewerBuilder,
            stack_panel::StackPanelBuilder,
            tab_control::{TabControlBuilder, TabControlMessage, TabDefinition},
            text::TextBuilder,
            text::TextMessage,
            text_box::{TextBoxBuilder},
            utils::make_dropdown_list_option,
            widget::{WidgetBuilder, WidgetMessage},
            window::{WindowBuilder, WindowMessage, WindowTitle},
            BuildContext, HorizontalAlignment, Orientation, Thickness, UiNode, UserInterface,
        },
    },
    plugin::EditorPlugin,
    Editor, Message,
};
use fyrox::gui::window::WindowAlignment;
use fyrox::gui::style::{resource::StyleResourceExt, Style};
use fyrox_blueprint::BlueprintAsset;
use fyrox_visual_scripting::{
    compile, BlueprintGraph, BuiltinNodeKind, DataType, GraphKind, Link, Node, NodeId, PinDirection,
    PinId, Value,
};
use fyrox_visual_scripting::model::VariableDef;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::plugins::absm::{
    canvas::{AbsmCanvas, AbsmCanvasBuilder, AbsmCanvasMessage},
    connection::ConnectionBuilder,
    node::{AbsmNodeBuilder, AbsmNodeLayout},
    socket::{Socket, SocketBuilder, SocketDirection},
};

#[derive(Clone, Debug, Reflect)]
struct BlueprintNodeModel {
    node_id: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BlueprintGraphTab {
    EventGraph,
    ConstructionScript,
}

fn tab_graph_name(tab: BlueprintGraphTab) -> &'static str {
    match tab {
        BlueprintGraphTab::EventGraph => "EventGraph",
        BlueprintGraphTab::ConstructionScript => "ConstructionScript",
    }
}

struct GraphView {
    canvas: Handle<UiNode>,
    models: Pool<BlueprintNodeModel>,

    node_views: HashMap<NodeId, Handle<UiNode>>,
    view_to_node: HashMap<Handle<UiNode>, NodeId>,
    pin_to_socket: HashMap<PinId, Handle<UiNode>>,
    socket_to_pin: HashMap<Handle<UiNode>, PinId>,
    pin_to_node: HashMap<PinId, NodeId>,
    node_primary_text_box_by_node: HashMap<NodeId, Handle<UiNode>>,
    node_text_box_binding: HashMap<Handle<UiNode>, (NodeId, String)>,
    node_value_binding: HashMap<Handle<UiNode>, (NodeId, String, DataType)>,
    connection_views: Vec<Handle<UiNode>>,
    node_view_handles: Vec<Handle<UiNode>>,
}

impl GraphView {
    fn new(canvas: Handle<UiNode>) -> Self {
        Self {
            canvas,
            models: Pool::new(),
            node_views: HashMap::new(),
            view_to_node: HashMap::new(),
            pin_to_socket: HashMap::new(),
            socket_to_pin: HashMap::new(),
            pin_to_node: HashMap::new(),
            node_primary_text_box_by_node: HashMap::new(),
            node_text_box_binding: HashMap::new(),
            node_value_binding: HashMap::new(),
            connection_views: Vec::new(),
            node_view_handles: Vec::new(),
        }
    }

    fn clear_ui(&mut self, ui: &UserInterface) {
        for h in self
            .connection_views
            .iter()
            .chain(self.node_view_handles.iter())
            .cloned()
        {
            ui.send(h, WidgetMessage::Remove);
        }

        self.models.clear();
        self.node_views.clear();
        self.view_to_node.clear();
        self.pin_to_socket.clear();
        self.socket_to_pin.clear();
        self.pin_to_node.clear();
        self.node_primary_text_box_by_node.clear();
        self.node_text_box_binding.clear();
        self.node_value_binding.clear();
        self.connection_views.clear();
        self.node_view_handles.clear();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum DetailsBinding {
    NodeProp { node: NodeId, key: &'static str },
    VariableName { index: usize },
    VariableType { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActionMenuAction {
    SpawnBuiltin(BuiltinNodeKind),
    SpawnGetVariable(usize),
    SpawnSetVariable(usize),
}

#[derive(Debug, Clone)]
struct PendingConnection {
    from: PinId,
    from_dir: PinDirection,
    from_type: DataType,
    graph_name: String,
}

struct BlueprintEditor {
    window: fyrox::core::pool::Handle<UiNode>,
    save: fyrox::core::pool::Handle<UiNode>,
    tab_control: fyrox::core::pool::Handle<UiNode>,

    my_blueprint_graphs_event: fyrox::core::pool::Handle<UiNode>,
    my_blueprint_graphs_construction: fyrox::core::pool::Handle<UiNode>,
    my_blueprint_new_graph: fyrox::core::pool::Handle<UiNode>,
    my_blueprint_graphs_panel: fyrox::core::pool::Handle<UiNode>,
    my_blueprint_graph_widgets: Vec<Handle<UiNode>>,
    my_blueprint_graph_select: HashMap<Handle<UiNode>, usize>,
    my_blueprint_new_variable: fyrox::core::pool::Handle<UiNode>,
    my_blueprint_variables_panel: fyrox::core::pool::Handle<UiNode>,
    my_blueprint_variable_widgets: Vec<Handle<UiNode>>,
    my_blueprint_variable_select: HashMap<Handle<UiNode>, usize>,
    my_blueprint_variable_get: HashMap<Handle<UiNode>, usize>,
    my_blueprint_variable_set: HashMap<Handle<UiNode>, usize>,

    my_blueprint_new_function: fyrox::core::pool::Handle<UiNode>,
    my_blueprint_functions_panel: fyrox::core::pool::Handle<UiNode>,
    my_blueprint_function_widgets: Vec<Handle<UiNode>>,
    my_blueprint_function_select: HashMap<Handle<UiNode>, usize>,

    details_panel: fyrox::core::pool::Handle<UiNode>,
    details_widgets: Vec<Handle<UiNode>>,
    details_bindings: HashMap<Handle<UiNode>, DetailsBinding>,
    selected_node: Option<NodeId>,
    selected_variable: Option<usize>,

    node_palette_buttons: HashMap<Handle<UiNode>, BuiltinNodeKind>,

    action_menu: Handle<UiNode>,
    action_menu_search: Handle<UiNode>,
    action_menu_list: Handle<UiNode>,
    action_menu_button_actions: HashMap<Handle<UiNode>, ActionMenuAction>,
    action_menu_spawn_graph: Option<String>,
    action_menu_spawn_position: Option<fyrox::core::algebra::Vector2<f32>>,

    pending_connection: Option<PendingConnection>,

    active_tab: BlueprintGraphTab,
    active_extra_tab: Option<usize>,

    path: Option<PathBuf>,
    version: u32,
    graph: BlueprintGraph,

    event_view: GraphView,
    construction_view: GraphView,

    extra_tabs: Vec<ExtraTab>,
}

struct ExtraTab {
    uuid: Uuid,
    name: String,
    kind: GraphKind,
    view: GraphView,
}

impl BlueprintEditor {
    fn active_graph_name(&self) -> &str {
        if let Some(extra) = self.active_extra_tab.and_then(|i| self.extra_tabs.get(i)) {
            return extra.name.as_str();
        }
        tab_graph_name(self.active_tab)
    }

    fn new(engine: &mut Engine) -> Self {
        let ctx = &mut engine.user_interfaces.first_mut().build_ctx();

        let mut node_palette_buttons = HashMap::new();

        let save;
        let my_blueprint_graphs_event;
        let my_blueprint_graphs_construction;
        let my_blueprint_new_graph;
        let my_blueprint_graphs_panel;
        let my_blueprint_new_variable;
        let my_blueprint_variables_panel;
        let my_blueprint_new_function;
        let my_blueprint_functions_panel;
        let details_panel;

        let action_menu;
        let action_menu_search;
        let action_menu_list;

        let event_canvas;
        let construction_canvas;

        let tab_control;

        let toolbar = StackPanelBuilder::new(
            WidgetBuilder::new()
                .on_row(0)
                .with_margin(Thickness::uniform(2.0))
                .with_horizontal_alignment(HorizontalAlignment::Right)
                .with_child({
                    save = ButtonBuilder::new(WidgetBuilder::new().with_width(120.0).with_height(24.0))
                        .with_text("Save")
                        .build(ctx);
                    save
                }),
        )
        .with_orientation(Orientation::Horizontal)
        .build(ctx);

        let components_window = WindowBuilder::new(WidgetBuilder::new().with_width(260.0).with_height(260.0))
            .can_close(false)
            .can_minimize(false)
            .with_title(WindowTitle::text("Components"))
            .with_content(
                TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(4.0)))
                    .with_text("(MVP) Components panel")
                    .build(ctx),
            )
            .build(ctx);

        let my_blueprint_window = WindowBuilder::new(WidgetBuilder::new().with_width(260.0).with_height(340.0))
            .can_close(false)
            .can_minimize(false)
            .with_title(WindowTitle::text("My Blueprint"))
            .with_content(
                StackPanelBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::uniform(4.0))
                        .with_child(
                            TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
                                .with_text("GRAPHS")
                                .build(ctx),
                        )
                        .with_child({
                            my_blueprint_new_graph = ButtonBuilder::new(
                                WidgetBuilder::new().with_height(24.0),
                            )
                            .with_text("+ Graph")
                            .build(ctx);
                            my_blueprint_new_graph
                        })
                        .with_child({
                            my_blueprint_graphs_event = ButtonBuilder::new(
                                WidgetBuilder::new().with_height(24.0),
                            )
                            .with_text("EventGraph")
                            .build(ctx);
                            my_blueprint_graphs_event
                        })
                        .with_child({
                            my_blueprint_graphs_construction = ButtonBuilder::new(
                                WidgetBuilder::new().with_height(24.0),
                            )
                            .with_text("ConstructionScript")
                            .build(ctx);
                            my_blueprint_graphs_construction
                        })
                        .with_child({
                            my_blueprint_graphs_panel = StackPanelBuilder::new(
                                WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                            )
                            .with_orientation(Orientation::Vertical)
                            .build(ctx);
                            my_blueprint_graphs_panel
                        })
                        .with_child(
                            TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(6.0)))
                                .with_text("VARIABLES")
                                .build(ctx),
                        )
                        .with_child({
                            my_blueprint_new_variable = ButtonBuilder::new(
                                WidgetBuilder::new().with_height(24.0),
                            )
                            .with_text("+ Variable")
                            .build(ctx);
                            my_blueprint_new_variable
                        })
                        .with_child({
                            my_blueprint_variables_panel = StackPanelBuilder::new(
                                WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                            )
                            .with_orientation(Orientation::Vertical)
                            .build(ctx);
                            my_blueprint_variables_panel
                        })
                        .with_child(
                            TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(6.0)))
                                .with_text("WORLD NODES")
                                .build(ctx),
                        )
                        .with_child({
                            let b = ButtonBuilder::new(WidgetBuilder::new().with_height(24.0))
                                .with_text("Self")
                                .build(ctx);
                            node_palette_buttons.insert(b, BuiltinNodeKind::Self_);
                            b
                        })
                        .with_child({
                            let b = ButtonBuilder::new(WidgetBuilder::new().with_height(24.0))
                                .with_text("Get Actor Transform")
                                .build(ctx);
                            node_palette_buttons.insert(b, BuiltinNodeKind::GetActorTransform);
                            b
                        })
                        .with_child({
                            let b = ButtonBuilder::new(WidgetBuilder::new().with_height(24.0))
                                .with_text("Set Actor Transform")
                                .build(ctx);
                            node_palette_buttons.insert(b, BuiltinNodeKind::SetActorTransform);
                            b
                        })
                        .with_child({
                            let b = ButtonBuilder::new(WidgetBuilder::new().with_height(24.0))
                                .with_text("Spawn Actor")
                                .build(ctx);
                            node_palette_buttons.insert(b, BuiltinNodeKind::SpawnActor);
                            b
                        })
                        .with_child({
                            let b = ButtonBuilder::new(WidgetBuilder::new().with_height(24.0))
                                .with_text("Get Actor By Name")
                                .build(ctx);
                            node_palette_buttons.insert(b, BuiltinNodeKind::GetActorByName);
                            b
                        })
                        .with_child({
                            let b = ButtonBuilder::new(WidgetBuilder::new().with_height(24.0))
                                .with_text("Get Actor Name")
                                .build(ctx);
                            node_palette_buttons.insert(b, BuiltinNodeKind::GetActorName);
                            b
                        })
                        .with_child(
                            TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(6.0)))
                                .with_text("FUNCTIONS")
                                .build(ctx),
                        )
                        .with_child({
                            my_blueprint_new_function = ButtonBuilder::new(
                                WidgetBuilder::new().with_height(24.0),
                            )
                            .with_text("+ Function")
                            .build(ctx);
                            my_blueprint_new_function
                        })
                        .with_child({
                            my_blueprint_functions_panel = StackPanelBuilder::new(
                                WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                            )
                            .with_orientation(Orientation::Vertical)
                            .build(ctx);
                            my_blueprint_functions_panel
                        }),
                )
                .with_orientation(Orientation::Vertical)
                .build(ctx),
            )
            .build(ctx);

        event_canvas =
            AbsmCanvasBuilder::new(WidgetBuilder::new().with_allow_drop(true)).build(ctx);
        construction_canvas =
            AbsmCanvasBuilder::new(WidgetBuilder::new().with_allow_drop(true)).build(ctx);

        tab_control = TabControlBuilder::new(WidgetBuilder::new())
            .with_tab(make_tab("Event Graph", event_canvas, ctx))
            .with_tab(make_tab("Construction Script", construction_canvas, ctx))
            .build(ctx);

        let graph_window = WindowBuilder::new(WidgetBuilder::new())
            .can_close(false)
            .can_minimize(false)
            .with_title(WindowTitle::text("Graph"))
            .with_content(tab_control)
            .build(ctx);

        details_panel = StackPanelBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(4.0)))
            .build(ctx);

        let details_window = WindowBuilder::new(WidgetBuilder::new().with_width(320.0))
            .can_close(false)
            .can_minimize(false)
            .with_title(WindowTitle::text("Details"))
            .with_content(
                ScrollViewerBuilder::new(WidgetBuilder::new())
                    .with_content(details_panel)
                    .build(ctx),
            )
            .build(ctx);

        // Unreal-like right-click action menu (search + list of node actions).
        action_menu = {
            let content = BorderBuilder::new(
                WidgetBuilder::new()
                    .with_width(340.0)
                    .with_max_size(fyrox::core::algebra::Vector2::new(340.0, 420.0))
                    .with_background(ctx.style.property(Style::BRUSH_DARKER))
                    .with_foreground(ctx.style.property(Style::BRUSH_LIGHT))
                    .with_child(
                        GridBuilder::new(
                            WidgetBuilder::new()
                                .with_margin(Thickness::uniform(6.0))
                                .with_child({
                                    action_menu_search = TextBoxBuilder::new(
                                        WidgetBuilder::new()
                                            .on_row(0)
                                            .with_height(24.0),
                                    )
                                    .with_text("")
                                    .build(ctx);
                                    action_menu_search
                                })
                                .with_child({
                                    action_menu_list = ListViewBuilder::new(
                                        WidgetBuilder::new()
                                            .on_row(1)
                                            .with_min_size(fyrox::core::algebra::Vector2::new(
                                                0.0,
                                                260.0,
                                            )),
                                    )
                                    .build(ctx);
                                    action_menu_list
                                }),
                        )
                        .add_row(Row::auto())
                        .add_row(Row::stretch())
                        .add_column(Column::stretch())
                        .build(ctx),
                    ),
            )
            .with_pad_by_corner_radius(false)
            .build(ctx);

            PopupBuilder::new(WidgetBuilder::new().with_visibility(false))
                .with_content(content)
                .build(ctx)
        };

        let docking_manager = DockingManagerBuilder::new(WidgetBuilder::new().on_row(1).with_child({
            TileBuilder::new(WidgetBuilder::new())
                .with_content(TileContent::HorizontalTiles {
                    splitter: 0.75,
                    tiles: [
                        TileBuilder::new(WidgetBuilder::new())
                            .with_content(TileContent::VerticalTiles {
                                splitter: 0.45,
                                tiles: [
                                    TileBuilder::new(WidgetBuilder::new())
                                        .with_content(TileContent::Window(components_window))
                                        .build(ctx),
                                    TileBuilder::new(WidgetBuilder::new())
                                        .with_content(TileContent::Window(my_blueprint_window))
                                        .build(ctx),
                                ],
                            })
                            .build(ctx),
                        TileBuilder::new(WidgetBuilder::new())
                            .with_content(TileContent::HorizontalTiles {
                                splitter: 0.70,
                                tiles: [
                                    TileBuilder::new(WidgetBuilder::new())
                                        .with_content(TileContent::Window(graph_window))
                                        .build(ctx),
                                    TileBuilder::new(WidgetBuilder::new())
                                        .with_content(TileContent::Window(details_window))
                                        .build(ctx),
                                ],
                            })
                            .build(ctx),
                    ],
                })
                .build(ctx)
        }))
        .build(ctx);

        let content = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(toolbar)
                .with_child(docking_manager),
        )
        .add_row(Row::auto())
        .add_row(Row::stretch())
        .add_column(Column::stretch())
        .build(ctx);

        let window = WindowBuilder::new(WidgetBuilder::new().with_width(900.0).with_height(600.0))
            .with_title(WindowTitle::text("Blueprint"))
            .open(false)
            .with_content(content)
            .build(ctx);

        Self {
            window,
            save,
            tab_control,

            my_blueprint_graphs_event,
            my_blueprint_graphs_construction,
            my_blueprint_new_graph,
            my_blueprint_graphs_panel,
            my_blueprint_graph_widgets: Vec::new(),
            my_blueprint_graph_select: HashMap::new(),
            my_blueprint_new_variable,
            my_blueprint_variables_panel,
            my_blueprint_variable_widgets: Vec::new(),
            my_blueprint_variable_select: HashMap::new(),
            my_blueprint_variable_get: HashMap::new(),
            my_blueprint_variable_set: HashMap::new(),

            my_blueprint_new_function,
            my_blueprint_functions_panel,
            my_blueprint_function_widgets: Vec::new(),
            my_blueprint_function_select: HashMap::new(),

            details_panel,
            details_widgets: Vec::new(),
            details_bindings: HashMap::new(),
            selected_node: None,
            selected_variable: None,

            node_palette_buttons,

            action_menu,
            action_menu_search,
            action_menu_list,
            action_menu_button_actions: HashMap::new(),
            action_menu_spawn_graph: None,
            action_menu_spawn_position: None,

            pending_connection: None,

            active_tab: BlueprintGraphTab::EventGraph,
            active_extra_tab: None,

            path: None,
            version: 1,
            graph: BlueprintGraph::new(fyrox_visual_scripting::GraphId("Blueprint".to_string())),

            event_view: GraphView::new(event_canvas),
            construction_view: GraphView::new(construction_canvas),

            extra_tabs: Vec::new(),
        }
    }

    fn rebuild_action_menu_items(&mut self, ui: &mut UserInterface, filter: &str) {
        self.action_menu_button_actions.clear();

        let needle = filter.trim().to_lowercase();
        let mut entries: Vec<(String, ActionMenuAction)> = Vec::new();

        // Built-in nodes.
        let builtins: &[(BuiltinNodeKind, &str)] = &[
            (BuiltinNodeKind::BeginPlay, "BeginPlay"),
            (BuiltinNodeKind::Tick, "Tick"),
            (BuiltinNodeKind::ConstructionScript, "Construction Script"),
            (BuiltinNodeKind::Print, "Print"),
            (BuiltinNodeKind::RhaiScript, "Rhai Script"),
            (BuiltinNodeKind::Branch, "Branch"),
            (BuiltinNodeKind::Self_, "Self"),
            (BuiltinNodeKind::GetActorTransform, "Get Actor Transform"),
            (BuiltinNodeKind::SetActorTransform, "Set Actor Transform"),
            (BuiltinNodeKind::SpawnActor, "Spawn Actor"),
            (BuiltinNodeKind::GetActorByName, "Get Actor By Name"),
            (BuiltinNodeKind::GetActorName, "Get Actor Name"),
        ];
        for (kind, label) in builtins.iter().copied() {
            entries.push((label.to_string(), ActionMenuAction::SpawnBuiltin(kind)));
        }

        // Variable shortcuts.
        for (i, var) in self.graph.variables.iter().enumerate() {
            entries.push((
                format!("Get {}", var.name),
                ActionMenuAction::SpawnGetVariable(i),
            ));
            entries.push((
                format!("Set {}", var.name),
                ActionMenuAction::SpawnSetVariable(i),
            ));
        }

        if !needle.is_empty() {
            entries.retain(|(label, _)| label.to_lowercase().contains(&needle));
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut item_widgets = Vec::new();
        if entries.is_empty() {
            let t = TextBuilder::new(
                WidgetBuilder::new()
                    .with_height(22.0)
                    .with_margin(Thickness::uniform(4.0)),
            )
            .with_text("No results")
            .build(&mut ui.build_ctx());
            item_widgets.push(t);
        } else {
            for (label, action) in entries {
                let b = ButtonBuilder::new(
                    WidgetBuilder::new()
                        .with_height(22.0)
                        .with_margin(Thickness::uniform(2.0)),
                )
                .with_text(&label)
                .build(&mut ui.build_ctx());
                self.action_menu_button_actions.insert(b, action);
                item_widgets.push(b);
            }
        }

        ui.send(self.action_menu_list, ListViewMessage::Items(item_widgets));
    }

    fn open_action_menu(&mut self, ui: &mut UserInterface, canvas: Handle<UiNode>, graph: String) {
        let Some(canvas_ref) = ui.node(canvas).query_component::<AbsmCanvas>() else {
            return;
        };

        self.action_menu_spawn_graph = Some(graph);
        self.action_menu_spawn_position = Some(canvas_ref.point_to_local_space(ui.cursor_position()));

        ui.send(self.action_menu_search, TextMessage::Text(String::new()));
        self.rebuild_action_menu_items(ui, "");

        ui.send(self.action_menu, PopupMessage::Placement(Placement::Cursor(canvas)));
        ui.send(self.action_menu, PopupMessage::Open);
        ui.send(self.action_menu_search, WidgetMessage::Focus);
    }

    fn spawn_world_node_at(
        &mut self,
        ui: &mut UserInterface,
        kind: BuiltinNodeKind,
        graph_name: &str,
        pos: fyrox::core::algebra::Vector2<f32>,
    ) {
        let mut n = Node::new(kind);
        n.graph = graph_name.to_string();
        n.position = [pos.x, pos.y];
        let node_id = self.graph.add_node(n);

        self.rebuild_all_graph_views(ui);
        self.set_selected_node(ui, Some(node_id));
    }

    fn try_apply_pending_connection(&mut self, new_node_id: NodeId, graph_name: &str) {
        let Some(pending) = self.pending_connection.take() else {
            return;
        };

        if pending.graph_name != graph_name {
            return;
        }

        let Some(new_node) = self.graph.nodes.get(&new_node_id) else {
            return;
        };

        // Prefer conventional pin names for exec.
        let mut target: Option<PinId> = None;
        if pending.from_type == DataType::Exec {
            let preferred = match pending.from_dir {
                PinDirection::Output => "exec",
                PinDirection::Input => "then",
            };
            if let Some(pin_id) = new_node.pin_named(preferred) {
                if let Some(p) = self.graph.pin(pin_id) {
                    let ty = self.get_actual_pin_type(pin_id).unwrap_or(p.data_type);
                    if p.direction != pending.from_dir && ty == pending.from_type {
                        target = Some(pin_id);
                    }
                }
            }
        }

        if target.is_none() {
            for pin in new_node.pins.iter() {
                let Some(p) = self.graph.pin(pin.id) else {
                    continue;
                };
                let ty = self.get_actual_pin_type(pin.id).unwrap_or(p.data_type);
                if p.direction != pending.from_dir && ty == pending.from_type {
                    target = Some(pin.id);
                    break;
                }
            }
        }

        let Some(target) = target else {
            return;
        };

        let (from, to) = match pending.from_dir {
            PinDirection::Output => (pending.from, target),
            PinDirection::Input => (target, pending.from),
        };

        let (Some(from_pin), Some(to_pin)) = (self.graph.pin(from), self.graph.pin(to)) else {
            return;
        };

        let from_data_type = self.get_actual_pin_type(from).unwrap_or(from_pin.data_type);
        let to_data_type = self.get_actual_pin_type(to).unwrap_or(to_pin.data_type);

        if from_pin.direction != PinDirection::Output
            || to_pin.direction != PinDirection::Input
            || from_data_type != to_data_type
        {
            return;
        }

        self.apply_connection(from, to);
    }

    fn close_all_extra_tabs(&mut self, ui: &mut UserInterface) {
        self.active_extra_tab = None;

        for mut tab in self.extra_tabs.drain(..) {
            tab.view.clear_ui(ui);
            ui.send(self.tab_control, TabControlMessage::RemoveTabByUuid(tab.uuid));
        }
    }

    fn open(&mut self, editor: &mut Editor, path: PathBuf) {
        self.path = Some(path);

        if let Some(path) = self.path.as_ref() {
            let title = path
                .file_name()
                .map(|n| format!("Blueprint [UX-2025-12-16] - {}", n.to_string_lossy()))
                .unwrap_or_else(|| "Blueprint [UX-2025-12-16]".to_string());

            let ui = editor.engine.user_interfaces.first_mut();
            ui.send(
                self.window,
                WindowMessage::Title(WindowTitle::text(title)),
            );

            self.reload_from_resource(&editor.engine.resource_manager, ui);
        }

        let ui = editor.engine.user_interfaces.first_mut();
        ui.send(
            self.window,
            WindowMessage::Open {
                alignment: WindowAlignment::Center,
                modal: false,
                focus_content: true,
            },
        );
        ui.send(
            editor.docking_manager,
            DockingManagerMessage::AddFloatingWindow(self.window),
        );
    }

    fn reload_from_resource(&mut self, resource_manager: &ResourceManager, ui: &mut UserInterface) {
        // Extra tabs belong to a single opened blueprint; discard them on reload.
        self.close_all_extra_tabs(ui);

        let Some(path) = self.path.as_ref() else {
            return;
        };

        let Ok(relative) = make_relative_path(path) else {
            Log::err(format!(
                "BlueprintEditor: path is outside registry: {}",
                path.display()
            ));
            return;
        };

        match block_on(resource_manager.request::<BlueprintAsset>(relative)) {
            Ok(resource) => {
                let guard = resource.data_ref();
                if let Some(asset) = guard.as_loaded_ref() {
                    self.version = asset.version;

                    match serde_json::from_str::<BlueprintGraph>(&asset.graph_json) {
                        Ok(graph) => {
                            self.graph = graph;
                        }
                        Err(err) => {
                            Log::err(format!(
                                "BlueprintEditor: invalid graph JSON in asset: {err}"
                            ));
                            self.graph = BlueprintGraph::new(
                                fyrox_visual_scripting::GraphId("Blueprint".to_string()),
                            );
                        }
                    }

                    self.graph.ensure_builtin_graphs();

                    if self.graph.nodes.is_empty() {
                        self.seed_default_graph();
                    }

                    self.sync_variable_node_pin_types();

                    self.rebuild_all_graph_views(ui);
                    self.rebuild_graphs_panel(ui);
                    self.rebuild_variables_panel(ui);
                    self.rebuild_functions_panel(ui);
                    self.set_selected_node(ui, None);
                } else {
                    Log::err("BlueprintEditor: blueprint asset is not loaded".to_string());
                }
            }
            Err(err) => Log::err(format!("BlueprintEditor: failed to load asset: {err:?}")),
        }
    }

    fn seed_default_graph(&mut self) {
        let begin = {
            let mut n = Node::new(BuiltinNodeKind::BeginPlay);
            n.graph = tab_graph_name(BlueprintGraphTab::EventGraph).to_string();
            n.position = [50.0, 50.0];
            self.graph.add_node(n)
        };

        let print = {
            let mut n = Node::new(BuiltinNodeKind::Print);
            n.graph = tab_graph_name(BlueprintGraphTab::EventGraph).to_string();
            n.position = [350.0, 50.0];
            n.set_property_string("text", "Hello".to_string());
            self.graph.add_node(n)
        };

        let then = self
            .graph
            .nodes
            .get(&begin)
            .and_then(|n| n.pin_named("then"))
            .unwrap();
        let exec = self
            .graph
            .nodes
            .get(&print)
            .and_then(|n| n.pin_named("exec"))
            .unwrap();

        self.graph.links.push(Link::exec(then, exec));
    }

    /// Get the actual data type of a pin, considering dynamic typing for variable nodes
    fn get_actual_pin_type(&self, pin_id: PinId) -> Option<DataType> {
        let pin = self.graph.pin(pin_id)?;
        let node_id = self.graph.pin_owner(pin_id)?;
        let node = self.graph.nodes.get(&node_id)?;

        // For variable nodes, determine type from the referenced variable
        match node.kind {
            BuiltinNodeKind::GetVariable | BuiltinNodeKind::SetVariable => {
                if pin.name == "value" {
                    node.properties
                        .get("name")
                        .and_then(|v| match v {
                            Value::String(var_name) => self
                                .graph
                                .variables
                                .iter()
                                .find(|var| var.name == *var_name)
                                .map(|var| var.data_type),
                            _ => None,
                        })
                        .or(Some(pin.data_type))
                } else {
                    Some(pin.data_type)
                }
            }
            _ => Some(pin.data_type),
        }
    }

    fn try_resolve_connection(
        &self,
        a: PinId,
        b: PinId,
        expected_graph: &str,
    ) -> Option<(PinId, PinId)> {
        let from_pin_id = a;
        let to_pin_id = b;
        if from_pin_id == to_pin_id {
            return None;
        }

        let from_pin = self.graph.pin(from_pin_id)?;
        let to_pin = self.graph.pin(to_pin_id)?;

        let from_owner = self.graph.pin_owner(from_pin_id)?;
        let to_owner = self.graph.pin_owner(to_pin_id)?;

        if self.graph.nodes.get(&from_owner)?.graph != expected_graph
            || self.graph.nodes.get(&to_owner)?.graph != expected_graph
        {
            return None;
        }

        // Always resolve direction using the pin metadata (more reliable than UI socket widgets).
        let (from, to) = match (from_pin.direction, to_pin.direction) {
            (PinDirection::Output, PinDirection::Input) => (from_pin_id, to_pin_id),
            (PinDirection::Input, PinDirection::Output) => (to_pin_id, from_pin_id),
            _ => return None,
        };

        let (from_pin, to_pin) = (self.graph.pin(from)?, self.graph.pin(to)?);

        let from_data_type = self.get_actual_pin_type(from).unwrap_or(from_pin.data_type);
        let to_data_type = self.get_actual_pin_type(to).unwrap_or(to_pin.data_type);

        if from_data_type != to_data_type {
            return None;
        }

        Some((from, to))
    }

    fn apply_connection(&mut self, from: PinId, to: PinId) {
        let (Some(from_pin), Some(to_pin)) = (self.graph.pin(from), self.graph.pin(to)) else {
            return;
        };

        let from_data_type = self.get_actual_pin_type(from).unwrap_or(from_pin.data_type);
        let to_data_type = self.get_actual_pin_type(to).unwrap_or(to_pin.data_type);

        if from_pin.direction != PinDirection::Output
            || to_pin.direction != PinDirection::Input
            || from_data_type != to_data_type
        {
            return;
        }

        // Each input pin can have only one incoming.
        // Additionally, exec output pins can have only one outgoing.
        if from_data_type == DataType::Exec {
            self.graph.links.retain(|l| l.to != to && l.from != from);
        } else {
            self.graph.links.retain(|l| l.to != to);
        }

        if !self.graph.links.iter().any(|l| l.from == from && l.to == to) {
            self.graph.links.push(Link::exec(from, to));
        }
    }

    fn rebuild_all_graph_views(&mut self, ui: &mut UserInterface) {
        let pin_owner = self.pin_owner_map();
        let event_visible = self.visible_nodes(BlueprintGraphTab::EventGraph);
        let construction_visible = self.visible_nodes(BlueprintGraphTab::ConstructionScript);

        self.rebuild_graph_view_with_pin_owner(
            ui,
            BlueprintGraphTab::EventGraph,
            &pin_owner,
            &event_visible,
        );
        self.rebuild_graph_view_with_pin_owner(
            ui,
            BlueprintGraphTab::ConstructionScript,
            &pin_owner,
            &construction_visible,
        );

        for i in 0..self.extra_tabs.len() {
            let name = self.extra_tabs[i].name.clone();
            let visible = self.visible_nodes_by_graph_name(&name);
            let view = &mut self.extra_tabs[i].view;
            Self::rebuild_graph_view_for_view(ui, &self.graph, &pin_owner, view, &visible);
        }
    }

    fn visible_nodes_by_graph_name(&self, name: &str) -> HashSet<NodeId> {
        self.graph
            .nodes
            .iter()
            .filter_map(|(id, n)| (n.graph == name).then_some(*id))
            .collect()
    }

    fn rebuild_graph_view(
        &mut self,
        ui: &mut UserInterface,
        tab: BlueprintGraphTab,
        visible_nodes: &HashSet<NodeId>,
    ) {
        let pin_owner = self.pin_owner_map();
        self.rebuild_graph_view_with_pin_owner(ui, tab, &pin_owner, visible_nodes);
    }

    fn rebuild_graph_view_with_pin_owner(
        &mut self,
        ui: &mut UserInterface,
        tab: BlueprintGraphTab,
        pin_owner: &HashMap<PinId, NodeId>,
        visible_nodes: &HashSet<NodeId>,
    ) {
        let view = match tab {
            BlueprintGraphTab::EventGraph => &mut self.event_view,
            BlueprintGraphTab::ConstructionScript => &mut self.construction_view,
        };

        Self::rebuild_graph_view_for_view(ui, &self.graph, pin_owner, view, visible_nodes);
    }

    fn rebuild_graph_view_for_view(
        ui: &mut UserInterface,
        graph: &BlueprintGraph,
        pin_owner: &HashMap<PinId, NodeId>,
        view: &mut GraphView,
        visible_nodes: &HashSet<NodeId>,
    ) {
        view.clear_ui(ui);

        let actual_pin_type = |node: &Node, pin: &fyrox_visual_scripting::Pin| -> DataType {
            match node.kind {
                BuiltinNodeKind::GetVariable | BuiltinNodeKind::SetVariable => {
                    if pin.name == "value" {
                        node.properties
                            .get("name")
                            .and_then(|v| match v {
                                Value::String(var_name) => graph
                                    .variables
                                    .iter()
                                    .find(|var| var.name == *var_name)
                                    .map(|var| var.data_type),
                                _ => None,
                            })
                            .unwrap_or(pin.data_type)
                    } else {
                        pin.data_type
                    }
                }
                _ => pin.data_type,
            }
        };

        // Create node views.
        for (node_id, node) in graph.nodes.iter() {
            if !visible_nodes.contains(node_id) {
                continue;
            }

            let model_handle = view.models.spawn(BlueprintNodeModel {
                node_id: node_id.0,
            });

            let mut input_sockets = Vec::new();
            let mut output_sockets = Vec::new();

            let mut pins = node.pins.iter().collect::<Vec<_>>();
            pins.sort_by_key(|p| {
                let exec_first = if p.data_type == DataType::Exec { 0 } else { 1 };
                let dir_group = match p.direction {
                    PinDirection::Input => 0,
                    PinDirection::Output => 1,
                };
                (dir_group, exec_first, p.name.as_str())
            });

            for pin in pins {
                let actual_data_type = actual_pin_type(node, pin);

                // Unreal-like pin colors based on data type.
                let pin_color = match actual_data_type {
                    DataType::Exec => fyrox::core::color::Color::WHITE,
                    DataType::Bool => fyrox::core::color::Color::opaque(200, 70, 70),
                    DataType::I32 => fyrox::core::color::Color::opaque(60, 200, 220),
                    DataType::F32 => fyrox::core::color::Color::opaque(90, 200, 90),
                    DataType::String => fyrox::core::color::Color::opaque(240, 80, 200),
                    DataType::Unit => fyrox::core::color::Color::opaque(140, 140, 140),
                };

                let type_suffix = if pin.direction == PinDirection::Input
                    && actual_data_type != DataType::Exec
                {
                    let ty = match actual_data_type {
                        DataType::Bool => "bool",
                        DataType::I32 => "int",
                        DataType::F32 => "float",
                        DataType::String => "string",
                        DataType::Unit => "unit",
                        DataType::Exec => "exec",
                    };
                    format!(" ({ty})")
                } else {
                    String::new()
                };

                let label = TextBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::left(4.0))
                        .with_height(18.0),
                )
                .with_text(format!("{}{}", pin.name, type_suffix))
                .build(&mut ui.build_ctx());

                // Unreal-style default value editor on the input pin row.
                let editor = if pin.direction == PinDirection::Input && actual_data_type != DataType::Exec {
                    let is_connected = graph.links.iter().any(|l| l.to == pin.id);

                    if is_connected {
                        // When linked, UE hides the default value editor.
                        label
                    } else {
                        let mut row = WidgetBuilder::new()
                            .with_horizontal_alignment(HorizontalAlignment::Stretch)
                            .with_child(label)
                            .with_margin(Thickness::uniform(0.0));

                        let key = pin.name.clone();
                        let value_widget: Handle<UiNode> = match actual_data_type {
                            DataType::String => {
                                let initial = node
                                    .properties
                                    .get(&key)
                                    .and_then(|v| match v {
                                        Value::String(s) => Some(s.as_str()),
                                        _ => None,
                                    })
                                    .unwrap_or("");

                                TextBoxBuilder::new(
                                    WidgetBuilder::new()
                                        .with_margin(Thickness::left(6.0))
                                        .with_height(22.0)
                                        .with_width(140.0)
                                        .with_horizontal_alignment(HorizontalAlignment::Stretch),
                                )
                                .with_text(initial)
                                .build(&mut ui.build_ctx())
                            }
                            DataType::Bool => {
                                let initial = node
                                    .properties
                                    .get(&key)
                                    .and_then(|v| match v {
                                        Value::Bool(b) => Some(*b),
                                        _ => None,
                                    })
                                    .unwrap_or(false);

                                CheckBoxBuilder::new(
                                    WidgetBuilder::new()
                                        .with_margin(Thickness::left(6.0))
                                        .with_height(18.0),
                                )
                                .checked(Some(initial))
                                .build(&mut ui.build_ctx())
                            }
                            DataType::I32 => {
                                let initial = node
                                    .properties
                                    .get(&key)
                                    .and_then(|v| match v {
                                        Value::I32(x) => Some(*x),
                                        _ => None,
                                    })
                                    .unwrap_or(0);

                                NumericUpDownBuilder::<i32>::new(
                                    WidgetBuilder::new()
                                        .with_margin(Thickness::left(6.0))
                                        .with_height(22.0)
                                        .with_width(120.0)
                                        .with_horizontal_alignment(HorizontalAlignment::Stretch),
                                )
                                .with_value(initial)
                                .build(&mut ui.build_ctx())
                            }
                            DataType::F32 => {
                                let initial = node
                                    .properties
                                    .get(&key)
                                    .and_then(|v| match v {
                                        Value::F32(x) => Some(*x),
                                        _ => None,
                                    })
                                    .unwrap_or(0.0);

                                NumericUpDownBuilder::<f32>::new(
                                    WidgetBuilder::new()
                                        .with_margin(Thickness::left(6.0))
                                        .with_height(22.0)
                                        .with_width(120.0)
                                        .with_horizontal_alignment(HorizontalAlignment::Stretch),
                                )
                                .with_value(initial)
                                .build(&mut ui.build_ctx())
                            }
                            _ => Handle::NONE,
                        };

                        if value_widget != Handle::NONE {
                            row = row.with_child(value_widget);
                            view.node_value_binding
                                .insert(value_widget, (*node_id, key, actual_data_type));
                        }

                        StackPanelBuilder::new(row)
                            .with_orientation(Orientation::Horizontal)
                            .build(&mut ui.build_ctx())
                    }
                } else {
                    label
                };

                let socket = SocketBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
                .with_direction(match pin.direction {
                    fyrox_visual_scripting::PinDirection::Input => SocketDirection::Input,
                    fyrox_visual_scripting::PinDirection::Output => SocketDirection::Output,
                })
                .with_parent_node(ErasedHandle::from(model_handle))
                .with_editor(editor)
                .with_index(pin.id.0 as usize)
                .with_show_index(false)
                .with_pin_color(pin_color)
                .with_canvas(view.canvas)
                .build(&mut ui.build_ctx());

                view.pin_to_socket.insert(pin.id, socket);
                view.socket_to_pin.insert(socket, pin.id);
                view.pin_to_node.insert(pin.id, *node_id);

                match pin.direction {
                    PinDirection::Input => input_sockets.push(socket),
                    PinDirection::Output => output_sockets.push(socket),
                }
            }

            let display_name = match node.kind {
                BuiltinNodeKind::BeginPlay => "BeginPlay",
                BuiltinNodeKind::Tick => "Tick",
                BuiltinNodeKind::ConstructionScript => "Construction Script",
                BuiltinNodeKind::Print => "Print",
                BuiltinNodeKind::RhaiScript => "Rhai Script",
                BuiltinNodeKind::Branch => "Branch",
                BuiltinNodeKind::GetVariable => "GetVariable",
                BuiltinNodeKind::SetVariable => "SetVariable",
                BuiltinNodeKind::Self_ => "Self",
                BuiltinNodeKind::GetActorTransform => "Get Actor Transform",
                BuiltinNodeKind::SetActorTransform => "Set Actor Transform",
                BuiltinNodeKind::SpawnActor => "Spawn Actor",
                BuiltinNodeKind::GetActorByName => "Get Actor By Name",
                BuiltinNodeKind::GetActorName => "Get Actor Name",
            }
            .to_string();

            // Unreal-like header colors based on node type.
            let header_color = match node.kind {
                BuiltinNodeKind::BeginPlay | BuiltinNodeKind::Tick => {
                    // Event nodes = red
                    fyrox::core::color::Color::opaque(180, 40, 40)
                }
                BuiltinNodeKind::ConstructionScript => {
                    // Construction = dark blue
                    fyrox::core::color::Color::opaque(30, 80, 160)
                }
                BuiltinNodeKind::Print | BuiltinNodeKind::RhaiScript => {
                    // Utility/debug = cyan
                    fyrox::core::color::Color::opaque(40, 140, 160)
                }
                BuiltinNodeKind::Branch => {
                    // Flow control = gray
                    fyrox::core::color::Color::opaque(90, 90, 90)
                }
                BuiltinNodeKind::GetVariable | BuiltinNodeKind::SetVariable => {
                    // Variable nodes = green
                    fyrox::core::color::Color::opaque(40, 140, 60)
                }
                BuiltinNodeKind::Self_
                | BuiltinNodeKind::GetActorTransform
                | BuiltinNodeKind::SetActorTransform
                | BuiltinNodeKind::SpawnActor
                | BuiltinNodeKind::GetActorByName
                | BuiltinNodeKind::GetActorName => {
                    // World nodes = orange
                    fyrox::core::color::Color::opaque(200, 120, 40)
                }
            };

            let selected_header_color = fyrox::core::color::Color::opaque(
                header_color.r.saturating_add(50),
                header_color.g.saturating_add(50),
                header_color.b.saturating_add(50),
            );

            let mut content = Handle::NONE;
            let mut content_key: Option<&'static str> = None;

            if matches!(node.kind, BuiltinNodeKind::GetVariable | BuiltinNodeKind::SetVariable) {
                content_key = Some("name");
            }

            if matches!(node.kind, BuiltinNodeKind::RhaiScript) {
                content_key = Some("code");
            }

            if let Some(key) = content_key {
                let initial = node
                    .properties
                    .get(key)
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .unwrap_or("");

                let height = if key == "code" { 72.0 } else { 24.0 };

                content = TextBoxBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::uniform(2.0))
                        .with_width(180.0)
                        .with_height(height),
                )
                .with_text(initial)
                .build(&mut ui.build_ctx());

                view.node_primary_text_box_by_node.insert(*node_id, content);
                view.node_text_box_binding
                    .insert(content, (*node_id, key.to_string()));
            }

            let node_view = {
                let mut builder = AbsmNodeBuilder::new(
                    WidgetBuilder::new().with_desired_position(
                        fyrox::core::algebra::Vector2::new(node.position[0], node.position[1]),
                    ),
                )
                .with_model_handle(model_handle)
                .with_name(display_name)
                .with_show_model_handle(false)
                .with_layout(AbsmNodeLayout::BlueprintCompact)
                .with_normal_brush(fyrox::gui::brush::Brush::Solid(header_color).into())
                .with_selected_brush(fyrox::gui::brush::Brush::Solid(selected_header_color).into())
                .with_input_sockets(input_sockets)
                .with_output_sockets(output_sockets);

                if content != Handle::NONE {
                    builder = builder.with_content(content);
                }

                builder.build(&mut ui.build_ctx())
            };

            ui.send_sync(node_view, WidgetMessage::LinkWith(view.canvas));
            view.node_views.insert(*node_id, node_view);
            view.view_to_node.insert(node_view, *node_id);
            view.node_view_handles.push(node_view);
        }

        for link in graph.links.iter() {
            let Some(src_node) = pin_owner.get(&link.from).copied() else {
                continue;
            };
            let Some(dst_node) = pin_owner.get(&link.to).copied() else {
                continue;
            };
            if !visible_nodes.contains(&src_node) || !visible_nodes.contains(&dst_node) {
                continue;
            }
            let data_type = graph
                .pin(link.from)
                .and_then(|p| graph.pin_owner(link.from).and_then(|n| graph.nodes.get(&n)).map(|node| (node, p)))
                .map(|(node, p)| actual_pin_type(node, p))
                .or_else(|| {
                    graph
                        .pin(link.to)
                        .and_then(|p| graph.pin_owner(link.to).and_then(|n| graph.nodes.get(&n)).map(|node| (node, p)))
                        .map(|(node, p)| actual_pin_type(node, p))
                })
                .unwrap_or(DataType::Unit);

            let is_exec = data_type == DataType::Exec;

            spawn_connection_view(ui, view, link.from, link.to, data_type, is_exec);
        }
    }

    fn pin_owner_map(&self) -> HashMap<PinId, NodeId> {
        let mut map = HashMap::new();
        for (node_id, node) in self.graph.nodes.iter() {
            for pin in node.pins.iter() {
                map.insert(pin.id, *node_id);
            }
        }
        map
    }

    fn visible_nodes(&self, tab: BlueprintGraphTab) -> HashSet<NodeId> {
        let name = tab_graph_name(tab);
        self.visible_nodes_by_graph_name(name)
    }

    fn rebuild_graphs_panel(&mut self, ui: &mut UserInterface) {
        for w in self.my_blueprint_graph_widgets.drain(..) {
            ui.send(w, WidgetMessage::Remove);
        }
        self.my_blueprint_graph_select.clear();

        let any = self
            .graph
            .graphs
            .iter()
            .enumerate()
            .any(|(_, g)| g.kind == GraphKind::Graph);

        if !any {
            let t = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
                .with_text("(MVP) No graphs")
                .build(&mut ui.build_ctx());
            ui.send(t, WidgetMessage::LinkWith(self.my_blueprint_graphs_panel));
            self.my_blueprint_graph_widgets.push(t);
            return;
        }

        for (index, g) in self.graph.graphs.iter().enumerate() {
            if g.kind != GraphKind::Graph {
                continue;
            }

            let b = ButtonBuilder::new(WidgetBuilder::new().with_height(24.0))
                .with_text(&g.name)
                .build(&mut ui.build_ctx());
            self.my_blueprint_graph_select.insert(b, index);
            ui.send(b, WidgetMessage::LinkWith(self.my_blueprint_graphs_panel));
            self.my_blueprint_graph_widgets.push(b);
        }
    }

    fn rebuild_functions_panel(&mut self, ui: &mut UserInterface) {
        for w in self.my_blueprint_function_widgets.drain(..) {
            ui.send(w, WidgetMessage::Remove);
        }
        self.my_blueprint_function_select.clear();

        let any = self
            .graph
            .graphs
            .iter()
            .enumerate()
            .any(|(_, g)| g.kind == GraphKind::Function);

        if !any {
            let t = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
                .with_text("(MVP) No functions")
                .build(&mut ui.build_ctx());
            ui.send(t, WidgetMessage::LinkWith(self.my_blueprint_functions_panel));
            self.my_blueprint_function_widgets.push(t);
            return;
        }

        for (index, g) in self.graph.graphs.iter().enumerate() {
            if g.kind != GraphKind::Function {
                continue;
            }

            let b = ButtonBuilder::new(WidgetBuilder::new().with_height(24.0))
                .with_text(&g.name)
                .build(&mut ui.build_ctx());
            self.my_blueprint_function_select.insert(b, index);
            ui.send(b, WidgetMessage::LinkWith(self.my_blueprint_functions_panel));
            self.my_blueprint_function_widgets.push(b);
        }
    }

    fn open_graph_tab(&mut self, ui: &mut UserInterface, name: &str, kind: GraphKind) {
        match kind {
            GraphKind::Event => {
                self.active_extra_tab = None;
                self.active_tab = BlueprintGraphTab::EventGraph;
                ui.send(self.tab_control, TabControlMessage::ActiveTab(Some(0)));
                return;
            }
            GraphKind::Construction => {
                self.active_extra_tab = None;
                self.active_tab = BlueprintGraphTab::ConstructionScript;
                ui.send(self.tab_control, TabControlMessage::ActiveTab(Some(1)));
                return;
            }
            _ => {}
        }

        if let Some(i) = self.extra_tabs.iter().position(|t| t.name == name) {
            self.active_extra_tab = Some(i);
            ui.send(self.tab_control, TabControlMessage::ActiveTab(Some(2 + i)));
            return;
        }

        let canvas =
            AbsmCanvasBuilder::new(WidgetBuilder::new().with_allow_drop(true)).build(&mut ui.build_ctx());
        let uuid = Uuid::new_v4();
        let definition = {
            let ctx = &mut ui.build_ctx();
            make_tab(name, canvas, ctx)
        };
        ui.send(
            self.tab_control,
            TabControlMessage::AddTab { uuid, definition },
        );

        let mut view = GraphView::new(canvas);
        let visible = self.visible_nodes_by_graph_name(name);
        let pin_owner = self.pin_owner_map();
        Self::rebuild_graph_view_for_view(ui, &self.graph, &pin_owner, &mut view, &visible);

        self.extra_tabs.push(ExtraTab {
            uuid,
            name: name.to_string(),
            kind,
            view,
        });
        self.active_extra_tab = Some(self.extra_tabs.len().saturating_sub(1));
    }

    fn set_selected_node(&mut self, ui: &mut UserInterface, node_id: Option<NodeId>) {
        self.selected_node = node_id;
        self.selected_variable = None;
        self.rebuild_details(ui);
    }

    fn set_selected_variable(&mut self, ui: &mut UserInterface, index: Option<usize>) {
        self.selected_variable = index;
        self.selected_node = None;
        self.rebuild_details(ui);
    }

    fn rebuild_details(&mut self, ui: &mut UserInterface) {
        for w in self.details_widgets.drain(..) {
            ui.send(w, WidgetMessage::Remove);
        }
        self.details_bindings.clear();

        let header_text = if let Some(node_id) = self.selected_node {
            self.graph
                .nodes
                .get(&node_id)
                .map(|n| format!("Selected: {:?}", n.kind))
                .unwrap_or_else(|| "Selected".to_string())
        } else if let Some(var_index) = self.selected_variable {
            self.graph
                .variables
                .get(var_index)
                .map(|v| format!("Variable: {}", v.name))
                .unwrap_or_else(|| "Variable".to_string())
        } else {
            "Select a node or variable to edit details".to_string()
        };

        let header = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
            .with_text(header_text)
            .build(&mut ui.build_ctx());
        ui.send(header, WidgetMessage::LinkWith(self.details_panel));
        self.details_widgets.push(header);

        if let Some(var_index) = self.selected_variable {
            let Some(var) = self.graph.variables.get(var_index) else {
                return;
            };

            let label = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
                .with_text("Name")
                .build(&mut ui.build_ctx());
            ui.send(label, WidgetMessage::LinkWith(self.details_panel));
            self.details_widgets.push(label);

            let tb = TextBoxBuilder::new(
                WidgetBuilder::new()
                    .with_margin(Thickness::uniform(2.0))
                    .with_height(24.0),
            )
            .with_text(var.name.clone())
            .build(&mut ui.build_ctx());
            ui.send(tb, WidgetMessage::LinkWith(self.details_panel));
            self.details_widgets.push(tb);
            self.details_bindings
                .insert(tb, DetailsBinding::VariableName { index: var_index });

            let label = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
                .with_text("Type")
                .build(&mut ui.build_ctx());
            ui.send(label, WidgetMessage::LinkWith(self.details_panel));
            self.details_widgets.push(label);

            let items = vec![
                make_dropdown_list_option(&mut ui.build_ctx(), "Bool"),
                make_dropdown_list_option(&mut ui.build_ctx(), "I32"),
                make_dropdown_list_option(&mut ui.build_ctx(), "F32"),
                make_dropdown_list_option(&mut ui.build_ctx(), "String"),
            ];

            let selected = match var.data_type {
                DataType::Bool => 0,
                DataType::I32 => 1,
                DataType::F32 => 2,
                _ => 3,
            };

            let dd = DropdownListBuilder::new(
                WidgetBuilder::new()
                    .with_margin(Thickness::uniform(2.0))
                    .with_height(24.0),
            )
            .with_items(items)
            .with_selected(selected)
            .build(&mut ui.build_ctx());

            ui.send(dd, WidgetMessage::LinkWith(self.details_panel));
            self.details_widgets.push(dd);
            self.details_bindings
                .insert(dd, DetailsBinding::VariableType { index: var_index });
            return;
        }

        let Some(node_id) = self.selected_node else {
            return;
        };
        let Some(node) = self.graph.nodes.get(&node_id) else {
            return;
        };

        match node.kind {
            BuiltinNodeKind::Print => {
                let label = TextBuilder::new(
                    WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                )
                .with_text("Text")
                .build(&mut ui.build_ctx());
                ui.send(label, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(label);

                let initial = node
                    .properties
                    .get("text")
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .unwrap_or("");

                let tb = TextBoxBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::uniform(2.0))
                        .with_height(24.0),
                )
                .with_text(initial)
                .build(&mut ui.build_ctx());
                ui.send(tb, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(tb);
                self.details_bindings
                    .insert(tb, DetailsBinding::NodeProp { node: node_id, key: "text" });
            }
            BuiltinNodeKind::RhaiScript => {
                let label = TextBuilder::new(
                    WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                )
                .with_text("Code")
                .build(&mut ui.build_ctx());
                ui.send(label, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(label);

                let initial = node
                    .properties
                    .get("code")
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .unwrap_or("");

                let tb = TextBoxBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::uniform(2.0))
                        .with_height(140.0),
                )
                .with_text(initial)
                .build(&mut ui.build_ctx());
                ui.send(tb, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(tb);
                self.details_bindings
                    .insert(tb, DetailsBinding::NodeProp { node: node_id, key: "code" });

                let hint = TextBuilder::new(
                    WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                )
                .with_text("Use: get_var(name), set_var(name, value), dt(), print(text)")
                .build(&mut ui.build_ctx());
                ui.send(hint, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(hint);
            }
            BuiltinNodeKind::GetVariable => {
                let label = TextBuilder::new(
                    WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                )
                .with_text("Variable")
                .build(&mut ui.build_ctx());
                ui.send(label, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(label);

                let initial = node
                    .properties
                    .get("name")
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .unwrap_or("");

                let tb = TextBoxBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::uniform(2.0))
                        .with_height(24.0),
                )
                .with_text(initial)
                .build(&mut ui.build_ctx());
                ui.send(tb, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(tb);
                self.details_bindings
                    .insert(tb, DetailsBinding::NodeProp { node: node_id, key: "name" });
            }
            BuiltinNodeKind::SetVariable => {
                let label = TextBuilder::new(
                    WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                )
                .with_text("Variable")
                .build(&mut ui.build_ctx());
                ui.send(label, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(label);

                let initial = node
                    .properties
                    .get("name")
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .unwrap_or("");

                let tb = TextBoxBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::uniform(2.0))
                        .with_height(24.0),
                )
                .with_text(initial)
                .build(&mut ui.build_ctx());
                ui.send(tb, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(tb);
                self.details_bindings
                    .insert(tb, DetailsBinding::NodeProp { node: node_id, key: "name" });

                let var_dt = node
                    .properties
                    .get("name")
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .and_then(|n| self.graph.variables.iter().find(|v| v.name == n))
                    .map(|v| v.data_type);

                if matches!(var_dt, Some(DataType::String) | None) {
                    let label = TextBuilder::new(
                        WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                    )
                    .with_text("Value")
                    .build(&mut ui.build_ctx());
                    ui.send(label, WidgetMessage::LinkWith(self.details_panel));
                    self.details_widgets.push(label);

                    let initial = node
                        .properties
                        .get("value")
                        .and_then(|v| match v {
                            Value::String(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .unwrap_or("");

                    let tb = TextBoxBuilder::new(
                        WidgetBuilder::new()
                            .with_margin(Thickness::uniform(2.0))
                            .with_height(24.0),
                    )
                    .with_text(initial)
                    .build(&mut ui.build_ctx());
                    ui.send(tb, WidgetMessage::LinkWith(self.details_panel));
                    self.details_widgets.push(tb);
                    self.details_bindings
                        .insert(tb, DetailsBinding::NodeProp { node: node_id, key: "value" });
                } else {
                    let hint = TextBuilder::new(
                        WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                    )
                    .with_text("(MVP) Wire a value into 'value' pin")
                    .build(&mut ui.build_ctx());
                    ui.send(hint, WidgetMessage::LinkWith(self.details_panel));
                    self.details_widgets.push(hint);
                }
            }
            _ => {
                let hint = TextBuilder::new(
                    WidgetBuilder::new().with_margin(Thickness::uniform(2.0)),
                )
                .with_text("(MVP) No editable properties")
                .build(&mut ui.build_ctx());
                ui.send(hint, WidgetMessage::LinkWith(self.details_panel));
                self.details_widgets.push(hint);
            }
        }
    }

    fn rebuild_variables_panel(&mut self, ui: &mut UserInterface) {
        for w in self.my_blueprint_variable_widgets.drain(..) {
            ui.send(w, WidgetMessage::Remove);
        }
        self.my_blueprint_variable_select.clear();
        self.my_blueprint_variable_get.clear();
        self.my_blueprint_variable_set.clear();

        if self.graph.variables.is_empty() {
            let t = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(2.0)))
                .with_text("(MVP) No variables")
                .build(&mut ui.build_ctx());
            ui.send(t, WidgetMessage::LinkWith(self.my_blueprint_variables_panel));
            self.my_blueprint_variable_widgets.push(t);
            return;
        }

        for (index, var) in self.graph.variables.iter().enumerate() {
            let select = {
                let label = format!("{} : {}", var.name, data_type_label(var.data_type));
                let b = ButtonBuilder::new(
                    WidgetBuilder::new()
                        .with_height(24.0)
                        .with_width(140.0)
                        .with_allow_drag(true),
                )
                .with_text(&label)
                .build(&mut ui.build_ctx());
                self.my_blueprint_variable_select.insert(b, index);
                b
            };

            let get_btn = {
                let b = ButtonBuilder::new(
                    WidgetBuilder::new().with_height(24.0).with_width(48.0),
                )
                .with_text("Get")
                .build(&mut ui.build_ctx());
                self.my_blueprint_variable_get.insert(b, index);
                b
            };

            let set_btn = {
                let b = ButtonBuilder::new(
                    WidgetBuilder::new().with_height(24.0).with_width(48.0),
                )
                .with_text("Set")
                .build(&mut ui.build_ctx());
                self.my_blueprint_variable_set.insert(b, index);
                b
            };

            let row = StackPanelBuilder::new(
                WidgetBuilder::new()
                    .with_margin(Thickness::uniform(1.0))
                    .with_child(select)
                    .with_child(get_btn)
                    .with_child(set_btn),
            )
            .with_orientation(Orientation::Horizontal)
            .build(&mut ui.build_ctx());

            ui.send(row, WidgetMessage::LinkWith(self.my_blueprint_variables_panel));
            self.my_blueprint_variable_widgets.push(row);
        }
    }

    fn create_variable(&mut self, ui: &mut UserInterface) {
        let base = "NewVar";
        let mut name = base.to_string();
        let mut i = 1;
        while self.graph.variables.iter().any(|v| v.name == name) {
            name = format!("{base}{i}");
            i += 1;
        }

        self.graph.variables.push(VariableDef {
            name: name.clone(),
            data_type: fyrox_visual_scripting::DataType::String,
            default_value: Some(Value::String(String::new())),
        });

        self.rebuild_variables_panel(ui);
        self.set_selected_variable(ui, Some(self.graph.variables.len().saturating_sub(1)));
    }

    fn spawn_get_variable(&mut self, ui: &mut UserInterface, var_index: usize) {
        let pos = [60.0, 200.0 + (var_index as f32) * 60.0];
        self.spawn_get_variable_at(ui, var_index, pos);
    }

    fn spawn_get_variable_at(&mut self, ui: &mut UserInterface, var_index: usize, pos: [f32; 2]) {
        let Some(var) = self.graph.variables.get(var_index) else {
            return;
        };

        let mut n = Node::new(BuiltinNodeKind::GetVariable);
        n.graph = self.active_graph_name().to_string();
        n.position = pos;
        n.set_property_string("name", var.name.clone());
        set_pin_data_type_by_name(&mut n, "value", var.data_type);
        let node_id = self.graph.add_node(n);

        self.rebuild_all_graph_views(ui);
        self.set_selected_node(ui, Some(node_id));
    }

    fn spawn_set_variable(&mut self, ui: &mut UserInterface, var_index: usize) {
        let Some(var) = self.graph.variables.get(var_index) else {
            return;
        };

        let mut n = Node::new(BuiltinNodeKind::SetVariable);
        n.graph = self.active_graph_name().to_string();
        n.position = [60.0, 240.0 + (var_index as f32) * 60.0];
        n.set_property_string("name", var.name.clone());
        match var.data_type {
            DataType::Bool => n.set_property_bool("value", false),
            DataType::I32 => n.set_property_i32("value", 0),
            DataType::F32 => n.set_property_f32("value", 0.0),
            _ => n.set_property_string("value", String::new()),
        }
        set_pin_data_type_by_name(&mut n, "value", var.data_type);
        let node_id = self.graph.add_node(n);

        self.rebuild_all_graph_views(ui);
        self.set_selected_node(ui, Some(node_id));
    }

    fn spawn_world_node(&mut self, ui: &mut UserInterface, kind: BuiltinNodeKind) {
        let graph_name = self.active_graph_name().to_string();
        self.spawn_world_node_at(
            ui,
            kind,
            &graph_name,
            fyrox::core::algebra::Vector2::new(300.0, 200.0),
        );
    }

    fn sync_variable_node_pin_types(&mut self) {
        let vars_by_name: HashMap<&str, DataType> = self
            .graph
            .variables
            .iter()
            .map(|v| (v.name.as_str(), v.data_type))
            .collect();

        for node in self.graph.nodes.values_mut() {
            match node.kind {
                BuiltinNodeKind::GetVariable | BuiltinNodeKind::SetVariable => {
                    let Some(Value::String(var_name)) = node.properties.get("name") else {
                        continue;
                    };
                    let Some(dt) = vars_by_name.get(var_name.as_str()).copied() else {
                        continue;
                    };
                    set_pin_data_type_by_name(node, "value", dt);
                }
                _ => {}
            }
        }
    }

    fn save_to_disk(&mut self, engine: &mut Engine) {
        let Some(path) = self.path.as_ref() else {
            return;
        };

        if let Err(err) = compile(&self.graph) {
            Log::err(format!("BlueprintEditor: compile error: {err}"));
            return;
        }

        let graph_json = match serde_json::to_string_pretty(&self.graph) {
            Ok(s) => s,
            Err(err) => {
                Log::err(format!("BlueprintEditor: failed to serialize graph: {err}"));
                return;
            }
        };

        let mut asset = BlueprintAsset {
            version: self.version,
            graph_json,
            prefab_path: None,
        };

        if let Err(err) = asset.save(path) {
            Log::err(format!("BlueprintEditor: save failed: {err:?}"));
            return;
        }

        if let Ok(relative) = make_relative_path(path) {
            if let Ok(resource) = block_on(engine.resource_manager.request_untyped(&relative)) {
                engine.resource_manager.state().reload_resource(resource);
            }
        }

        Log::info(format!("Saved blueprint: {}", path.display()));
    }

    fn handle_ui_message(&mut self, message: &UiMessage, engine: &mut Engine) {
        // Right-click anywhere on a graph canvas (or its children) opens the Unreal-like action menu.
        if let Some(WidgetMessage::MouseDown { button, .. }) = message.data() {
            if *button == MouseButton::Right {
                let ui = engine.user_interfaces.first_mut();

                let dest = message.destination();
                let canvas = ui
                    .try_get_node(dest)
                    .map(|n| {
                        if n.has_component::<AbsmCanvas>() {
                            dest
                        } else {
                            n.find_by_criteria_up(ui, |x| x.has_component::<AbsmCanvas>())
                        }
                    })
                    .unwrap_or_default();

                if canvas.is_some() {
                    if canvas == self.event_view.canvas {
                        self.pending_connection = None;
                        self.open_action_menu(ui, canvas, tab_graph_name(BlueprintGraphTab::EventGraph).to_string());
                        return;
                    }
                    if canvas == self.construction_view.canvas {
                        self.pending_connection = None;
                        self.open_action_menu(
                            ui,
                            canvas,
                            tab_graph_name(BlueprintGraphTab::ConstructionScript).to_string(),
                        );
                        return;
                    }
                    if let Some(extra) = self.extra_tabs.iter().find(|t| t.view.canvas == canvas) {
                        self.pending_connection = None;
                        self.open_action_menu(ui, canvas, extra.name.clone());
                        return;
                    }
                }
            }
        }

        if let Some(ButtonMessage::Click) = message.data() {
            if message.destination() == self.save {
                self.save_to_disk(engine);
            }

            if let Some(action) = self
                .action_menu_button_actions
                .get(&message.destination())
                .copied()
            {
                let ui = engine.user_interfaces.first_mut();
                let graph_name = self
                    .action_menu_spawn_graph
                    .clone()
                    .unwrap_or_else(|| self.active_graph_name().to_string());
                let pos = self
                    .action_menu_spawn_position
                    .unwrap_or(fyrox::core::algebra::Vector2::new(300.0, 200.0));

                let mut spawned: Option<NodeId> = None;

                match action {
                    ActionMenuAction::SpawnBuiltin(kind) => {
                        let mut n = Node::new(kind);
                        n.graph = graph_name.clone();
                        n.position = [pos.x, pos.y];
                        spawned = Some(self.graph.add_node(n));
                    }
                    ActionMenuAction::SpawnGetVariable(index) => {
                        // Spawn at the action-menu position and graph.
                        if let Some(var) = self.graph.variables.get(index).cloned() {
                            let mut n = Node::new(BuiltinNodeKind::GetVariable);
                            n.graph = graph_name.clone();
                            n.position = [pos.x, pos.y];
                            n.set_property_string("name", var.name);
                            set_pin_data_type_by_name(&mut n, "value", var.data_type);
                            spawned = Some(self.graph.add_node(n));
                        }
                    }
                    ActionMenuAction::SpawnSetVariable(index) => {
                        if let Some(var) = self.graph.variables.get(index).cloned() {
                            let mut n = Node::new(BuiltinNodeKind::SetVariable);
                            n.graph = graph_name.clone();
                            n.position = [pos.x, pos.y];
                            n.set_property_string("name", var.name);
                            match var.data_type {
                                DataType::Bool => n.set_property_bool("value", false),
                                DataType::I32 => n.set_property_i32("value", 0),
                                DataType::F32 => n.set_property_f32("value", 0.0),
                                _ => n.set_property_string("value", String::new()),
                            }
                            set_pin_data_type_by_name(&mut n, "value", var.data_type);
                            spawned = Some(self.graph.add_node(n));
                        }
                    }
                }

                if let Some(node_id) = spawned {
                    self.try_apply_pending_connection(node_id, &graph_name);
                    self.rebuild_all_graph_views(ui);
                    self.set_selected_node(ui, Some(node_id));
                }

                ui.send(self.action_menu, PopupMessage::Close);
                return;
            }

            if message.destination() == self.my_blueprint_new_graph {
                let ui = engine.user_interfaces.first_mut();
                let base = "NewGraph";
                let mut name = base.to_string();
                let mut i = 1;
                while self.graph.graphs.iter().any(|g| g.name == name) {
                    name = format!("{base}{i}");
                    i += 1;
                }
                self.graph.add_graph(name.clone(), GraphKind::Graph);
                self.rebuild_graphs_panel(ui);
                self.open_graph_tab(ui, &name, GraphKind::Graph);
            }

            if message.destination() == self.my_blueprint_new_variable {
                let ui = engine.user_interfaces.first_mut();
                self.create_variable(ui);
            }

            if message.destination() == self.my_blueprint_new_function {
                let ui = engine.user_interfaces.first_mut();
                let base = "NewFunction";
                let mut name = base.to_string();
                let mut i = 1;
                while self.graph.graphs.iter().any(|g| g.name == name) {
                    name = format!("{base}{i}");
                    i += 1;
                }
                self.graph.add_graph(name.clone(), GraphKind::Function);
                self.rebuild_functions_panel(ui);
                self.open_graph_tab(ui, &name, GraphKind::Function);
            }

            if message.destination() == self.my_blueprint_graphs_event {
                self.active_extra_tab = None;
                self.active_tab = BlueprintGraphTab::EventGraph;
                engine
                    .user_interfaces
                    .first_mut()
                    .send(self.tab_control, TabControlMessage::ActiveTab(Some(0)));
            }

            if message.destination() == self.my_blueprint_graphs_construction {
                self.active_extra_tab = None;
                self.active_tab = BlueprintGraphTab::ConstructionScript;
                engine
                    .user_interfaces
                    .first_mut()
                    .send(self.tab_control, TabControlMessage::ActiveTab(Some(1)));
            }

            if let Some(&graph_index) = self.my_blueprint_graph_select.get(&message.destination()) {
                let ui = engine.user_interfaces.first_mut();
                if let Some(g) = self.graph.graphs.get(graph_index) {
                    let name = g.name.clone();
                    let kind = g.kind;
                    self.open_graph_tab(ui, &name, kind);
                }
            }

            if let Some(&graph_index) =
                self.my_blueprint_function_select.get(&message.destination())
            {
                let ui = engine.user_interfaces.first_mut();
                if let Some(g) = self.graph.graphs.get(graph_index) {
                    let name = g.name.clone();
                    let kind = g.kind;
                    self.open_graph_tab(ui, &name, kind);
                }
            }

            if let Some(&index) = self.my_blueprint_variable_select.get(&message.destination()) {
                let ui = engine.user_interfaces.first_mut();
                self.set_selected_variable(ui, Some(index));
            }

            if let Some(&index) = self.my_blueprint_variable_get.get(&message.destination()) {
                let ui = engine.user_interfaces.first_mut();
                self.spawn_get_variable(ui, index);
            }

            if let Some(&index) = self.my_blueprint_variable_set.get(&message.destination()) {
                let ui = engine.user_interfaces.first_mut();
                self.spawn_set_variable(ui, index);
            }

            if let Some(kind) = self.node_palette_buttons.get(&message.destination()).copied() {
                let ui = engine.user_interfaces.first_mut();
                self.spawn_world_node(ui, kind);
            }
            return;
        }

        if let Some(TextMessage::Text(text)) = message.data::<TextMessage>() {
            if message.destination() == self.action_menu_search
                && message.direction() == MessageDirection::FromWidget
            {
                let ui = engine.user_interfaces.first_mut();
                self.rebuild_action_menu_items(ui, text);
            }
        }

        if let Some(TabControlMessage::ActiveTab(Some(index))) = message.data() {
            if message.destination() == self.tab_control {
                if *index == 0 {
                    self.active_extra_tab = None;
                    self.active_tab = BlueprintGraphTab::EventGraph;
                } else if *index == 1 {
                    self.active_extra_tab = None;
                    self.active_tab = BlueprintGraphTab::ConstructionScript;
                } else {
                    let extra_index = index.saturating_sub(2);
                    if extra_index < self.extra_tabs.len() {
                        self.active_extra_tab = Some(extra_index);
                    } else {
                        self.active_extra_tab = None;
                    }
                }
            }
        }

        // Route graph edits/selection from both views (including embedded widgets).
        let ui = engine.user_interfaces.first_mut();
        self.handle_canvas_message(message, ui, BlueprintGraphTab::EventGraph);
        self.handle_canvas_message(message, ui, BlueprintGraphTab::ConstructionScript);
        for i in 0..self.extra_tabs.len() {
            if self.handle_extra_canvas_message(message, ui, i) {
                return;
            }
        }

        // Details panel edits.
        if let Some(TextMessage::Text(text)) = message.data::<TextMessage>() {
            if let Some(binding) = self.details_bindings.get(&message.destination()).copied() {
                let ui = engine.user_interfaces.first_mut();
                match binding {
                    DetailsBinding::NodeProp { node, key } => {
                        if let Some(n) = self.graph.nodes.get_mut(&node) {
                            n.properties
                                .insert(key.to_string(), Value::String(text.clone()));
                        }

                        if let Some(tb) = self
                            .event_view
                            .node_primary_text_box_by_node
                            .get(&node)
                            .copied()
                        {
                            ui.send(tb, TextMessage::Text(text.clone()));
                        }
                        if let Some(tb) = self
                            .construction_view
                            .node_primary_text_box_by_node
                            .get(&node)
                            .copied()
                        {
                            ui.send(tb, TextMessage::Text(text.clone()));
                        }

                        for tab in self.extra_tabs.iter() {
                            if let Some(tb) =
                                tab.view.node_primary_text_box_by_node.get(&node).copied()
                            {
                                ui.send(tb, TextMessage::Text(text.clone()));
                            }
                        }

                        // Also sync any per-pin editor widgets bound to the same property.
                        let sync_view = |v: &GraphView| {
                            for (w, (n, k, ty)) in v.node_value_binding.iter() {
                                if *n == node && k == key && *ty == DataType::String {
                                    ui.send(*w, TextMessage::Text(text.clone()));
                                }
                            }
                        };
                        sync_view(&self.event_view);
                        sync_view(&self.construction_view);
                        for tab in self.extra_tabs.iter() {
                            sync_view(&tab.view);
                        }
                    }
                    DetailsBinding::VariableName { index } => {
                        let Some(var) = self.graph.variables.get_mut(index) else {
                            return;
                        };
                        let old = var.name.clone();
                        var.name = text.clone();

                        for n in self.graph.nodes.values_mut() {
                            if matches!(
                                n.kind,
                                BuiltinNodeKind::GetVariable | BuiltinNodeKind::SetVariable
                            ) {
                                if let Some(Value::String(name)) = n.properties.get_mut("name") {
                                    if *name == old {
                                        *name = text.clone();
                                    }
                                }
                            }
                        }

                        self.rebuild_variables_panel(ui);
                        self.rebuild_all_graph_views(ui);
                        self.rebuild_details(ui);
                    }
                    DetailsBinding::VariableType { .. } => {}
                }
            }
        }

        if let Some(DropdownListMessage::Selection(Some(selection))) = message.data() {
            if let Some(binding) = self.details_bindings.get(&message.destination()).copied() {
                if let DetailsBinding::VariableType { index } = binding {
                    let Some(var) = self.graph.variables.get_mut(index) else {
                        return;
                    };

                    var.data_type = match *selection {
                        0 => DataType::Bool,
                        1 => DataType::I32,
                        2 => DataType::F32,
                        _ => DataType::String,
                    };

                    var.default_value = Some(match var.data_type {
                        DataType::Bool => Value::Bool(false),
                        DataType::I32 => Value::I32(0),
                        DataType::F32 => Value::F32(0.0),
                        DataType::String => Value::String(String::new()),
                        _ => Value::Unit,
                    });

                    self.sync_variable_node_pin_types();

                    let ui = engine.user_interfaces.first_mut();
                    self.rebuild_all_graph_views(ui);
                    self.rebuild_details(ui);
                }
            }
        }
    }

    fn handle_canvas_message(
        &mut self,
        message: &UiMessage,
        ui: &mut UserInterface,
        tab: BlueprintGraphTab,
    ) {
        let view = match tab {
            BlueprintGraphTab::EventGraph => &mut self.event_view,
            BlueprintGraphTab::ConstructionScript => &mut self.construction_view,
        };

        // Drag & drop variables from "My Blueprint" onto the canvas.
        if message.destination() == view.canvas {
            if let Some(WidgetMessage::Drop(dragged)) = message.data::<WidgetMessage>() {
                if let Some(&var_index) = self.my_blueprint_variable_select.get(dragged) {
                    let local_pos = ui
                        .node(view.canvas)
                        .query_component::<AbsmCanvas>()
                        .map(|c| c.point_to_local_space(ui.cursor_position()))
                        .unwrap_or(ui.cursor_position());

                    self.spawn_get_variable_at(ui, var_index, [local_pos.x, local_pos.y]);
                    return;
                }
            }
        }

        if let Some(AbsmCanvasMessage::SelectionChanged(selection)) = message.data_from(view.canvas)
        {
            let selected = selection
                .iter()
                .find_map(|h| view.view_to_node.get(h).copied());
            self.set_selected_node(ui, selected);
            return;
        }

        if let Some(AbsmCanvasMessage::CommitDrag { .. }) = message.data_from(view.canvas) {
            for (node_id, node_view) in view.node_views.iter() {
                let pos = ui.node(*node_view).desired_local_position();
                if let Some(node) = self.graph.nodes.get_mut(node_id) {
                    node.position = [pos.x, pos.y];
                }
            }
            return;
        }

        if let Some(AbsmCanvasMessage::CommitConnection {
            source_socket,
            dest_socket,
        }) = message.data_from(view.canvas)
        {
            let expected_graph = tab_graph_name(tab);

            let (Some(a), Some(b)) = (
                view.socket_to_pin.get(source_socket).copied(),
                view.socket_to_pin.get(dest_socket).copied(),
            ) else {
                Log::warn(
                    "BlueprintEditor: CommitConnection rejected: unknown socket->pin mapping"
                        .to_string(),
                );
                return;
            };

            let Some((from, to)) = self.try_resolve_connection(a, b, expected_graph) else {
                Log::warn("BlueprintEditor: CommitConnection rejected".to_string());
                return;
            };

            self.apply_connection(from, to);

            self.rebuild_all_graph_views(ui);
            return;
        }

        if let Some(AbsmCanvasMessage::CommitConnectionToEmpty { source_socket }) =
            message.data_from(view.canvas)
        {
            let canvas = view.canvas;
            let graph_name = tab_graph_name(tab).to_string();

            let Some(from_pin_id) = view.socket_to_pin.get(source_socket).copied() else {
                Log::warn(
                    "BlueprintEditor: CommitConnectionToEmpty rejected: unknown socket->pin mapping"
                        .to_string(),
                );
                return;
            };

            let Some(from_pin) = self.graph.pin(from_pin_id) else {
                return;
            };

            let from_type = self
                .get_actual_pin_type(from_pin_id)
                .unwrap_or(from_pin.data_type);

            self.pending_connection = Some(PendingConnection {
                from: from_pin_id,
                from_dir: from_pin.direction,
                from_type,
                graph_name: graph_name.clone(),
            });

            self.open_action_menu(ui, canvas, graph_name);
            return;
        }

        // Inline node editors.
        if let Some(TextMessage::Text(text)) = message.data::<TextMessage>() {
            if let Some((node_id, key, ty)) = view.node_value_binding.get(&message.destination()) {
                if *ty == DataType::String {
                    if let Some(node) = self.graph.nodes.get_mut(node_id) {
                        node.properties.insert(key.clone(), Value::String(text.clone()));
                    }
                }
            } else if let Some((node_id, key)) = view.node_text_box_binding.get(&message.destination()) {
                // Legacy primary textbox binding (variable node name).
                if let Some(node) = self.graph.nodes.get_mut(node_id) {
                    node.properties.insert(key.clone(), Value::String(text.clone()));
                }
            }
        }

        if let Some(CheckBoxMessage::Check(Some(value))) = message.data::<CheckBoxMessage>() {
            if let Some((node_id, key, ty)) = view.node_value_binding.get(&message.destination()) {
                if *ty == DataType::Bool {
                    if let Some(node) = self.graph.nodes.get_mut(node_id) {
                        node.properties.insert(key.clone(), Value::Bool(*value));
                    }
                }
            }
        }

        if let Some(NumericUpDownMessage::Value(value)) = message.data::<NumericUpDownMessage<i32>>() {
            if let Some((node_id, key, ty)) = view.node_value_binding.get(&message.destination()) {
                if *ty == DataType::I32 {
                    if let Some(node) = self.graph.nodes.get_mut(node_id) {
                        node.properties.insert(key.clone(), Value::I32(*value));
                    }
                }
            }
        }

        if let Some(NumericUpDownMessage::Value(value)) = message.data::<NumericUpDownMessage<f32>>() {
            if let Some((node_id, key, ty)) = view.node_value_binding.get(&message.destination()) {
                if *ty == DataType::F32 {
                    if let Some(node) = self.graph.nodes.get_mut(node_id) {
                        node.properties.insert(key.clone(), Value::F32(*value));
                    }
                }
            }
        }
    }

    fn handle_extra_canvas_message(
        &mut self,
        message: &UiMessage,
        ui: &mut UserInterface,
        extra_index: usize,
    ) -> bool {
        let Some(tab) = self.extra_tabs.get(extra_index) else {
            return false;
        };
        let canvas = tab.view.canvas;

        // Drag & drop variables from "My Blueprint" onto extra graph canvases.
        if message.destination() == canvas {
            if let Some(WidgetMessage::Drop(dragged)) = message.data::<WidgetMessage>() {
                if let Some(&var_index) = self.my_blueprint_variable_select.get(dragged) {
                    let local_pos = ui
                        .node(canvas)
                        .query_component::<AbsmCanvas>()
                        .map(|c| c.point_to_local_space(ui.cursor_position()))
                        .unwrap_or(ui.cursor_position());

                    self.spawn_get_variable_at(ui, var_index, [local_pos.x, local_pos.y]);
                    return true;
                }
            }
        }

        if let Some(AbsmCanvasMessage::SelectionChanged(selection)) = message.data_from(canvas) {
            let selected = {
                let view = &self.extra_tabs[extra_index].view;
                selection
                    .iter()
                    .find_map(|h| view.view_to_node.get(h).copied())
            };
            self.set_selected_node(ui, selected);
            return true;
        }

        if let Some(AbsmCanvasMessage::CommitDrag { .. }) = message.data_from(canvas) {
            let node_views: Vec<(NodeId, Handle<UiNode>)> = {
                let view = &self.extra_tabs[extra_index].view;
                view.node_views.iter().map(|(a, b)| (*a, *b)).collect()
            };

            for (node_id, node_view) in node_views {
                let pos = ui.node(node_view).desired_local_position();
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    node.position = [pos.x, pos.y];
                }
            }
            return true;
        }

        if let Some(AbsmCanvasMessage::CommitConnection {
            source_socket,
            dest_socket,
        }) = message.data_from(canvas)
        {
            let (a, b, expected_graph) = {
                let view = &self.extra_tabs[extra_index].view;
                (
                    view.socket_to_pin.get(source_socket).copied(),
                    view.socket_to_pin.get(dest_socket).copied(),
                    self.extra_tabs[extra_index].name.clone(),
                )
            };

            let (Some(a), Some(b)) = (a, b) else {
                Log::warn(
                    "BlueprintEditor: CommitConnection rejected: unknown socket->pin mapping"
                        .to_string(),
                );
                return true;
            };

            let Some((from, to)) = self.try_resolve_connection(a, b, &expected_graph) else {
                Log::warn("BlueprintEditor: CommitConnection rejected".to_string());
                return true;
            };

            self.apply_connection(from, to);

            self.rebuild_all_graph_views(ui);
            return true;
        }

        if let Some(AbsmCanvasMessage::CommitConnectionToEmpty { source_socket }) =
            message.data_from(canvas)
        {
            let (from_pin_id, graph_name) = {
                let view = &self.extra_tabs[extra_index].view;
                (
                    view.socket_to_pin.get(source_socket).copied(),
                    self.extra_tabs[extra_index].name.clone(),
                )
            };

            let Some(from_pin_id) = from_pin_id else {
                Log::warn(
                    "BlueprintEditor: CommitConnectionToEmpty rejected: unknown socket->pin mapping"
                        .to_string(),
                );
                return true;
            };

            let Some(from_pin) = self.graph.pin(from_pin_id) else {
                return true;
            };

            let from_type = self
                .get_actual_pin_type(from_pin_id)
                .unwrap_or(from_pin.data_type);

            self.pending_connection = Some(PendingConnection {
                from: from_pin_id,
                from_dir: from_pin.direction,
                from_type,
                graph_name: graph_name.clone(),
            });
            self.open_action_menu(ui, canvas, graph_name);
            return true;
        }

        // Inline node editors (extra tabs).
        if let Some(TextMessage::Text(text)) = message.data::<TextMessage>() {
            let value_binding = {
                let view = &self.extra_tabs[extra_index].view;
                view.node_value_binding.get(&message.destination()).cloned()
            };
            if let Some((node_id, key, ty)) = value_binding {
                if ty == DataType::String {
                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                        node.properties.insert(key, Value::String(text.clone()));
                    }
                }
            } else {
                let binding = {
                    let view = &self.extra_tabs[extra_index].view;
                    view.node_text_box_binding.get(&message.destination()).cloned()
                };
                if let Some((node_id, key)) = binding {
                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                        node.properties.insert(key, Value::String(text.clone()));
                    }
                }
            }
        }

        if let Some(CheckBoxMessage::Check(Some(value))) = message.data::<CheckBoxMessage>() {
            let value_binding = {
                let view = &self.extra_tabs[extra_index].view;
                view.node_value_binding.get(&message.destination()).cloned()
            };
            if let Some((node_id, key, ty)) = value_binding {
                if ty == DataType::Bool {
                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                        node.properties.insert(key, Value::Bool(*value));
                    }
                }
            }
        }

        if let Some(NumericUpDownMessage::Value(value)) = message.data::<NumericUpDownMessage<i32>>() {
            let value_binding = {
                let view = &self.extra_tabs[extra_index].view;
                view.node_value_binding.get(&message.destination()).cloned()
            };
            if let Some((node_id, key, ty)) = value_binding {
                if ty == DataType::I32 {
                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                        node.properties.insert(key, Value::I32(*value));
                    }
                }
            }
        }

        if let Some(NumericUpDownMessage::Value(value)) = message.data::<NumericUpDownMessage<f32>>() {
            let value_binding = {
                let view = &self.extra_tabs[extra_index].view;
                view.node_value_binding.get(&message.destination()).cloned()
            };
            if let Some((node_id, key, ty)) = value_binding {
                if ty == DataType::F32 {
                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                        node.properties.insert(key, Value::F32(*value));
                    }
                }
            }
        }

        false
    }
}

fn make_tab(name: &str, content: Handle<UiNode>, ctx: &mut BuildContext) -> TabDefinition {
    TabDefinition {
        header: TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(6.0)))
            .with_text(name)
            .build(ctx),
        content,
        can_be_closed: false,
        user_data: None,
    }
}

fn spawn_connection_view(
    ui: &mut UserInterface,
    view: &mut GraphView,
    from: PinId,
    to: PinId,
    data_type: DataType,
    is_exec: bool,
) {
    let Some(source_socket) = view.pin_to_socket.get(&from).copied() else {
        return;
    };
    let Some(dest_socket) = view.pin_to_socket.get(&to).copied() else {
        return;
    };

    let Some(source_node_id) = view.pin_to_node.get(&from).copied() else {
        return;
    };
    let Some(dest_node_id) = view.pin_to_node.get(&to).copied() else {
        return;
    };

    let Some(source_node_view) = view.node_views.get(&source_node_id).copied() else {
        return;
    };
    let Some(dest_node_view) = view.node_views.get(&dest_node_id).copied() else {
        return;
    };

    // Unreal-like: exec is white; data wires are type-colored (matching pin colors).
    let base_color = match data_type {
        DataType::Exec => fyrox::core::color::Color::WHITE,
        DataType::Bool => fyrox::core::color::Color::opaque(200, 70, 70),
        DataType::I32 => fyrox::core::color::Color::opaque(60, 200, 220),
        DataType::F32 => fyrox::core::color::Color::opaque(90, 200, 90),
        DataType::String => fyrox::core::color::Color::opaque(240, 80, 200),
        DataType::Unit => fyrox::core::color::Color::opaque(140, 140, 140),
    };

    let hover_color = fyrox::core::color::Color::opaque(
        base_color.r.saturating_add(40),
        base_color.g.saturating_add(40),
        base_color.b.saturating_add(40),
    );

    let connection = ConnectionBuilder::new(WidgetBuilder::new())
        .with_source_socket(source_socket)
        .with_dest_socket(dest_socket)
        .with_source_node(source_node_view)
        .with_dest_node(dest_node_view)
        .with_brushes(
            fyrox::gui::brush::Brush::Solid(base_color),
            fyrox::gui::brush::Brush::Solid(hover_color),
        )
        .with_thickness(if is_exec { 6.0 } else { 4.0 })
        .build(view.canvas, &mut ui.build_ctx());

    ui.send_sync(connection, WidgetMessage::LinkWith(view.canvas));
    view.connection_views.push(connection);
}

fn set_pin_data_type_by_name(node: &mut Node, pin_name: &str, data_type: DataType) {
    if let Some(pin) = node.pins.iter_mut().find(|p| p.name == pin_name) {
        pin.data_type = data_type;
    }
}

fn data_type_label(dt: DataType) -> &'static str {
    match dt {
        DataType::Bool => "Bool",
        DataType::I32 => "Int",
        DataType::F32 => "Float",
        DataType::String => "String",
        DataType::Exec => "Exec",
        DataType::Unit => "Unit",
    }
}

fn ui_node_socket_dir(ui: &UserInterface, socket: Handle<UiNode>) -> Option<SocketDirection> {
    ui.node(socket)
        .query_component::<Socket>()
        .map(|s| s.direction)
}

#[derive(Default)]
pub struct BlueprintEditorPlugin {
    editor: Option<BlueprintEditor>,
}

impl EditorPlugin for BlueprintEditorPlugin {
    fn on_ui_message(&mut self, message: &mut UiMessage, editor: &mut Editor) {
        let Some(bp) = self.editor.as_mut() else {
            return;
        };

        bp.handle_ui_message(message, &mut editor.engine);

        if let Some(WindowMessage::Close) = message.data() {
            if message.destination() == bp.window {
                editor
                    .engine
                    .user_interfaces
                    .first()
                    .send(bp.window, WidgetMessage::Remove);
                self.editor = None;
            }
        }
    }

    fn on_message(&mut self, message: &Message, editor: &mut Editor) {
        let Message::OpenBlueprintEditor(path) = message else {
            return;
        };

        let bp = self
            .editor
            .get_or_insert_with(|| BlueprintEditor::new(&mut editor.engine));
        bp.open(editor, path.clone());
    }
}
