// Simplified socket implementation - Unreal Engine style
// This socket is designed to be simpler and more robust than the previous version

use crate::fyrox::core::pool::ErasedHandle;
use crate::fyrox::{
    core::{
        algebra::Vector2, color::Color, pool::Handle, reflect::prelude::*, type_traits::prelude::*,
        uuid_provider, visitor::prelude::*,
    },
    gui::{
        define_widget_deref,
        message::{MouseButton, UiMessage},
        vector_image::{Primitive, VectorImageBuilder},
        widget::{Widget, WidgetBuilder, WidgetMessage},
        BuildContext, Control, Thickness, UiNode, UserInterface,
    },
};

use fyrox::gui::message::MessageData;
use fyrox::gui::style::resource::StyleResourceExt;
use fyrox::gui::style::Style;
use std::ops::{Deref, DerefMut};

/// Message sent when a socket pin begins to be dragged
#[derive(Debug, Clone, PartialEq)]
pub enum SocketV2Message {
    /// Fired when drag starts on the pin
    /// Contains the socket handle and the position where drag started
    DragStarted(Handle<UiNode>, Vector2<f32>),
}
impl MessageData for SocketV2Message {}

/// Direction of the socket (input or output)
#[derive(Copy, Clone, PartialEq, Hash, Debug, Eq, Visit, Reflect, Default)]
pub enum SocketDirectionV2 {
    #[default]
    Input,
    Output,
}

/// A socket represents a pin on a node that can be connected to other pins
/// This is a simplified version designed to be more like Unreal Engine
#[derive(Clone, Debug, Visit, Reflect, ComponentProvider)]
#[reflect(derived_type = "UiNode")]
pub struct SocketV2 {
    widget: Widget,
    /// Position where the mouse was pressed (for drag threshold)
    click_position: Option<Vector2<f32>>,
    /// The parent node handle
    pub parent_node: ErasedHandle,
    /// Direction of this socket
    pub direction: SocketDirectionV2,
    /// The visual pin circle widget
    pin: Handle<UiNode>,
    /// Index of this socket in the node's socket list
    pub index: usize,
    /// The canvas that should receive drag events
    pub canvas: Handle<UiNode>,
}

define_widget_deref!(SocketV2);

const PIN_RADIUS: f32 = 6.0;
const PIN_SIZE: f32 = PIN_RADIUS * 2.0;

uuid_provider!(SocketV2 = "b7d1584f-8184-5fa2-b792-df99896bf63b");

impl Control for SocketV2 {
    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);

        if let Some(msg) = message.data::<WidgetMessage>() {
            match msg {
                WidgetMessage::MouseDown { button, pos } => {
                    // Accept any mouse down on the socket widget (not just the pin)
                    if *button == MouseButton::Left && !message.handled() {
                        self.click_position = Some(*pos);
                        ui.capture_mouse(self.handle());
                        message.set_handled(true);
                    }
                }
                WidgetMessage::MouseUp { button, .. } => {
                    if *button == MouseButton::Left {
                        self.click_position = None;
                        ui.release_mouse_capture();
                        message.set_handled(true);
                    }
                }
                WidgetMessage::MouseMove { pos, .. } => {
                    // If we have a click position and moved far enough, start dragging
                    if let Some(click_pos) = self.click_position {
                        if click_pos.metric_distance(pos) >= 5.0 {
                            // Send drag start message directly to the canvas
                            ui.send(
                                self.canvas,
                                SocketV2Message::DragStarted(self.handle(), *pos),
                            );
                            self.click_position = None;
                        }
                    }
                }
                WidgetMessage::MouseEnter => {
                    // Highlight the pin on hover
                    ui.send(
                        self.pin,
                        WidgetMessage::Foreground(ui.style.property(Style::BRUSH_BRIGHTEST)),
                    );
                }
                WidgetMessage::MouseLeave => {
                    // Restore normal pin color
                    ui.send(
                        self.pin,
                        WidgetMessage::Foreground(ui.style.property(Style::BRUSH_BRIGHT)),
                    );
                }
                _ => (),
            }
        }
    }
}

pub struct SocketV2Builder {
    widget_builder: WidgetBuilder,
    parent_node: ErasedHandle,
    direction: SocketDirectionV2,
    index: usize,
    pin_color: Option<Color>,
    canvas: Handle<UiNode>,
}

impl SocketV2Builder {
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self {
            widget_builder,
            parent_node: Default::default(),
            direction: SocketDirectionV2::Input,
            index: 0,
            pin_color: None,
            canvas: Handle::NONE,
        }
    }

    pub fn with_parent_node(mut self, parent_node: ErasedHandle) -> Self {
        self.parent_node = parent_node;
        self
    }

    pub fn with_direction(mut self, direction: SocketDirectionV2) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn with_pin_color(mut self, color: Color) -> Self {
        self.pin_color = Some(color);
        self
    }

    pub fn with_canvas(mut self, canvas: Handle<UiNode>) -> Self {
        self.canvas = canvas;
        self
    }

    pub fn build(self, ctx: &mut BuildContext) -> (Handle<UiNode>, Handle<UiNode>) {
        use crate::fyrox::gui::brush::Brush;

        // Create the visual pin circle
        let pin_foreground = self
            .pin_color
            .map(|c| Brush::Solid(c).into())
            .unwrap_or_else(|| ctx.style.property(Style::BRUSH_BRIGHT));

        let pin = VectorImageBuilder::new(
            WidgetBuilder::new()
                .with_width(PIN_SIZE)
                .with_height(PIN_SIZE)
                .with_foreground(pin_foreground),
        )
        .with_primitives(vec![Primitive::Circle {
            center: Vector2::new(PIN_RADIUS, PIN_RADIUS),
            radius: PIN_RADIUS,
            segments: 16,
        }])
        .build(ctx);

        // Create the socket container (just the pin for simplicity)
        let socket = SocketV2 {
            widget: self
                .widget_builder
                .with_child(pin)
                .with_margin(Thickness::uniform(5.0))
                .build(ctx),
            click_position: Default::default(),
            parent_node: self.parent_node,
            direction: self.direction,
            pin,
            index: self.index,
            canvas: self.canvas,
        };

        let socket_handle = ctx.add_node(UiNode::new(socket));
        (socket_handle, pin)
    }
}
