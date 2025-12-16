//! Event nodes (entry points).

use super::{NodeCategory, NodeDefinition, PinDef};
#[allow(unused_imports)]
use crate::model::DataType;

/// BeginPlay event - fires once when the game starts.
pub struct BeginPlayNode;

impl NodeDefinition for BeginPlayNode {
    fn kind_name(&self) -> &'static str {
        "BeginPlay"
    }

    fn display_name(&self) -> &'static str {
        "Event BeginPlay"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Event
    }

    fn description(&self) -> &'static str {
        "Fires once when the game starts or the actor is spawned."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![PinDef::exec_out("then")]
    }

    fn is_entry(&self) -> bool {
        true
    }

    fn allowed_graphs(&self) -> Vec<&'static str> {
        vec!["EventGraph"]
    }
}

/// Tick event - fires every frame.
pub struct TickNode;

impl NodeDefinition for TickNode {
    fn kind_name(&self) -> &'static str {
        "Tick"
    }

    fn display_name(&self) -> &'static str {
        "Event Tick"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Event
    }

    fn description(&self) -> &'static str {
        "Fires every frame. Use sparingly for performance."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_out("then"),
            PinDef::output("deltaTime", DataType::F32),
        ]
    }

    fn is_entry(&self) -> bool {
        true
    }

    fn allowed_graphs(&self) -> Vec<&'static str> {
        vec!["EventGraph"]
    }
}

/// Construction Script event - fires when the actor is constructed/spawned.
pub struct ConstructionScriptNode;

impl NodeDefinition for ConstructionScriptNode {
    fn kind_name(&self) -> &'static str {
        "ConstructionScript"
    }

    fn display_name(&self) -> &'static str {
        "Construction Script"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Event
    }

    fn description(&self) -> &'static str {
        "Fires when the actor is constructed. Use for setup logic."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![PinDef::exec_out("then")]
    }

    fn is_entry(&self) -> bool {
        true
    }

    fn allowed_graphs(&self) -> Vec<&'static str> {
        vec!["ConstructionScript"]
    }
}
