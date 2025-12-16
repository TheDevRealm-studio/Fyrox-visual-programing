//! SetVariable node.

use super::{NodeCategory, NodeDefinition, PinDef, PropertyDef};
use crate::model::{DataType, Value};

/// SetVariable node - writes a variable value.
pub struct SetVariableNode;

impl NodeDefinition for SetVariableNode {
    fn kind_name(&self) -> &'static str {
        "SetVariable"
    }

    fn display_name(&self) -> &'static str {
        "Set"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Variable
    }

    fn description(&self) -> &'static str {
        "Sets the value of a variable."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            // Note: The data type of "value" is dynamic based on the variable.
            PinDef::input("value", DataType::String),
        ]
    }

    fn properties(&self) -> Vec<PropertyDef> {
        vec![PropertyDef::new("name", Value::String(String::new())).inline()]
    }

    fn inline_property_key(&self) -> Option<&'static str> {
        Some("name")
    }
}
