use crate::{
    compile::CompiledNode,
    interpret::{Interpreter, InterpreterOutput},
    model::{NodeId, PinId, Value},
    runtime::NodeRuntime,
};

pub struct BranchRuntime;

impl NodeRuntime for BranchRuntime {
    fn execute(
        &self,
        interpreter: &mut Interpreter,
        _out: &mut InterpreterOutput,
        node_id: NodeId,
        node: &CompiledNode,
    ) -> Option<PinId> {
        let condition = interpreter
            .read_bool_input(node_id, "condition")
            .or_else(|| {
                node.properties.get("condition").and_then(|v| match v {
                    Value::Bool(b) => Some(*b),
                    _ => None,
                })
            })
            .unwrap_or(false);

        if condition {
            interpreter.next_exec(node_id, "true")
        } else {
            interpreter.next_exec(node_id, "false")
        }
    }
}
