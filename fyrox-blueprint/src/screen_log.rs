use fyrox::{
    core::{
        reflect::prelude::*,
        visitor::prelude::*,
    },
    gui::{
        text::{TextBuilder, TextMessage},
        widget::{WidgetBuilder, WidgetMessage},
        HorizontalAlignment, Thickness, UiNode, VerticalAlignment,
    },
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
};

use std::collections::VecDeque;

use crate::{register, register_resources};

#[derive(Debug, Visit, Reflect, Default)]
#[reflect(non_cloneable)]
pub struct BlueprintScreenLogPlugin {
    #[reflect(hidden)]
    #[visit(skip)]
    text: fyrox::core::pool::Handle<UiNode>,

    #[reflect(hidden)]
    #[visit(skip)]
    lines: VecDeque<(String, f32)>,

    #[reflect(hidden)]
    #[visit(skip)]
    needs_ui_init: bool,
}

impl BlueprintScreenLogPlugin {
    pub const DEFAULT_TTL: f32 = 2.0;
    pub const MAX_LINES: usize = 6;

    pub fn push(&mut self, text: String) {
        self.lines.push_back((text, Self::DEFAULT_TTL));
        while self.lines.len() > Self::MAX_LINES {
            self.lines.pop_front();
        }
    }

    fn ensure_ui(&mut self, ctx: &mut PluginContext) {
        if self.text.is_some() {
            return;
        }

        let ui = ctx.user_interfaces.first_mut();
        let build_ctx = &mut ui.build_ctx();

        self.text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(6.0))
                .with_horizontal_alignment(HorizontalAlignment::Left)
                .with_vertical_alignment(VerticalAlignment::Top),
        )
        .build(build_ctx);

        // Attach to the UI root so it's always visible.
        ui.send(self.text, WidgetMessage::LinkWith(ui.root()));

        self.needs_ui_init = false;
    }

    fn render(&self, ctx: &mut PluginContext) {
        if self.text.is_none() {
            return;
        }

        let joined = self
            .lines
            .iter()
            .map(|(t, _)| t.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        ctx.user_interfaces.first().send(self.text, TextMessage::Text(joined));
    }
}

impl Plugin for BlueprintScreenLogPlugin {
    fn register(&self, context: PluginRegistrationContext) {
        // Ensure the runtime knows how to load `.blueprint` assets and instantiate BlueprintScript.
        register_resources(context.resource_manager);
        register(&context.serialization_context.script_constructors);
    }

    fn init(&mut self, _scene_path: Option<&str>, _context: PluginContext) {
        self.needs_ui_init = true;
    }

    fn update(&mut self, context: &mut PluginContext) {
        if self.needs_ui_init {
            self.ensure_ui(context);
        }

        if !self.lines.is_empty() {
            for (_, ttl) in self.lines.iter_mut() {
                *ttl -= context.dt;
            }
            while self.lines.front().is_some_and(|(_, ttl)| *ttl <= 0.0) {
                self.lines.pop_front();
            }
        }

        self.render(context);
    }
}
