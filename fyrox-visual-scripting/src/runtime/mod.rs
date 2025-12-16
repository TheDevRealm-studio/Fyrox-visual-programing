//! Runtime execution for built-in nodes.

mod branch;
mod passthrough;
mod print;
mod rhai_script;
mod set_variable;

pub use branch::*;
pub use passthrough::*;
pub use print::*;
pub use rhai_script::*;
pub use set_variable::*;

use crate::{
    compile::CompiledNode,
    interpret::{Interpreter, InterpreterOutput},
    model::{BuiltinNodeKind, NodeId, PinId},
};

pub trait NodeRuntime: Send + Sync {
    /// Execute a node and return the next exec *input* pin to follow.
    fn execute(
        &self,
        interpreter: &mut Interpreter,
        out: &mut InterpreterOutput,
        node_id: NodeId,
        node: &CompiledNode,
    ) -> Option<PinId>;
}

static PASSTHROUGH: PassthroughRuntime = PassthroughRuntime;
static PRINT: PrintRuntime = PrintRuntime;
static BRANCH: BranchRuntime = BranchRuntime;
static SET_VARIABLE: SetVariableRuntime = SetVariableRuntime;
static RHAI_SCRIPT: RhaiScriptRuntime = RhaiScriptRuntime;

pub fn runtime_for(kind: BuiltinNodeKind) -> &'static dyn NodeRuntime {
    match kind {
        BuiltinNodeKind::Print => &PRINT,
        BuiltinNodeKind::Branch => &BRANCH,
        BuiltinNodeKind::SetVariable => &SET_VARIABLE,
        BuiltinNodeKind::RhaiScript => &RHAI_SCRIPT,

        // These either do not execute directly (entry/pure nodes) or are MVP no-ops.
        BuiltinNodeKind::BeginPlay
        | BuiltinNodeKind::Tick
        | BuiltinNodeKind::ConstructionScript
        | BuiltinNodeKind::GetVariable
        | BuiltinNodeKind::Self_
        | BuiltinNodeKind::GetActorTransform
        | BuiltinNodeKind::SetActorTransform
        | BuiltinNodeKind::SpawnActor
        | BuiltinNodeKind::GetActorByName
        | BuiltinNodeKind::GetActorName => &PASSTHROUGH,
    }
}
