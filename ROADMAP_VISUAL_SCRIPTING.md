# Visual Scripting (Blueprints) Roadmap (Fyrox)

Date: 2025-12-15

## Goal
Build a Blueprint-style visual scripting system on top of Fyrox that:

- Lets users author gameplay logic using a node graph in the Fyrox editor.
- Saves graphs as assets/resources and runs them in exported builds.
- Is extensible from Rust (node libraries, project-specific nodes).

## Non-goals (for v1)
- Full parity with Unreal Blueprints.
- Visual shader/material graphs.
- Network replication or multiplayer-specific scripting.
- Hot-reload of graphs in shipped builds (nice-to-have later).

## Unreal-style workflow target
Unreal’s mental model is **Blueprint Class → spawn Actor → interact with World**. In Fyrox terms, this roadmap targets:

- **Blueprint Class asset**: a reusable “Actor-like” asset that defines:
  - a node hierarchy/prefab (optional for MVP)
  - a visual script graph
  - exposed variables (editable per-instance)
- **Blueprint Instance**: a scene node with a runtime component (e.g. `BlueprintComponent`) that:
  - stores per-instance variable values
  - binds to engine/world events
  - executes the compiled graph

This keeps Fyrox’s native scene graph as the source of truth while still giving an Unreal-like authoring experience.

## Editor UX target (Blueprint Editor window)
To feel like Unreal Blueprints, the editor deliverable is not “just a graph panel”:

- **Blueprint Editor** (dockable): Components tree + Viewport preview + Event Graph + Functions/Macros + Variables/Details.
- **My Blueprint panel**: Variables, Functions, Macros, Events/Dispatchers.
- **Asset Browser integration**: create/open blueprint assets; drag & drop into scenes to place instances.
- **Reuse across scenes**: one Blueprint Class asset can be placed many times, with per-instance variable overrides.

## MVP definition (first working “vertical slice”)
Deliver a prototype that can drive a simple scene using only graphs:

- **Editor UI:** a dockable “Blueprint” panel with a node canvas.
- **Graph basics:** create/delete nodes, drag links, pan/zoom, selection.
- **Exec flow:** “exec pins” (imperative flow) + typed data pins.
- **Nodes (minimum set):**
  - `Event/BeginPlay`
  - `Event/Tick(dt)`
  - `Construction Script` entry
  - `Branch` (bool)
  - `Print/Log` (string)
  - `Get Variable` / `Set Variable`
  - `Call Method` (limited: a small curated list)
- **Runtime:** attach a `BlueprintComponent` to a scene node, run graph on events.
- **Persistence:** graphs saved as assets and referenced from scenes.
- **Debug (minimal):** runtime error log + highlight last executed node.

### MVP must include “Actor + World interaction” basics
To feel like Unreal immediately, add a minimal set of “world” nodes early:

- **Actor identity:** `Self` (current node/actor), `Get Name`, `Get/Set Transform`.
- **Spawning:** `Spawn Actor (Blueprint Class)` (can be limited to a prefab/blueprint whitelist in v1).
- **Finding:** `Get Actor By Name/Tag` (simple), `Get Children`, `Get Parent`.
- **Messaging:** `Call Blueprint Event` (by reference) or `Send Message` (simple event bus).

## Existing Fyrox integration points (this repo)
- **Editor extensibility:** `editor/src/plugin.rs` (`EditorPlugin` + `EditorPluginsContainer`).
- **Docking/layout UI:** `editor/src/lib.rs` uses Fyrox GUI docking manager.
- **Command/undo model:** `editor/src/command/*` patterns (Command stack).
- **Examples of complex editor tools:** `editor/src/plugins/*` (e.g. ABM/curve/material).
- **UI-scene controller patterns:** `editor/src/ui_scene/*` shows MVC + commands.

## Proposed architecture
### High-level split
1) **Core graph crate/module (engine-agnostic):** data model, type system, validation, compilation.
2) **Editor plugin:** authoring UI (canvas, palette), undo/redo, asset management.
3) **Runtime integration:** components + event hooks + execution VM/interpreter.

### Suggested workspace layout
Choose one of these approaches (A is recommended):

- **A) New crates (clean separation):**
  - `fyrox-visual-scripting` (core graph model + compiler)
  - `fyrox-visual-scripting-runtime` (execution + engine bindings)
  - `fyrox-visual-scripting-editor` (editor plugin)

- **B) New module inside an existing crate (faster start, less clean):**
  - Add `visual_scripting/` module under `fyrox/` and editor plugin under `editor/src/plugins/`.

## Technical decisions (lock early)
### 1) Execution model
- **v0/v1:** imperative exec-flow (Unreal-like) + typed data pins.
- **Later:** optional dataflow-only nodes, coroutines/latent actions.

### 2) Reflection & API surface
- **Phase 1 (v0/v1): curated node library** (safe + stable). This is required for the MVP.
- **Phase 2 (v1+): generated nodes via reflection + whitelist**. Goal: “access everything possible” without making the engine unsafe or the editor unusable.
- **Phase 3 (later): project-level node packs** (game-specific APIs) shipped as plugins.

#### “Access everything possible” (practical strategy)
Providing nodes for *everything* usually needs a policy, not just code generation:

- **Expose-by-default is risky** (stability, foot-guns, security, undo/redo complexity in editor).
- Prefer **opt-in exposure**:
  - a Rust `NodeLibrary`/registry for functions and properties
  - optional derive/macro attributes like `#[blueprint_expose]` / `#[blueprint_category("...")]`
  - an editor-side allowlist/denylist to hide unsafe or noisy APIs
- Support **two kinds of callable nodes**:
  - **Pure/data nodes** (no side effects): safe to evaluate anytime.
  - **Impure/exec nodes** (side effects): only run when the exec pin fires.

### 3) Determinism and safety
- Deterministic execution order for graphs.
- Strict validation before run: missing links, type mismatches, cycles on exec flow.

## Milestones

### Milestone 0 — Discovery & scaffolding (1–3 days)
**Deliverables**
- Confirm MVP (events, exec-flow pins, first demo).
- Identify reusable UI/graph components in existing editor plugins.
- Create skeleton crates/modules and a “hello graph” compile/run unit test.

**Tasks**
- Audit `editor/src/plugins/*` for existing node-graph canvases (ABSM/curve/material are likely candidates).
- Decide A vs B layout (new crates vs modules).
- Define the minimal type set and value representation.

**Exit criteria**
- A small unit test that builds a graph in code and executes `BeginPlay -> Print("Hello")`.

---

### Milestone 1 — Core graph model + validation (3–7 days)
**Deliverables**
- Serializable graph model with stable IDs.
- Validation: pin compatibility, missing required links, exec-flow acyclicity.

**Tasks**
- Define:
  - `Graph { nodes, links, variables }`
  - `Node { id, kind, pins, properties }`
  - `Pin { id, direction, data_type, is_exec }`
  - `Link { from_pin, to_pin }`
- Add type system (start small): `Bool, I32, F32, String, Entity/Handle, Unit`.
- Add `compile()` step that produces an executable form (basic block graph or linear bytecode).

**Exit criteria**
- `compile()` returns structured errors with node/pin references.

---

### Milestone 2 — Runtime execution (1–2 weeks)
**Deliverables**
- Runtime engine binding: `BlueprintComponent` that can be added to scene nodes.
- Event hooks: `BeginPlay`, `Tick(dt)`.
- Execution VM/interpreter for the MVP node set.
 - Construction Script execution (once before `BeginPlay`).

**Tasks**
- Implement runtime context (access to scene, node handle, delta time, logging).
- Implement node execution for:
  - events
  - construction script (one-shot)
  - branch
  - variable get/set
  - print/log
- Implement a restricted “call method” node (explicit whitelist).
- Add minimal “Actor/World” nodes needed for Unreal-like feel:
  - `Self`, `Get/Set Transform`
  - `Spawn Actor (Blueprint Class)`
  - `Get Actor By Name/Tag` (simple)

**Exit criteria**
- A template/demo scene runs logic in a normal game build, not only in editor.

---

### Milestone 3 — Editor graph UI (1–3 weeks)
**Deliverables**
- Dockable Blueprint panel, node canvas, palette/search.
- Undo/redo for graph edits using editor command stack.
- Save/load graphs as assets/resources.
- “Blueprint Class” authoring entry point (create asset, open, set exposed variables).
 - Blueprint Editor window layout: Components + Viewport + Event Graph + Variables/Details.
 - Construction Script authoring + preview re-run on changes.

**Tasks**
- Build UI with Fyrox GUI:
  - node widgets
  - pin widgets
  - link rendering
  - selection + dragging + panning/zoom
- Integrate editor’s MVC approach:
  - UI is a projection of the graph model
  - modifications via Commands
- Asset lifecycle:
  - create blueprint asset
  - open blueprint in editor
  - reference blueprint from a scene node
  - optionally: create scene node from blueprint (spawn/instantiate shortcut)

**Exit criteria**
- You can author a graph in the editor, save it, reopen editor, and it still runs.

---

### Milestone 4 — Debugging and diagnostics (1–2 weeks)
**Deliverables**
- Graph validation panel with clickable errors.
- Live execution highlighting (current node) + debug-object selection.
- Breakpoints + step/continue (optional v1, recommended v2).

**Tasks**
- Runtime debug events emitted by VM (node entered/exited, errors).
- Editor subscribes to debug stream when in Play mode.

**Exit criteria**
- When a graph fails, user sees where and why (node/pin).

---

### Milestone 5 — Extensibility + stabilization (ongoing)
**Deliverables**
- Node registry API for projects/plugins.
- Versioned serialization and migrations.
- Tests for compile/run and serialization round-trips.
- Optional reflection-driven node generation with editor allowlist.

**Tasks**
- Add `NodeLibrary` registration from Rust.
- Add categories, keywords, and signatures for palette/search.
- Add reflection helpers (opt-in) to generate nodes from exposed APIs.
- Add migrations `vN -> vN+1` for graph format.
- Add CI-friendly tests (no GPU needed).

**Exit criteria**
- Projects can define custom nodes without forking the editor.

## Risks / unknowns
- **Graph UI effort:** node canvas UX can take longer than runtime.
- **API exposure:** “everything as nodes” can overwhelm UX and introduce unsafe operations; enforce opt-in exposure and a whitelist.
- **Asset system integration:** ensure graphs are first-class resources and can be referenced by scenes/templates.

## Recommended next action
1) Lock the MVP choice: gameplay Blueprints + exec-flow pins + runtime builds.
2) Identify the best existing graph/canvas code to reuse in `editor/src/plugins/*`.
3) Implement Milestone 1 as a small new crate with tests.

See also: `VISUAL_SCRIPTING_BUILD_PLAN.md` for a more detailed, Unreal-style Actor/components implementation plan.
