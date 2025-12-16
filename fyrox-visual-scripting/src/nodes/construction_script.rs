//! Construction Script event node.

use super::{NodeCategory, NodeDefinition, PinDef};

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
