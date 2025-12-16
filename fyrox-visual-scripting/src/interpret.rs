use crate::{
    compile::CompiledGraph,
    model::{BuiltinNodeKind, DataType, NodeId, Value},
    runtime::runtime_for,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rhai::{Dynamic, Engine, EvalAltResult, FLOAT, INT};

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

fn value_to_dynamic(v: &Value) -> Dynamic {
    match v {
        Value::Bool(b) => (*b).into(),
        Value::I32(i) => (INT::from(*i)).into(),
        Value::F32(f) => (FLOAT::from(*f)).into(),
        Value::String(s) => s.clone().into(),
        Value::Unit => Dynamic::UNIT,
    }
}

fn dynamic_to_value(v: &Dynamic) -> Option<Value> {
    if v.is_unit() {
        return Some(Value::Unit);
    }
    if v.is::<bool>() {
        return Some(Value::Bool(v.clone_cast::<bool>()));
    }
    if v.is::<INT>() {
        let i = v.clone_cast::<INT>();
        return i32::try_from(i).ok().map(Value::I32);
    }
    if v.is::<FLOAT>() {
        let f = v.clone_cast::<FLOAT>();
        return Some(Value::F32(f as f32));
    }
    if v.is::<String>() {
        return Some(Value::String(v.clone_cast::<String>()));
    }

    None
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

            let Some(node) = self.compiled.nodes.get(&node_id).cloned() else {
                break;
            };

            next_exec_in_pin = runtime_for(node.kind).execute(self, &mut out, node_id, &node);
        }

        out.variables = self.variables.clone();
        out
    }

    pub(crate) fn execute_rhai(
        &mut self,
        code: &str,
        out: &mut InterpreterOutput,
    ) -> Result<(), Box<EvalAltResult>> {
        // Shared buffers for Rhai callbacks.
        let emitted = Arc::new(Mutex::new(Vec::<ExecutionEvent>::new()));
        let vars = Arc::new(Mutex::new(self.variables.clone()));

        let mut engine = Engine::new();

        // Blueprint-integrated print.
        {
            let emitted = emitted.clone();
            engine.register_fn("print", move |text: &str| {
                if let Ok(mut events) = emitted.lock() {
                    events.push(ExecutionEvent::Print(text.to_string()));
                }
            });
        }

        // Variable bridge.
        {
            let vars = vars.clone();
            engine.register_fn("get_var", move |name: &str| -> Dynamic {
                let Ok(vars) = vars.lock() else {
                    return Dynamic::UNIT;
                };
                vars.get(name)
                    .map(value_to_dynamic)
                    .unwrap_or(Dynamic::UNIT)
            });
        }
        {
            let vars = vars.clone();
            engine.register_fn("set_var", move |name: &str, value: Dynamic| {
                let Some(v) = dynamic_to_value(&value) else {
                    return;
                };
                if let Ok(mut vars) = vars.lock() {
                    vars.insert(name.to_string(), v);
                }
            });
        }
        {
            let vars = vars.clone();
            engine.register_fn("dt", move || -> FLOAT {
                let Ok(vars) = vars.lock() else {
                    return FLOAT::from(0.0_f32);
                };
                match vars.get("__dt") {
                    Some(Value::F32(f)) => FLOAT::from(*f),
                    _ => FLOAT::from(0.0_f32),
                }
            });
        }

        let result: Result<Dynamic, Box<EvalAltResult>> = engine.eval(code);

        if let Ok(vars_guard) = vars.lock() {
            self.variables = vars_guard.clone();
        }

        if let Ok(mut events) = emitted.lock() {
            out.events.extend(events.drain(..));
        }

        result.map(|_| ())
    }

    pub(crate) fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub(crate) fn input_pin_type(&self, node_id: NodeId, input_name: &str) -> Option<DataType> {
        let node = self.compiled.nodes.get(&node_id)?;
        let (_, _, ty) = node.pin(input_name)?;
        Some(ty)
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

    pub(crate) fn next_exec(
        &self,
        node_id: NodeId,
        exec_out_name: &str,
    ) -> Option<crate::model::PinId> {
        let node = self.compiled.nodes.get(&node_id)?;
        let (out_pin, _, ty) = node.pin(exec_out_name)?;
        if ty != DataType::Exec {
            return None;
        }
        self.compiled.exec_edges.get(&out_pin).copied()
    }

    pub(crate) fn read_string_input(&self, node_id: NodeId, input_name: &str) -> Option<String> {
        match self.read_value_input(node_id, input_name)? {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn read_bool_input(&self, node_id: NodeId, input_name: &str) -> Option<bool> {
        match self.read_value_input(node_id, input_name)? {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub(crate) fn read_value_input(&self, node_id: NodeId, input_name: &str) -> Option<Value> {
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
