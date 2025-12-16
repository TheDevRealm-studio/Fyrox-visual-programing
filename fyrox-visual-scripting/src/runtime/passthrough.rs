use crate::{
    compile::CompiledNode,
    interpret::{Interpreter, InterpreterOutput},
    model::{NodeId, PinId},
    runtime::NodeRuntime,
};

/// Default behavior: follow the `then` exec output if it exists.
pub struct PassthroughRuntime;

impl NodeRuntime for PassthroughRuntime {
    fn execute(
        &self,
        interpreter: &mut Interpreter,
        _out: &mut InterpreterOutput,
        node_id: NodeId,
        _node: &CompiledNode,
    ) -> Option<PinId> {
        interpreter.next_exec(node_id, "then")
    }
}
