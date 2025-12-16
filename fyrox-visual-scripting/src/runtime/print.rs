use crate::{
    compile::CompiledNode,
    interpret::{ExecutionEvent, Interpreter, InterpreterOutput},
    model::{NodeId, PinId, Value},
    runtime::NodeRuntime,
};

pub struct PrintRuntime;

impl NodeRuntime for PrintRuntime {
    fn execute(
        &self,
        interpreter: &mut Interpreter,
        out: &mut InterpreterOutput,
        node_id: NodeId,
        node: &CompiledNode,
    ) -> Option<PinId> {
        // Prefer linked input, otherwise literal property "text".
        let text = interpreter
            .read_string_input(node_id, "text")
            .or_else(|| {
                node.properties.get("text").and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
            })
            .unwrap_or_default();

        out.events.push(ExecutionEvent::Print(text));
        interpreter.next_exec(node_id, "then")
    }
}
