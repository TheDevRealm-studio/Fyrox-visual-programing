use crate::{
    compile::CompiledNode,
    interpret::{ExecutionEvent, Interpreter, InterpreterOutput},
    model::{NodeId, PinId, Value},
    runtime::NodeRuntime,
};

pub struct RhaiScriptRuntime;

impl NodeRuntime for RhaiScriptRuntime {
    fn execute(
        &self,
        interpreter: &mut Interpreter,
        out: &mut InterpreterOutput,
        node_id: NodeId,
        node: &CompiledNode,
    ) -> Option<PinId> {
        // Keep execution exactly as before by delegating to the interpreter's existing helper.
        // This avoids duplicating Rhai bridge glue here.
        let code = interpreter
            .read_string_input(node_id, "code")
            .or_else(|| {
                node.properties.get("code").and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
            })
            .unwrap_or_default();

        if let Err(err) = interpreter.execute_rhai(&code, out) {
            out.events
                .push(ExecutionEvent::Print(format!("[Rhai error] {err}")));
        }

        interpreter.next_exec(node_id, "then")
    }
}
