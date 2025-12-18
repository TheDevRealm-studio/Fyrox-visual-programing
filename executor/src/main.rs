use fyrox::{
    core::{
        log::Log,
        pool::Handle,
        reflect::prelude::*,
        visitor::prelude::*,
    },
    engine::executor::Executor,
    event_loop::EventLoop,
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    scene::Scene,
};

use std::path::Path;

fn set_working_directory_from_override_scene_arg() {
    // The editor runs `cargo run -p executor -- --override-scene <path>`.
    // We want resource scanning/loading to happen relative to the project root,
    // not necessarily the current process working directory.
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == "--override-scene" {
            if let Some(scene_path) = args.next() {
                let scene_path = Path::new(&scene_path);

                // If the scene is in `<project>/data/...`, use `<project>` as the cwd.
                let candidate_project_root = scene_path
                    .to_string_lossy()
                    .split("/data/")
                    .next()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty());

                if let Some(project_root) = candidate_project_root {
                    let _ = std::env::set_current_dir(project_root);
                } else if let Some(parent) = scene_path.parent() {
                    let _ = std::env::set_current_dir(parent);
                }

                return;
            }
        }
    }
}

#[derive(Default, Visit, Reflect, Debug)]
#[reflect(non_cloneable)]
struct Game {
    scene: Handle<Scene>,
}

impl Plugin for Game {
    fn register(&self, context: PluginRegistrationContext) {
        // Enable `.blueprint` resources and BlueprintScript deserialization.
        fyrox_blueprint::register_resources(context.resource_manager);
        fyrox_blueprint::register(&context.serialization_context.script_constructors);
    }

    fn init(&mut self, scene_path: Option<&str>, context: PluginContext) {
        // Load the scene that the editor passes via `--override-scene`.
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));
    }

    fn on_scene_begin_loading(&mut self, _path: &Path, context: &mut PluginContext) {
        if self.scene.is_some() {
            context.scenes.remove(self.scene);
        }
    }

    fn on_scene_loaded(
        &mut self,
        _path: &Path,
        scene: Handle<Scene>,
        _data: &[u8],
        _context: &mut PluginContext,
    ) {
        self.scene = scene;
    }
}

fn main() {
    Log::set_file_name("executor.log");

    set_working_directory_from_override_scene_arg();

    let event_loop = EventLoop::new().unwrap();
    let mut executor = Executor::new(Some(event_loop));
    executor.add_plugin(Game::default());
    executor.add_plugin(fyrox_blueprint::BlueprintScreenLogPlugin::default());
    executor.run()
}
