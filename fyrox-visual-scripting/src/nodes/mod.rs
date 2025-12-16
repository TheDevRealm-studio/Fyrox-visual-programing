//! Node definitions for visual scripting.
//!
//! This module contains the base node trait and all built-in node implementations.
//! To add a new node type:
//! 1. Create a new file in this folder (e.g., `my_node.rs`)
//! 2. Implement `NodeDefinition` for your node
//! 3. Register it in the `NODE_REGISTRY`

mod base;

mod begin_play;
mod branch;
mod construction_script;
mod get_actor_by_name;
mod get_actor_name;
mod get_actor_transform;
mod get_variable;
mod print;
mod rhai_script;
mod self_node;
mod set_actor_transform;
mod set_variable;
mod spawn_actor;
mod tick;

pub use base::*;

pub use begin_play::*;
pub use branch::*;
pub use construction_script::*;
pub use get_actor_by_name::*;
pub use get_actor_name::*;
pub use get_actor_transform::*;
pub use get_variable::*;
pub use print::*;
pub use rhai_script::*;
pub use self_node::*;
pub use set_actor_transform::*;
pub use set_variable::*;
pub use spawn_actor::*;
pub use tick::*;

/// Registry of all available node definitions.
/// Used by the editor to populate the node palette.
pub fn all_node_definitions() -> Vec<&'static dyn NodeDefinition> {
    vec![
        // Events
        &BeginPlayNode,
        &TickNode,
        &ConstructionScriptNode,
        // Flow Control
        &BranchNode,
        // Utilities
        &PrintNode,
        &RhaiScriptNode,
        // Variables
        &GetVariableNode,
        &SetVariableNode,
        // World Interaction
        &SelfNode,
        &GetActorTransformNode,
        &SetActorTransformNode,
        &SpawnActorNode,
        &GetActorByNameNode,
        &GetActorNameNode,
    ]
}

/// Get a node definition by its kind name.
pub fn get_node_definition(kind: &str) -> Option<&'static dyn NodeDefinition> {
    all_node_definitions()
        .into_iter()
        .find(|def| def.kind_name() == kind)
}

/// Node category for organizing in the editor palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeCategory {
    Event,
    FlowControl,
    Utility,
    Variable,
    Math,
    String,
    Custom,
}

use crate::model::DataType;

impl NodeCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            NodeCategory::Event => "Events",
            NodeCategory::FlowControl => "Flow Control",
            NodeCategory::Utility => "Utilities",
            NodeCategory::Variable => "Variables",
            NodeCategory::Math => "Math",
            NodeCategory::String => "String",
            NodeCategory::Custom => "Custom",
        }
    }

    /// Unreal-like header color for this category (enhanced vibrant palette).
    pub fn header_color(&self) -> (u8, u8, u8) {
        match self {
            NodeCategory::Event => (220, 64, 64),         // Rich red
            NodeCategory::FlowControl => (110, 110, 110), // Neutral gray
            NodeCategory::Utility => (64, 180, 200),      // Vibrant cyan
            NodeCategory::Variable => (64, 180, 96),      // Vibrant green
            NodeCategory::Math => (80, 220, 80),          // Bright green
            NodeCategory::String => (255, 100, 220),      // Hot pink/magenta
            NodeCategory::Custom => (120, 120, 120),      // Medium gray
        }
    }
}

/// Pin color based on data type (Unreal-like - enhanced vibrant colors).
pub fn pin_color_for_type(data_type: DataType) -> (u8, u8, u8) {
    match data_type {
        DataType::Exec => (255, 255, 255),     // White
        DataType::Bool => (220, 96, 96),       // Warm red
        DataType::I32 => (100, 220, 255),      // Bright cyan
        DataType::F32 => (120, 220, 100),      // Bright green
        DataType::String => (255, 100, 220),   // Hot pink/magenta
        DataType::Unit => (160, 160, 160),     // Light gray
    }
}
