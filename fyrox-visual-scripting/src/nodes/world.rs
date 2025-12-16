//! World interaction nodes (Self, Transform, Spawn, etc.).

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
        vec![
            PinDef::output("handle", DataType::String), // MVP: represent handles as strings
        ]
    }

    fn is_pure(&self) -> bool {
        true
    }
}

/// GetActorTransform node - reads position/rotation/scale from an actor.
pub struct GetActorTransformNode;

impl NodeDefinition for GetActorTransformNode {
    fn kind_name(&self) -> &'static str {
        "GetActorTransform"
    }

    fn display_name(&self) -> &'static str {
        "Get Actor Transform"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Reads the transform (position, rotation, scale) of an actor."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("target", DataType::String), // Handle to actor
            PinDef::output("position_x", DataType::F32),
            PinDef::output("position_y", DataType::F32),
            PinDef::output("position_z", DataType::F32),
        ]
    }

    fn is_pure(&self) -> bool {
        false
    }
}

/// SetActorTransform node - moves an actor to a new position.
pub struct SetActorTransformNode;

impl NodeDefinition for SetActorTransformNode {
    fn kind_name(&self) -> &'static str {
        "SetActorTransform"
    }

    fn display_name(&self) -> &'static str {
        "Set Actor Transform"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Moves an actor to a new position."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("target", DataType::String), // Handle to actor
            PinDef::input("position_x", DataType::F32),
            PinDef::input("position_y", DataType::F32),
            PinDef::input("position_z", DataType::F32),
        ]
    }

    fn is_pure(&self) -> bool {
        false
    }
}

/// SpawnActor node - spawns a new actor/instance of a blueprint (MVP: simple version).
pub struct SpawnActorNode;

impl NodeDefinition for SpawnActorNode {
    fn kind_name(&self) -> &'static str {
        "SpawnActor"
    }

    fn display_name(&self) -> &'static str {
        "Spawn Actor"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Spawns a new actor (or blueprint instance) at the given location."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("blueprint_name", DataType::String), // MVP: name/path to spawn
            PinDef::input("position_x", DataType::F32),
            PinDef::input("position_y", DataType::F32),
            PinDef::input("position_z", DataType::F32),
            PinDef::output("new_actor", DataType::String), // Handle to spawned actor
        ]
    }

    fn is_pure(&self) -> bool {
        false
    }
}

/// GetActorByName node - finds an actor in the scene by name (MVP: simple string lookup).
pub struct GetActorByNameNode;

impl NodeDefinition for GetActorByNameNode {
    fn kind_name(&self) -> &'static str {
        "GetActorByName"
    }

    fn display_name(&self) -> &'static str {
        "Get Actor By Name"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Finds an actor in the scene by name. Returns the actor handle if found."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::exec_in("exec"),
            PinDef::exec_out("then"),
            PinDef::input("name", DataType::String),
            PinDef::output("found", DataType::String), // Handle if found, empty if not
        ]
    }

    fn is_pure(&self) -> bool {
        false
    }
}

/// GetActorName node - gets the name of an actor.
pub struct GetActorNameNode;

impl NodeDefinition for GetActorNameNode {
    fn kind_name(&self) -> &'static str {
        "GetActorName"
    }

    fn display_name(&self) -> &'static str {
        "Get Actor Name"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Utility
    }

    fn description(&self) -> &'static str {
        "Gets the name of an actor."
    }

    fn pins(&self) -> Vec<PinDef> {
        vec![
            PinDef::input("target", DataType::String),
            PinDef::output("name", DataType::String),
        ]
    }

    fn is_pure(&self) -> bool {
        true
    }
}
