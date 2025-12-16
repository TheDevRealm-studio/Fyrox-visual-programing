use crate::{
    compile::CompiledGraph,
    model::{BuiltinNodeKind, DataType, NodeId, Value},
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionEvent {
    EnterNode(NodeId),
    Print(String),
}

#[derive(Debug, Default, Clone)]
pub struct InterpreterOutput {
    pub events: Vec<ExecutionEvent>,
    pub variables: BTreeMap<String, Value>,
}

pub struct Interpreter {
    compiled: CompiledGraph,
    variables: BTreeMap<String, Value>,
}

impl Interpreter {
    pub fn new(compiled: CompiledGraph) -> Self {
        Self {
            variables: compiled.variables.clone(),
            compiled,
        }
    }

    pub fn run_begin_play(&mut self) -> InterpreterOutput {
        self.run_entry(self.compiled.begin_play_entry)
    }

    pub fn run_construction_script(&mut self) -> InterpreterOutput {
        self.run_entry(self.compiled.construction_entry)
    }

    pub fn tick(&mut self, dt: f32) -> InterpreterOutput {
        // For now, only supports a single Tick node.
        if let Some(tick_node) = self.compiled.tick_entry {
            // Populate dt output for consumers (MVP: none consume it yet).
            self.variables
                .insert("__dt".to_string(), Value::F32(dt));
            self.run_from_exec_out(tick_node, "then")
        } else {
            InterpreterOutput::default()
        }
    }

    fn run_entry(&mut self, entry: Option<NodeId>) -> InterpreterOutput {
        let Some(entry_node) = entry else {
            return InterpreterOutput::default();
        };
        // Entry nodes start execution from their "then" pin.
        self.run_from_exec_out(entry_node, "then")
    }

    fn run_from_exec_out(&mut self, start_node: NodeId, exec_out_pin: &str) -> InterpreterOutput {
        let mut out = InterpreterOutput::default();

        let Some(start) = self.compiled.nodes.get(&start_node) else {
            return out;
        };
        let Some((out_pin_id, _, out_pin_type)) = start.pin(exec_out_pin) else {
            return out;
        };
        if out_pin_type != DataType::Exec {
            return out;
        }

        let mut next_exec_in_pin = self.compiled.exec_edges.get(&out_pin_id).copied();
        while let Some(exec_in_pin) = next_exec_in_pin {
            let Some(node_id) = self.pin_owner(exec_in_pin) else {
                break;
            };

            out.events.push(ExecutionEvent::EnterNode(node_id));

            let Some(node) = self.compiled.nodes.get(&node_id) else {
                break;
            };

            match node.kind {
                BuiltinNodeKind::Print => {
                    // Prefer linked input, otherwise literal property "text".
                    let text = self.read_string_input(node_id, "text")
                        .or_else(|| node.properties.get("text").and_then(|v| match v {
                            Value::String(s) => Some(s.clone()),
                            _ => None,
                        }))
                        .unwrap_or_default();

                    out.events.push(ExecutionEvent::Print(text));
                    next_exec_in_pin = self.next_exec(node_id, "then");
                }
                BuiltinNodeKind::Branch => {
                    let cond = self
                        .read_bool_input(node_id, "condition")
                        .or_else(|| node.properties.get("condition").and_then(|v| match v {
                            Value::Bool(b) => Some(*b),
                            _ => None,
                        }))
                        .unwrap_or(false);

                    next_exec_in_pin = if cond {
                        self.next_exec(node_id, "true")
                    } else {
                        self.next_exec(node_id, "false")
                    };
                }
                BuiltinNodeKind::SetVariable => {
                    let name = node
                        .properties
                        .get("name")
                        .and_then(|v| match v {
                            Value::String(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "var".to_string());

                    let expected_ty = self
                        .compiled
                        .nodes
                        .get(&node_id)
                        .and_then(|n| n.pin("value"))
                        .map(|(_, _, ty)| ty)
                        .unwrap_or(DataType::Unit);

                    let value = self
                        .read_value_input(node_id, "value")
                        .or_else(|| {
                            node.properties.get("value").and_then(|v| {
                                if v.data_type() == expected_ty {
                                    Some(v.clone())
                                } else {
                                    None
                                }
                            })
                        })
                        .unwrap_or(Value::Unit);

                    self.variables.insert(name, value);
                    next_exec_in_pin = self.next_exec(node_id, "then");
                }
                // Entry nodes are never executed here; we jump from them.
                BuiltinNodeKind::BeginPlay
                | BuiltinNodeKind::ConstructionScript
                | BuiltinNodeKind::Tick
                | BuiltinNodeKind::GetVariable
                | BuiltinNodeKind::Self_
                | BuiltinNodeKind::GetActorTransform
                | BuiltinNodeKind::SetActorTransform
                | BuiltinNodeKind::SpawnActor
                | BuiltinNodeKind::GetActorByName
                | BuiltinNodeKind::GetActorName => {
                    next_exec_in_pin = self.next_exec(node_id, "then");
                }
            }
        }

        out.variables = self.variables.clone();
        out
    }

    fn pin_owner(&self, pin_id: crate::model::PinId) -> Option<NodeId> {
        // This is O(n) but fine for MVP.
        self.compiled
            .nodes
            .iter()
            .find_map(|(node_id, node)| {
                node.pins
                    .values()
                    .any(|(id, _, _)| *id == pin_id)
                    .then_some(*node_id)
            })
    }

    fn next_exec(&self, node_id: NodeId, exec_out_name: &str) -> Option<crate::model::PinId> {
        let node = self.compiled.nodes.get(&node_id)?;
        let (out_pin, _, ty) = node.pin(exec_out_name)?;
        if ty != DataType::Exec {
            return None;
        }
        self.compiled.exec_edges.get(&out_pin).copied()
    }

    fn read_string_input(&self, node_id: NodeId, input_name: &str) -> Option<String> {
        match self.read_value_input(node_id, input_name)? {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    fn read_bool_input(&self, node_id: NodeId, input_name: &str) -> Option<bool> {
        match self.read_value_input(node_id, input_name)? {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    fn read_value_input(&self, node_id: NodeId, input_name: &str) -> Option<Value> {
        let node = self.compiled.nodes.get(&node_id)?;
        let (input_pin, _, expected_ty) = node.pin(input_name)?;

        let from_pin = *self.compiled.data_edges.get(&input_pin)?;

        let from_node = self.pin_owner(from_pin)?;
        let from_compiled = self.compiled.nodes.get(&from_node)?;

        let value = match from_compiled.kind {
            BuiltinNodeKind::GetVariable => {
                let name = from_compiled
                    .properties
                    .get("name")
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })?;
                self.variables.get(&name).cloned()?
            }
            _ => return None,
        };

        if value.data_type() != expected_ty {
            return None;
        }

        Some(value)
    }
}
