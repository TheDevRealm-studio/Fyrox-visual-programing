//! Rhai Script node.

use super::{NodeCategory, NodeDefinition, PinDef, PropertyDef};
use crate::model::{DataType, Value};

/// Rhai Script node - evaluates a Rhai snippet with access to blueprint variables.
pub struct RhaiScriptNode;

impl NodeDefinition for RhaiScriptNode {
    fn kind_name(&self) -> &'static str {
        "RhaiScript"
    }

    fn display_name(&self) -> &'static str {
        "Rhai Script"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Runs a Rhai script. Use get_var(name), set_var(name, value), dt(), and print(text)."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("code", DataType::String),
        ]
    }

    fn properties(&self) -> Vec<PropertyDef> {
        vec![PropertyDef::new("code", Value::String(String::new())).inline()]
    }

    fn inline_property_key(&self) -> Option<&'static str> {
        Some("code")
    }
}
