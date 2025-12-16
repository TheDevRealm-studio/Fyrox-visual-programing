//! Get Actor By Name node.

use super::{NodeCategory, NodeDefinition, PinDef};
use crate::model::DataType;

/// GetActorByName node - finds an actor in the scene by name (MVP: simple string lookup).
pub struct GetActorByNameNode;

impl NodeDefinition for GetActorByNameNode {
    fn kind_name(&self) -> &'static str {
        "GetActorByName"
    }

    fn display_name(&self) -> &'static str {
        "Get Actor By Name"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Finds an actor in the scene by name. Returns the actor handle if found."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("name", DataType::String),
            PinDef::output("actor", DataType::String),
        ]
    }

    fn is_pure(&self) -> bool {
        false
    }
}
