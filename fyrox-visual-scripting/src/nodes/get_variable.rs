//! GetVariable node.

use super::{NodeCategory, NodeDefinition, PinDef, PropertyDef};
use crate::model::{DataType, Value};

/// GetVariable node - reads a variable value.
pub struct GetVariableNode;

impl NodeDefinition for GetVariableNode {
    fn kind_name(&self) -> &'static str {
        "GetVariable"
    }

    fn display_name(&self) -> &'static str {
        "Get"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Variable
    }

    fn description(&self) -> &'static str {
        "Gets the value of a variable."
    }

    fn pins(&self) -> Vec<PinDef> {
        // Note: The data type of "value" is dynamic based on the variable.
        // Default to String; the editor/runtime will update it.
        vec![PinDef::output("value", DataType::String)]
    }

    fn properties(&self) -> Vec<PropertyDef> {
        vec![PropertyDef::new("name", Value::String(String::new())).inline()]
    }

    fn is_pure(&self) -> bool {
        true
    }

    fn inline_property_key(&self) -> Option<&'static str> {
        Some("name")
    }
}
