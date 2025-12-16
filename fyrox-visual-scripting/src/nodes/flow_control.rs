//! Flow control nodes (Branch, Sequence, etc.).

use super::{NodeCategory, NodeDefinition, PinDef};
use crate::model::DataType;

/// Branch node - if/else flow control.
pub struct BranchNode;

impl NodeDefinition for BranchNode {
    fn kind_name(&self) -> &'static str {
        "Branch"
    }

    fn display_name(&self) -> &'static str {
        "Branch"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::FlowControl
    }

    fn description(&self) -> &'static str {
        "Executes one of two paths based on a boolean condition."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::input("condition", DataType::Bool),
            PinDef::exec_out("true"),
            PinDef::exec_out("false"),
        ]
    }
}

// Future nodes can be added here:
// - SequenceNode
// - ForLoopNode
// - WhileLoopNode
// - SwitchNode
// - DoOnceNode
// - GateNode
// - FlipFlopNode
