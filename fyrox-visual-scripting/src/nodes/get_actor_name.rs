//! Get Actor Name node.

use super::{NodeCategory, NodeDefinition, PinDef};
use crate::model::DataType;

/// GetActorName node - gets the name of an actor.
pub struct GetActorNameNode;

impl NodeDefinition for GetActorNameNode {
    fn kind_name(&self) -> &'static str {
        "GetActorName"
    }

    fn display_name(&self) -> &'static str {
        "Get Actor Name"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Gets the name of an actor."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![PinDef::input("target", DataType::String), PinDef::output("name", DataType::String)]
    }

    fn is_pure(&self) -> bool {
        true
    }
}
