//! Get Actor Transform node.

use super::{NodeCategory, NodeDefinition, PinDef};
use crate::model::DataType;

/// GetActorTransform node - reads position/rotation/scale from an actor.
pub struct GetActorTransformNode;

impl NodeDefinition for GetActorTransformNode {
    fn kind_name(&self) -> &'static str {
        "GetActorTransform"
    }

    fn display_name(&self) -> &'static str {
        "Get Actor Transform"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Reads the transform (position, rotation, scale) of an actor."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("target", DataType::String),
            PinDef::output("position_x", DataType::F32),
            PinDef::output("position_y", DataType::F32),
            PinDef::output("position_z", DataType::F32),
        ]
    }

    fn is_pure(&self) -> bool {
        false
    }
}
