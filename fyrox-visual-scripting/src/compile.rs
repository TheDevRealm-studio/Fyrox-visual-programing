use crate::{
    error::{CompileError, ValidationError},
    model::{BlueprintGraph, BuiltinNodeKind, DataType, Link, NodeId, PinDirection, PinId, Value},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct CompiledGraph {
    pub begin_play_entry: Option<NodeId>,
    pub construction_entry: Option<NodeId>,
    pub tick_entry: Option<NodeId>,

    pub variables: BTreeMap<String, Value>,

    pub nodes: BTreeMap<NodeId, CompiledNode>,
    pub exec_edges: BTreeMap<PinId, PinId>,
    // Data edges are keyed by *input* pin id, because each input can have at most one incoming
    // connection, while outputs may fan-out to many inputs.
    pub data_edges: BTreeMap<PinId, PinId>,
}

#[derive(Debug, Clone)]
pub struct CompiledNode {
    pub kind: BuiltinNodeKind,
    pub properties: BTreeMap<String, Value>,
    pub pins: BTreeMap<String, (PinId, PinDirection, DataType)>,
}

impl CompiledNode {
    pub fn pin(&self, name: &str) -> Option<(PinId, PinDirection, DataType)> {
        self.pins.get(name).copied()
    }
}

pub fn compile(graph: &BlueprintGraph) -> Result<CompiledGraph, CompileError> {
    validate(graph)?;

    let mut variables = BTreeMap::new();
    for var in graph.variables.iter() {
        let value = var
            .default_value
            .clone()
            .unwrap_or_else(|| match var.data_type {
                DataType::Bool => Value::Bool(false),
                DataType::I32 => Value::I32(0),
                DataType::F32 => Value::F32(0.0),
                DataType::String => Value::String(String::new()),
                DataType::Exec | DataType::Unit => Value::Unit,
            });
        variables.insert(var.name.clone(), value);
    }

    let mut nodes = BTreeMap::new();
    for (node_id, node) in graph.nodes.iter() {
        let mut pins = BTreeMap::new();
        for pin in node.pins.iter() {
            pins.insert(pin.name.clone(), (pin.id, pin.direction, pin.data_type));
        }
        nodes.insert(
            *node_id,
            CompiledNode {
                kind: node.kind,
                properties: node.properties.clone(),
                pins,
            },
        );
    }

    let mut exec_edges = BTreeMap::new();
    let mut data_edges = BTreeMap::new();

    for Link { from, to } in graph.links.iter().cloned() {
        let from_pin = graph
            .pin(from)
            .ok_or_else(|| CompileError::new(ValidationError::UnknownPin).with_pin(from))?;
        let _to_pin = graph
            .pin(to)
            .ok_or_else(|| CompileError::new(ValidationError::UnknownPin).with_pin(to))?;

        if from_pin.data_type == DataType::Exec {
            exec_edges.insert(from, to);
        } else {
            // Store by input pin for correct fan-out semantics.
            data_edges.insert(to, from);
        }
    }

    Ok(CompiledGraph {
        begin_play_entry: find_entry(graph, BuiltinNodeKind::BeginPlay),
        construction_entry: find_entry(graph, BuiltinNodeKind::ConstructionScript),
        tick_entry: find_entry(graph, BuiltinNodeKind::Tick),
        variables,
        nodes,
        exec_edges,
        data_edges,
    })
}

fn find_entry(graph: &BlueprintGraph, kind: BuiltinNodeKind) -> Option<NodeId> {
    graph
        .nodes
        .iter()
        .find_map(|(id, n)| (n.kind == kind).then_some(*id))
}

fn validate(graph: &BlueprintGraph) -> Result<(), CompileError> {
    // Variables: unique names.
    {
        let mut seen = BTreeSet::new();
        for v in graph.variables.iter() {
            if !seen.insert(v.name.as_str()) {
                return Err(CompileError::new(ValidationError::DuplicateVariable));
            }
        }
    }

    let vars: BTreeSet<&str> = graph.variables.iter().map(|v| v.name.as_str()).collect();

    // Links: pin existence, direction and type correctness.
    for Link { from, to } in graph.links.iter() {
        let from_pin = graph
            .pin(*from)
            .ok_or_else(|| CompileError::new(ValidationError::UnknownPin).with_pin(*from))?;
        let to_pin = graph
            .pin(*to)
            .ok_or_else(|| CompileError::new(ValidationError::UnknownPin).with_pin(*to))?;

        // Do not allow links between different graphs (EventGraph vs ConstructionScript).
        if let (Some(from_node), Some(to_node)) = (graph.pin_owner(*from), graph.pin_owner(*to)) {
            let from_graph = graph
                .nodes
                .get(&from_node)
                .map(|n| n.graph.as_str())
                .unwrap_or("EventGraph");
            let to_graph = graph
                .nodes
                .get(&to_node)
                .map(|n| n.graph.as_str())
                .unwrap_or("EventGraph");
            if from_graph != to_graph {
                return Err(CompileError::new(ValidationError::CrossGraphLink)
                    .with_node(from_node)
                    .with_node(to_node)
                    .with_pin(*from)
                    .with_pin(*to));
            }
        }

        if from_pin.direction != PinDirection::Output || to_pin.direction != PinDirection::Input {
            return Err(CompileError::new(ValidationError::DirectionMismatch)
                .with_pin(*from)
                .with_pin(*to));
        }

        if from_pin.data_type != to_pin.data_type {
            return Err(CompileError::new(ValidationError::TypeMismatch)
                .with_pin(*from)
                .with_pin(*to));
        }
    }

    // Exec input pins can only have one incoming.
    let mut exec_incoming_count: BTreeMap<PinId, usize> = BTreeMap::new();
    for Link { from: _, to } in graph.links.iter() {
        let to_pin = graph
            .pin(*to)
            .ok_or_else(|| CompileError::new(ValidationError::UnknownPin).with_pin(*to))?;
        if to_pin.data_type == DataType::Exec {
            *exec_incoming_count.entry(*to).or_insert(0) += 1;
        }
    }
    if let Some((pin, _)) = exec_incoming_count
        .iter()
        .find(|(_, count)| **count > 1)
    {
        let node = graph.pin_owner(*pin);
        return Err(CompileError::new(ValidationError::MultipleExecInputs)
            .with_pin(*pin)
            .with_node(node.unwrap_or(NodeId(0))));
    }

    // Entry nodes are optional at compile time.
    // The editor will create the common graphs by default; the runtime will no-op if an entry is missing.

    // Variable nodes must refer to existing variables.
    for (node_id, node) in graph.nodes.iter() {
        match node.kind {
            BuiltinNodeKind::GetVariable | BuiltinNodeKind::SetVariable => {
                let name = node.properties.get("name").and_then(|v| match v {
                    Value::String(s) => Some(s.as_str()),
                    _ => None,
                });
                if let Some(name) = name {
                    if !vars.contains(name) {
                        return Err(CompileError::new(ValidationError::UnknownVariable)
                            .with_node(*node_id));
                    }
                } else {
                    return Err(CompileError::new(ValidationError::UnknownVariable)
                        .with_node(*node_id));
                }
            }
            _ => {}
        }
    }

    // Detect cycles on exec flow graph.
    detect_exec_cycles(graph)?;

    Ok(())
}

fn detect_exec_cycles(graph: &BlueprintGraph) -> Result<(), CompileError> {
    // Build adjacency on node-level for exec links.
    let mut adjacency: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();

    for Link { from, to } in graph.links.iter() {
        let from_pin = graph
            .pin(*from)
            .ok_or_else(|| CompileError::new(ValidationError::UnknownPin).with_pin(*from))?;
        if from_pin.data_type != DataType::Exec {
            continue;
        }
        let from_node = graph
            .pin_owner(*from)
            .ok_or_else(|| CompileError::new(ValidationError::BrokenExecLink).with_pin(*from))?;
        let to_node = graph
            .pin_owner(*to)
            .ok_or_else(|| CompileError::new(ValidationError::BrokenExecLink).with_pin(*to))?;
        adjacency.entry(from_node).or_default().push(to_node);
    }

    let mut visited = BTreeSet::new();
    let mut stack = BTreeSet::new();

    for node_id in graph.nodes.keys().copied() {
        if !visited.contains(&node_id) {
            if dfs_cycle(node_id, &adjacency, &mut visited, &mut stack) {
                return Err(CompileError::new(ValidationError::ExecCycle).with_node(node_id));
            }
        }
    }

    Ok(())
}

fn dfs_cycle(
    node: NodeId,
    adjacency: &BTreeMap<NodeId, Vec<NodeId>>,
    visited: &mut BTreeSet<NodeId>,
    stack: &mut BTreeSet<NodeId>,
) -> bool {
    visited.insert(node);
    stack.insert(node);

    if let Some(neighbors) = adjacency.get(&node) {
        for &next in neighbors.iter() {
            if !visited.contains(&next) && dfs_cycle(next, adjacency, visited, stack) {
                return true;
            }
            if stack.contains(&next) {
                return true;
            }
        }
    }

    stack.remove(&node);
    false
}
