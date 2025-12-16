//! Base node trait and common utilities.

use crate::model::{DataType, Pin, PinDirection, PinId, Value};
use std::collections::BTreeMap;

use super::NodeCategory;

/// Pin definition for node templates (before IDs are assigned by the graph).
#[derive(Debug, Clone)]
pub struct PinDef {
    pub name: String,
    pub direction: PinDirection,
    pub data_type: DataType,
}

impl PinDef {
    pub fn input(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            direction: PinDirection::Input,
            data_type,
        }
    }

    pub fn output(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            direction: PinDirection::Output,
            data_type,
        }
    }

    pub fn exec_in(name: impl Into<String>) -> Self {
        Self::input(name, DataType::Exec)
    }

    pub fn exec_out(name: impl Into<String>) -> Self {
        Self::output(name, DataType::Exec)
    }

    /// Convert to a Pin with a placeholder ID (will be remapped by the graph).
    pub fn to_pin(&self, id: u32) -> Pin {
        Pin {
            id: PinId(id),
            name: self.name.clone(),
            direction: self.direction,
            data_type: self.data_type,
        }
    }
}

/// Property definition for node configuration.
#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub default_value: Value,
    /// If true, show an inline editor in the node body.
    pub inline_editable: bool,
}

impl PropertyDef {
    pub fn new(name: impl Into<String>, default_value: Value) -> Self {
        let data_type = default_value.data_type();
        Self {
            name: name.into(),
            data_type,
            default_value,
            inline_editable: false,
        }
    }

    pub fn inline(mut self) -> Self {
        self.inline_editable = true;
        self
    }
}

/// Trait that all node types must implement.
/// This allows easy extension of the node system.
pub trait NodeDefinition: Send + Sync {
    /// Unique identifier for this node type (e.g., "BeginPlay", "Print").
    fn kind_name(&self) -> &'static str;

    /// Display name shown in the editor.
    fn display_name(&self) -> &'static str;

    /// Category for organizing in the node palette.
    fn category(&self) -> NodeCategory;

    /// Description shown in tooltips.
    fn description(&self) -> &'static str {
        ""
    }

    /// Pin definitions for this node type.
    fn pins(&self) -> Vec<PinDef>;

    /// Property definitions for this node type.
    fn properties(&self) -> Vec<PropertyDef> {
        vec![]
    }

    /// Whether this node is an entry point (event node).
    fn is_entry(&self) -> bool {
        false
    }

    /// Whether this node is pure (no side effects, can be evaluated anytime).
    fn is_pure(&self) -> bool {
        false
    }

    /// Which graph types this node can appear in.
    fn allowed_graphs(&self) -> Vec<&'static str> {
        vec!["EventGraph", "ConstructionScript"]
    }

    /// Create pins with proper IDs.
    fn create_pins(&self) -> Vec<Pin> {
        self.pins()
            .into_iter()
            .enumerate()
            .map(|(i, def)| def.to_pin(i as u32))
            .collect()
    }

    /// Create default properties map.
    fn create_properties(&self) -> BTreeMap<String, Value> {
        self.properties()
            .into_iter()
            .map(|p| (p.name, p.default_value))
            .collect()
    }

    /// Get the property key for inline editing (if any).
    /// Override this in node implementations that have inline editable properties.
    fn inline_property_key(&self) -> Option<&'static str> {
        None
    }
}
