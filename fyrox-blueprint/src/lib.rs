mod resource;
mod screen_log;

use fyrox::{
    core::{
        impl_component_provider,
        log::Log,
        reflect::prelude::*,
        uuid_provider,
        variable::InheritableVariable,
        visitor::prelude::*,
    },
    script::{constructor::ScriptConstructorContainer, ScriptContext, ScriptTrait},
};
use fyrox_visual_scripting::{
    compile,
    compile::CompiledGraph,
    interpret::{ExecutionEvent, Interpreter},
    BlueprintGraph,
};

use std::sync::Once;

pub use crate::resource::{register_resources, BlueprintAsset, BlueprintLoader, BlueprintResource};
pub use crate::screen_log::BlueprintScreenLogPlugin;

#[derive(Visit, Reflect)]
#[reflect(non_cloneable)]
pub struct BlueprintScript {
    /// Blueprint asset (resource) that provides the graph.
    #[visit(optional)]
    pub blueprint: InheritableVariable<Option<BlueprintResource>>,

    #[reflect(hidden)]
    #[visit(optional)]
    pub construction_ran: InheritableVariable<bool>,

    #[reflect(hidden)]
    #[visit(optional)]
    pub begin_play_ran: InheritableVariable<bool>,

    #[reflect(hidden)]
    #[visit(skip)]
    compiled: Option<CompiledGraph>,

    #[reflect(hidden)]
    #[visit(skip)]
    interpreter: Option<Interpreter>,
}

impl std::fmt::Debug for BlueprintScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlueprintScript")
            .field("blueprint", &"<resource>")
            .field("construction_ran", &*self.construction_ran)
            .field("begin_play_ran", &*self.begin_play_ran)
            .finish()
    }
}

impl Clone for BlueprintScript {
    fn clone(&self) -> Self {
        Self {
            blueprint: self.blueprint.clone(),
            construction_ran: self.construction_ran.clone(),
            begin_play_ran: self.begin_play_ran.clone(),
            compiled: None,
            interpreter: None,
        }
    }
}

impl Default for BlueprintScript {
    fn default() -> Self {
        Self {
            blueprint: Default::default(),
            construction_ran: false.into(),
            begin_play_ran: false.into(),
            compiled: None,
            interpreter: None,
        }
    }
}

impl_component_provider!(BlueprintScript);
uuid_provider!(BlueprintScript = "a4c9f660-2a5b-4e8a-b171-5213384e011b");

impl BlueprintScript {
    fn ensure_compiled(&mut self) {
        if self.interpreter.is_some() {
            return;
        }

        let Some(blueprint) = self.blueprint.clone_inner() else {
            return;
        };

        let asset_guard = blueprint.data_ref();
        let Some(asset) = asset_guard.as_loaded_ref() else {
            // Still loading or failed.
            return;
        };

        let graph_json = asset.graph_json.clone();
        if graph_json.trim().is_empty() {
            return;
        }

        let graph: BlueprintGraph = match serde_json::from_str(&graph_json) {
            Ok(graph) => graph,
            Err(err) => {
                Log::err(format!("BlueprintScript: invalid graph JSON: {err}"));
                return;
            }
        };

        let compiled = match compile(&graph) {
            Ok(compiled) => compiled,
            Err(err) => {
                Log::err(format!("BlueprintScript: compile error: {err}"));
                return;
            }
        };

        self.interpreter = Some(Interpreter::new(compiled.clone()));
        self.compiled = Some(compiled);
    }

    fn flush_events(&self, mut ctx: Option<&mut ScriptContext>, events: Vec<ExecutionEvent>) {
        for event in events {
            match event {
                ExecutionEvent::EnterNode(_) => {}
                ExecutionEvent::Print(text) => {
                    Log::info(format!("[Blueprint] {text}"));

                    if let Some(ctx) = ctx.as_deref_mut() {
                        if let Some(screen_log) =
                            ctx.plugins.of_type_mut::<BlueprintScreenLogPlugin>()
                        {
                            screen_log.push(text);
                        }
                    }
                }
            }
        }
    }

    fn run_construction(&mut self, ctx: Option<&mut ScriptContext>) {
        self.ensure_compiled();
        let Some(interpreter) = self.interpreter.as_mut() else {
            return;
        };

        let out = interpreter.run_construction_script();
        self.flush_events(ctx, out.events);
        *self.construction_ran = true;
    }

    fn run_begin_play(&mut self, ctx: Option<&mut ScriptContext>) {
        self.ensure_compiled();
        let Some(interpreter) = self.interpreter.as_mut() else {
            return;
        };

        let out = interpreter.run_begin_play();
        self.flush_events(ctx, out.events);
        *self.begin_play_ran = true;
    }

    fn run_tick(&mut self, ctx: &mut ScriptContext) {
        self.ensure_compiled();
        let Some(interpreter) = self.interpreter.as_mut() else {
            return;
        };

        let out = interpreter.tick(ctx.dt);
        self.flush_events(Some(ctx), out.events);
    }
}

impl ScriptTrait for BlueprintScript {
    fn on_init(&mut self, ctx: &mut ScriptContext) {
        // Construction Script (fresh instances). For loaded instances (save games), `on_init` might
        // be skipped by the engine; `on_start` below will handle that.
        if !*self.construction_ran {
            self.run_construction(Some(ctx));
        }
    }

    fn on_start(&mut self, ctx: &mut ScriptContext) {
        // Ensure Construction Script runs before BeginPlay.
        if !*self.construction_ran {
            self.run_construction(Some(ctx));
        }

        // BeginPlay might not run here if the blueprint resource is still loading.
        if !*self.begin_play_ran {
            self.run_begin_play(Some(ctx));
        }
    }

    fn on_update(&mut self, ctx: &mut ScriptContext) {
        // If the blueprint resource was still loading during on_start, try again on update.
        if !*self.construction_ran {
            self.run_construction(Some(ctx));
        }

        if !*self.begin_play_ran {
            self.run_begin_play(Some(ctx));
        }

        // Match typical gameplay order: no ticking before BeginPlay.
        if *self.begin_play_ran {
            self.run_tick(ctx);
        }
    }
}

/// Registers blueprint-related scripts in the given constructor container.
pub fn register(container: &ScriptConstructorContainer) {
    static REGISTER_ONCE: Once = Once::new();

    REGISTER_ONCE.call_once(|| {
        container.add::<BlueprintScript>("Blueprint Script");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use fyrox_visual_scripting::{model::BuiltinNodeKind, model::GraphId, model::Link, model::Node};

    #[test]
    fn script_compiles_from_json() {
        let mut graph = BlueprintGraph::new(GraphId("test".to_string()));

        let begin = graph.add_node(Node::new(BuiltinNodeKind::BeginPlay));
        let print = graph.add_node(Node::new(BuiltinNodeKind::Print));

        graph
            .nodes
            .get_mut(&print)
            .unwrap()
            .set_property_string("text", "Hello".to_string());

        let begin_then = graph.nodes.get(&begin).unwrap().pin_named("then").unwrap();
        let print_exec = graph.nodes.get(&print).unwrap().pin_named("exec").unwrap();
        graph.add_link(Link::exec(begin_then, print_exec));

        let json = serde_json::to_string(&graph).unwrap();

        // Ensure we can parse and compile the on-disk payload format.
        let asset = BlueprintAsset {
            version: 1,
            graph_json: json,
            prefab_path: None,
        };
        let parsed: BlueprintGraph = serde_json::from_str(&asset.graph_json).unwrap();
        assert!(compile(&parsed).is_ok());
    }
}
