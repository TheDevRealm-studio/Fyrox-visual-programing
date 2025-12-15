use crate::model::{NodeId, PinId};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("link refers to unknown pin")]
    UnknownPin,

    #[error("pin direction mismatch")]
    DirectionMismatch,

    #[error("pin type mismatch")]
    TypeMismatch,

    #[error("multiple exec inputs connected")]
    MultipleExecInputs,

    #[error("no entry node for {0}")]
    MissingEntry(&'static str),

    #[error("exec flow cycle detected")]
    ExecCycle,

    #[error("broken exec link")]
    BrokenExecLink,

    #[error("link crosses graphs")]
    CrossGraphLink,

    #[error("duplicate variable name")]
    DuplicateVariable,

    #[error("unknown variable")]
    UnknownVariable,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("compile failed: {kind} (node={node:?} pin={pin:?})")]
pub struct CompileError {
    pub kind: ValidationError,
    pub node: Option<NodeId>,
    pub pin: Option<PinId>,
}

impl CompileError {
    pub fn new(kind: ValidationError) -> Self {
        Self {
            kind,
            node: None,
            pin: None,
        }
    }

    pub fn with_node(mut self, node: NodeId) -> Self {
        self.node = Some(node);
        self
    }

    pub fn with_pin(mut self, pin: PinId) -> Self {
        self.pin = Some(pin);
        self
    }
}
