//! Set Actor Transform node.

use super::{NodeCategory, NodeDefinition, PinDef};
use crate::model::DataType;

/// SetActorTransform node - moves an actor to a new position.
pub struct SetActorTransformNode;

impl NodeDefinition for SetActorTransformNode {
    fn kind_name(&self) -> &'static str {
        "SetActorTransform"
    }

    fn display_name(&self) -> &'static str {
        "Set Actor Transform"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Moves an actor to a new position."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("target", DataType::String),
            PinDef::input("position_x", DataType::F32),
            PinDef::input("position_y", DataType::F32),
            PinDef::input("position_z", DataType::F32),
        ]
    }

    fn is_pure(&self) -> bool {
        false
    }
}
