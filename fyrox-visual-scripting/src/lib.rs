#![forbid(unsafe_code)]

pub mod compile;
pub mod error;
pub mod interpret;
pub mod model;
pub mod nodes;
mod runtime;

pub use crate::{
    compile::{compile, CompiledGraph},
    error::{CompileError, ValidationError},
    interpret::{ExecutionEvent, Interpreter, InterpreterOutput},
    model::{
        BlueprintGraph, BuiltinNodeKind, DataType, GraphDef, GraphId, GraphKind, Link, Node, NodeId,
        Pin, PinDirection, PinId, Value,
    },
    nodes::{NodeCategory, NodeDefinition, all_node_definitions, get_node_definition, pin_color_for_type},
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

    #[test]
    fn variable_connects_to_print() {
        use model::VariableDef;

        let mut graph = BlueprintGraph::new(GraphId("test".to_string()));

        // Add a string variable
        graph.variables.push(VariableDef {
            name: "message".to_string(),
            data_type: DataType::String,
            default_value: Some(Value::String("Hello from variable!".to_string())),
        });

        let begin_play = graph.add_node(Node::new(BuiltinNodeKind::BeginPlay));

        // Create GetVariable node
        let mut get_var = Node::new(BuiltinNodeKind::GetVariable);
        get_var.set_property_string("name", "message".to_string());
        let get_var_id = graph.add_node(get_var);

        let print = graph.add_node(Node::new(BuiltinNodeKind::Print));

        // Connect exec flow: BeginPlay -> Print
        let begin_then = graph
            .nodes
            .get(&begin_play)
            .unwrap()
            .pin_named("then")
            .unwrap();
        let print_exec = graph.nodes.get(&print).unwrap().pin_named("exec").unwrap();
        graph.add_link(Link::exec(begin_then, print_exec));

        // Connect data flow: GetVariable.value -> Print.text
        let var_value_out = graph.nodes.get(&get_var_id).unwrap().pin_named("value").unwrap();
        let print_text_in = graph.nodes.get(&print).unwrap().pin_named("text").unwrap();
        graph.add_link(Link::exec(var_value_out, print_text_in));

        // This should now compile successfully with dynamic typing
        let compiled = compile(&graph).expect("compile should succeed with dynamic variable typing");
        let mut interpreter = Interpreter::new(compiled);
        let out = interpreter.run_begin_play();

        assert!(out
            .events
            .iter()
            .any(|e| matches!(e, ExecutionEvent::Print(s) if s == "Hello from variable!")));
    }

    #[test]
    fn rhai_script_prints() {
        let mut graph = BlueprintGraph::new(GraphId("test".to_string()));

        let begin_play = graph.add_node(Node::new(BuiltinNodeKind::BeginPlay));

        let mut script = Node::new(BuiltinNodeKind::RhaiScript);
        script.set_property_string("code", "print(\"Hello from Rhai\");".to_string());
        let script_id = graph.add_node(script);

        let begin_then = graph
            .nodes
            .get(&begin_play)
            .unwrap()
            .pin_named("then")
            .unwrap();
        let script_exec = graph.nodes.get(&script_id).unwrap().pin_named("exec").unwrap();
        graph.add_link(Link::exec(begin_then, script_exec));

        let compiled = compile(&graph).expect("compile");
        let mut interpreter = Interpreter::new(compiled);
        let out = interpreter.run_begin_play();

        assert!(out
            .events
            .iter()
            .any(|e| matches!(e, ExecutionEvent::Print(s) if s == "Hello from Rhai")));
    }
}
