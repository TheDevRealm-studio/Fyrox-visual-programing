//! BeginPlay event node.

use super::{NodeCategory, NodeDefinition, PinDef};

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
