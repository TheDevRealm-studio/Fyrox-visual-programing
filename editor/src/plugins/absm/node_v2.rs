// Simplified Blueprint Node - Unreal Engine style
// Designed to be robust and easy to understand

use crate::fyrox::{
    core::{
        pool::Handle, reflect::prelude::*, type_traits::prelude::*,
        uuid_provider, visitor::prelude::*,
    },
    gui::{
        border::BorderBuilder,
        brush::Brush,
        define_widget_deref,
        grid::{Column, GridBuilder, Row},
        message::UiMessage,
        stack_panel::StackPanelBuilder,
        text::TextBuilder,
        widget::{Widget, WidgetBuilder},
        BuildContext, Control, Thickness, UiNode, UserInterface,
        VerticalAlignment,
    },
};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Visit, Reflect, ComponentProvider)]
#[reflect(derived_type = "UiNode")]
pub struct SimpleBlueprintNode {
    widget: Widget,
    background: Handle<UiNode>,
    pub input_sockets: Vec<Handle<UiNode>>,
    pub output_sockets: Vec<Handle<UiNode>>,
}

define_widget_deref!(SimpleBlueprintNode);

uuid_provider!(SimpleBlueprintNode = "c8e2695f-9295-6fb3-c880-3ef00aa7cf74");

impl Control for SimpleBlueprintNode {
    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);
    }
}

pub struct SimpleBlueprintNodeBuilder {
    widget_builder: WidgetBuilder,
    node_name: String,
    header_color: Brush,
    input_sockets: Vec<Handle<UiNode>>,
    output_sockets: Vec<Handle<UiNode>>,
}

impl SimpleBlueprintNodeBuilder {
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self {
            widget_builder,
            node_name: "Node".to_string(),
            header_color: Brush::Solid(fyrox::core::color::Color::opaque(100, 100, 100)),
            input_sockets: Vec::new(),
            output_sockets: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.node_name = name.into();
        self
    }

    pub fn with_header_color(mut self, color: Brush) -> Self {
        self.header_color = color;
        self
    }

    pub fn with_input_sockets(mut self, sockets: Vec<Handle<UiNode>>) -> Self {
        self.input_sockets = sockets;
        self
    }

    pub fn with_output_sockets(mut self, sockets: Vec<Handle<UiNode>>) -> Self {
        self.output_sockets = sockets;
        self
    }

    pub fn build(self, ctx: &mut BuildContext) -> Handle<UiNode> {
        // Header with node name
        let header_text = TextBuilder::new(
            WidgetBuilder::new().with_margin(Thickness {
                left: 10.0,
                top: 6.0,
                right: 10.0,
                bottom: 6.0,
            }),
        )
        .with_text(self.node_name)
        .build(ctx);

        let header = BorderBuilder::new(
            WidgetBuilder::new()
                .on_row(0)
                .with_height(32.0)
                .with_background(self.header_color.clone().into())
                .with_child(header_text),
        )
        .with_pad_by_corner_radius(false)
        .with_corner_radius(6.0.into())
        .with_stroke_thickness(Thickness::zero().into())
        .build(ctx);

        // Input sockets (left)
        let input_panel = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness {
                    left: 0.0,
                    top: 10.0,
                    right: 10.0,
                    bottom: 10.0,
                })
                .with_vertical_alignment(VerticalAlignment::Top)
                .with_children(self.input_sockets.clone())
                .on_column(0),
        )
        .build(ctx);

        // Output sockets (right)
        let output_panel = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness {
                    left: 10.0,
                    top: 10.0,
                    right: 0.0,
                    bottom: 10.0,
                })
                .with_vertical_alignment(VerticalAlignment::Top)
                .with_children(self.output_sockets.clone())
                .on_column(2),
        )
        .build(ctx);

        // Main grid layout: [input | output]
        let background = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(input_panel)
                .with_child(output_panel),
        )
        .add_row(Row::stretch()) // Content row
        .add_column(Column::auto()) // Input column
        .add_column(Column::auto()) // Output column
        .build(ctx);

        // Root: header on top, content below
        let root_grid = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(header)
                .with_child(background),
        )
        .add_row(Row::auto()) // Header
        .add_row(Row::stretch()) // Content
        .add_column(Column::stretch())
        .build(ctx);

        let node = SimpleBlueprintNode {
            widget: self
                .widget_builder
                .with_child(root_grid)
                .with_background(
                    Brush::Solid(fyrox::core::color::Color::opaque(30, 30, 30)).into(),
                )
                .build(ctx),
            background,
            input_sockets: self.input_sockets,
            output_sockets: self.output_sockets,
        };

        ctx.add_node(UiNode::new(node))
    }
}
