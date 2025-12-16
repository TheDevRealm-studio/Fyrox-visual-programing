//! Utility nodes (Print, Delay, etc.).

use super::{NodeCategory, NodeDefinition, PinDef, PropertyDef};
use crate::model::{DataType, Value};

/// Print node - outputs text to the log.
pub struct PrintNode;

impl NodeDefinition for PrintNode {
    fn kind_name(&self) -> &'static str {
        "Print"
    }

    fn display_name(&self) -> &'static str {
        "Print String"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Prints a string to the output log."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("text", DataType::String),
        ]
    }

    fn properties(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef::new("text", Value::String("Hello".to_string())).inline(),
        ]
    }

    fn inline_property_key(&self) -> Option<&'static str> {
        Some("text")
    }
}

// Future nodes can be added here:
// - DelayNode
// - PrintWarningNode
// - PrintErrorNode
// - FormatStringNode
