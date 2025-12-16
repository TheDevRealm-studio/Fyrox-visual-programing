use crate::{
    compile::CompiledNode,
    interpret::{Interpreter, InterpreterOutput},
    model::{DataType, NodeId, PinId, Value},
    runtime::NodeRuntime,
};

pub struct SetVariableRuntime;

impl NodeRuntime for SetVariableRuntime {
    fn execute(
        &self,
        interpreter: &mut Interpreter,
        _out: &mut InterpreterOutput,
        node_id: NodeId,
        node: &CompiledNode,
    ) -> Option<PinId> {
        let name = node
            .properties
            .get("name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "var".to_string());

        let expected_ty = interpreter
            .input_pin_type(node_id, "value")
            .unwrap_or(DataType::Unit);

        // Prefer linked input, otherwise literal property "value".
        let value = interpreter
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

        interpreter.set_variable(name, value);
        interpreter.next_exec(node_id, "then")
    }
}
