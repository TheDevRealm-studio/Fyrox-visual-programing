//! Self node.

use super::{NodeCategory, NodeDefinition, PinDef};
use crate::model::DataType;

/// Self node - returns the current actor/node handle.
pub struct SelfNode;

impl NodeDefinition for SelfNode {
    fn kind_name(&self) -> &'static str {
        "Self"
    }

    fn display_name(&self) -> &'static str {
        "Self"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Returns the current actor/node handle executing this graph."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![PinDef::output("handle", DataType::String)]
    }

    fn is_pure(&self) -> bool {
        true
    }
}
