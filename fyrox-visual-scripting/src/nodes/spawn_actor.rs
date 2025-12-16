//! Spawn Actor node.

use super::{NodeCategory, NodeDefinition, PinDef};
use crate::model::DataType;

/// SpawnActor node - spawns a new actor/instance of a blueprint (MVP: simple version).
pub struct SpawnActorNode;

impl NodeDefinition for SpawnActorNode {
    fn kind_name(&self) -> &'static str {
        "SpawnActor"
    }

    fn display_name(&self) -> &'static str {
        "Spawn Actor"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Spawns a new actor (or blueprint instance) at the given location."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("blueprint_name", DataType::String),
            PinDef::input("position_x", DataType::F32),
            PinDef::input("position_y", DataType::F32),
            PinDef::input("position_z", DataType::F32),
            PinDef::output("new_actor", DataType::String),
        ]
    }

    fn is_pure(&self) -> bool {
        false
    }
}
