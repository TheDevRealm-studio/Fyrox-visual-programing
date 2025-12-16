//! Tick event node.

use super::{NodeCategory, NodeDefinition, PinDef};
use crate::model::DataType;

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
        vec![PinDef::exec_out("then"), PinDef::output("deltaTime", DataType::F32)]
    }

    fn is_entry(&self) -> bool {
        true
    }

    fn allowed_graphs(&self) -> Vec<&'static str> {
        vec!["EventGraph"]
    }
}
