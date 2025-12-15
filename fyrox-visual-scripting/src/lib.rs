#![forbid(unsafe_code)]

pub mod compile;
pub mod error;
pub mod interpret;
pub mod model;

pub use crate::{
    compile::{compile, CompiledGraph},
    error::{CompileError, ValidationError},
    interpret::{ExecutionEvent, Interpreter, InterpreterOutput},
    model::{
        BlueprintGraph, BuiltinNodeKind, DataType, GraphDef, GraphId, GraphKind, Link, Node, NodeId,
        Pin, PinDirection, PinId, Value,
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_play_prints_hello() {
        let mut graph = BlueprintGraph::new(GraphId("test".to_string()));

        let begin_play = graph.add_node(Node::new(BuiltinNodeKind::BeginPlay));
        let print = graph.add_node(Node::new(BuiltinNodeKind::Print));

        // Set Print literal.
        graph.nodes
            .get_mut(&print)
            .unwrap()
            .set_property_string("text", "Hello".to_string());

        let begin_then = graph
            .nodes
            .get(&begin_play)
            .unwrap()
            .pin_named("then")
            .unwrap();
        let print_exec_in = graph.nodes.get(&print).unwrap().pin_named("exec") .unwrap();

        graph.add_link(Link::exec(begin_then, print_exec_in));

        let compiled = compile(&graph).expect("compile");
        let mut interpreter = Interpreter::new(compiled);
        let out = interpreter.run_begin_play();

        assert!(out
            .events
            .iter()
            .any(|e| matches!(e, ExecutionEvent::Print(s) if s == "Hello")));
    }

    #[test]
    fn construction_script_runs() {
        let mut graph = BlueprintGraph::new(GraphId("test".to_string()));

        let construction = graph.add_node(Node::new(BuiltinNodeKind::ConstructionScript));
        let print = graph.add_node(Node::new(BuiltinNodeKind::Print));

        graph.nodes
            .get_mut(&print)
            .unwrap()
            .set_property_string("text", "Built".to_string());

        let then_pin = graph
            .nodes
            .get(&construction)
            .unwrap()
            .pin_named("then")
            .unwrap();
        let print_exec_in = graph.nodes.get(&print).unwrap().pin_named("exec").unwrap();
        graph.add_link(Link::exec(then_pin, print_exec_in));

        let compiled = compile(&graph).expect("compile");
        let mut interpreter = Interpreter::new(compiled);
        let out = interpreter.run_construction_script();

        assert!(out
            .events
            .iter()
            .any(|e| matches!(e, ExecutionEvent::Print(s) if s == "Built")));
    }
}
