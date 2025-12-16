# Visual Scripting Build Plan (Unreal-like Actors) — Fyrox

Date: 2025-12-15

## What you want (requirements)

You want a simplified Unreal Engine workflow:

-   Create a **Blueprint Actor**.
-   Add/attach existing engine things (Camera, Physics, Lights, etc.) as **components** of that Actor.
-   Use **node-based visual scripting** to access and manipulate “everything possible” in the world.

This document describes a practical way to build that on top of this repo.

## UX target (Unreal-like, clean and simple)

Your benchmark is Unreal’s Blueprint Editor experience:

-   A dedicated **Blueprint Editor window** (dockable like the rest of Fyrox editor).
-   A **viewport** preview of the Blueprint Actor.
-   A **graph editor** with tabs: Event Graph, **Construction Script**, Function graphs, Macro graphs.
-   A **My Blueprint** panel: Graphs, Functions, Macros, Variables, Event Dispatchers, Local Variables (scoped to functions).
-   A **Details** panel to edit selected components/nodes plus Blueprint/Class defaults and per-instance overrides.

This plan focuses on delivering that UX using Fyrox’s existing docking/layout system and editor plugin architecture.

## Glossary (Fyrox mapping)

### Actor (Unreal) → Scene node (Fyrox)

-   In Fyrox, the primary “thing in the world” is a **scene graph node** (`fyrox-impl/src/scene/node/*`).
-   Actor instances are represented by a **root node handle** (`Handle<Node>`).

### Components (Unreal) → Child nodes + scripts

-   Many “components” in Fyrox are expressed as dedicated node types (Camera, RigidBody, Light, etc.).
-   Additional behavior is attached using **scripts** (`fyrox-impl/src/script/mod.rs`), stored on the base node (`fyrox-impl/src/scene/base.rs`).

### Blueprint Class (Unreal) → Blueprint asset (Prefab + Graph)

A **Blueprint Class asset** should bundle:

-   A **prefab** (node hierarchy) that contains the Actor’s “components” as child nodes.
-   A **visual script graph**.
-   **Exposed variables** (editable per instance).

At runtime, spawning a Blueprint Class means instantiating its prefab and attaching a script that executes the graph.

## Editor workflow (what the user will actually do)

### Create a Blueprint Actor asset

1. In **Asset Browser**, user creates `MyActor.blueprint` (new asset type).
2. Double-click opens **Blueprint Editor**.

### Add components (existing engine node types)

In the Blueprint Editor:

-   Left panel shows a **Components tree** (the prefab hierarchy).
-   “Add Component” uses the same constructors as the main **Create** menu (`editor/src/menu/create.rs` via `SerializationContext.node_constructors`).
-   Adding a Camera/Light/RigidBody/etc. just adds the corresponding node to the prefab hierarchy.

### Script logic (Event Graph)

In the Blueprint Editor:

-   User wires `BeginPlay` / `Tick` to nodes.
-   User reads/writes variables.
-   User manipulates components and the world.

### Reuse across scenes (like Unreal)

From **Asset Browser**:

-   Drag & drop the `.blueprint` asset into a scene viewport to **place an instance** (same mental model as dragging a model; Fyrox already supports drag-preview for assets in `editor/src/scene/mod.rs`).
-   Instances can be placed in any scene.
-   Instances can override exposed variables per instance.

## Blueprint Editor layout ("video editor" style)

Use a 4-pane layout similar to Unreal:

-   **Left: Components**
    -   prefab hierarchy tree
    -   add/remove/reorder child nodes
-   **Center-top: Viewport**
    -   renders a preview scene containing the prefab instance
    -   simple orbit camera + selection
-   **Center-bottom: Graphs (tabbed)**
    -   tabs: **Event Graph**, **Construction Script** (optional v1), **Functions**, **Macros**
    -   node canvas: pan/zoom, box select, drag wires, reroute nodes
    -   right-click add-node menu + searchable palette
-   **Right: My Blueprint + Details**
    -   My Blueprint: Graphs, Functions, Macros, Variables, Event Dispatchers
    -   Details: selected component/node properties + class defaults + instance overrides

## Blueprint Editor feature checklist (explicit)

This is the minimum feature set to “feel like Unreal Blueprints” (simplified):

### Left: Components

-   Components tree (prefab hierarchy)
-   Add component (uses `SerializationContext.node_constructors`)
-   Rename/reparent/reorder child nodes
-   Search/filter components by name

### Center: Viewport

-   Preview instance of the Blueprint Actor
-   Select components by clicking in viewport (sync selection with Components tree)
-   Basic navigation (orbit/pan/zoom)

### Center: Graphs

-   **Event Graph**
    -   events: BeginPlay, Tick
-   **Construction Script** (MVP requirement)
    -   runs in-editor when the Blueprint preview instance is created or when a relevant value changes
    -   runs at runtime immediately after the Blueprint is spawned/instantiated
    -   intended for setting up components (materials, light intensity, meshes, etc.)
-   **Functions**
    -   create/rename/delete functions
    -   function inputs/outputs
    -   **Local variables** scoped to a function
-   **Macros**
    -   create/rename/delete macros
    -   macro inputs/outputs
    -   macro expansion/inlining at compile time

### Right: My Blueprint

-   Variables list
    -   add/rename/delete
    -   type selection
    -   default value editing
    -   category grouping
-   Event Dispatchers
    -   declare dispatcher (signature)
    -   nodes: Bind, Unbind, Call/Broadcast
-   Graphs list
    -   Event Graph, Construction Script, Functions, Macros

### Right: Details (Unreal-like)

Details must support both **class defaults** and **instance overrides**:

-   **Actor/Blueprint settings** (v1 subset)
    -   Enable Tick
    -   Tick interval
    -   Start with Tick enabled
    -   (later) replication flags, networking, tags
-   Selected component/node properties (existing Fyrox inspector)
-   Exposed variable defaults and instance overrides
    -   Reset to default

### Middle graph UX (stunning but simple)

Keep it “clean Unreal-like” without inventing new theming:

-   crisp node layout, good spacing
-   consistent pin alignment
-   minimal type coloring
-   readable connection wires (hover highlights)
-   clear error states (red underline/badge + tooltip)

Implementation notes (Fyrox editor consistency):

-   Build this as an `EditorPlugin` panel using Fyrox docking manager (`editor/src/lib.rs`).
-   Reuse existing inspector/property editors where possible.
-   Follow MVC used across the editor: the graph/prefab is the model; UI is a projection.

## “Great visuals” without inventing a new theme

Keep visuals consistent with the existing Fyrox editor style:

-   Reuse the current `Style`/theme resources.
-   Reuse common UI patterns (dock tabs, tree views, inspector).
-   For the graph canvas, keep it clean:
    -   minimal pin colors by type
    -   readable alignment + spacing
    -   clear selection/hover states
    -   avoid over-decorated nodes in v1

## Key design choices (lock early)

### 1) Execution style

-   Use **exec-flow pins** (Unreal-like) for predictable side-effects.
-   Keep typed data pins for values.

### 2) “Access everything possible” (the reality)

Rust doesn’t let you safely call arbitrary methods through reflection.

So “everything as nodes” needs a policy:

-   **Properties (fields):** can be accessed generically via Fyrox `Reflect` (safe-ish).
-   **Methods / actions:** must be **explicitly exposed** (opt-in) via a registry/macro + allowlist.

This gives you broad power without turning the editor into an unusable wall of nodes or exposing unsafe engine internals.

### 3) Reuse and per-instance overrides (Blueprint instances)

Unreal relies heavily on:

-   placing Blueprint instances in many scenes
-   overriding exposed variables per instance
-   updating the Blueprint class and optionally pushing changes to instances

Fyrox already has **inheritance / inheritable properties** patterns (see `InheritableVariable` usage in scripts and scene systems).

Plan for v1:

-   Blueprint instance stores:
    -   reference to the Blueprint asset
    -   per-instance overrides for exposed variables
-   Updating a Blueprint class:
    -   v1: require manual “Reapply Blueprint” action (safe and explicit)
    -   later: auto-detect changes and prompt to update instances

## Architecture overview

Build three layers:

1. **Core graph** (engine-agnostic): graph model, pins, links, validation, compilation.
2. **Runtime execution** (engine binding): a script that executes compiled graphs against the world.
3. **Editor authoring**: Blueprint asset editor (components + graph), plus Create menu integration.

## Data model (Blueprint asset)

Create a new asset/resource type (recommended) — conceptually:

-   `BlueprintAsset`:
    -   `prefab_path`: points to a prefab/model resource (node hierarchy).
    -   `graph`: visual scripting graph data.
    -   `exposed_vars`: schema + defaults.
    -   `version`: for migrations.

### Exposed variables (Unreal-like)

Provide two layers:

-   **Defaults** stored in `BlueprintAsset` (editable in Blueprint Editor).
-   **Overrides** stored on each instance (editable in the main scene inspector).

Variable UX:

-   My Blueprint panel: add/rename/delete variables, set type, set default.
-   Details panel: show instance overrides with “Reset to Default”.

### Prefab choice

You already have prefab-like flows in the editor (instantiate models, “save selection as prefab”).

Blueprints can reuse this idea:

-   **Edit-time:** user edits a node hierarchy that will be saved as the prefab.
-   **Run-time:** engine instantiates that hierarchy.

## Runtime execution (BlueprintScript)

Use Fyrox scripts to execute graphs.

-   Implement `BlueprintScript` that implements `ScriptTrait` (`fyrox-impl/src/script/mod.rs`).
-   Store it on the Actor root node’s `Base.scripts` (`fyrox-impl/src/scene/base.rs`).

### Required script lifecycle hooks

-   `on_init`: resolve references, load blueprint asset, build caches.
-   `on_start`: fire `BeginPlay`.
-   `on_update`: fire `Tick(dt)`.
-   `on_message`: optional event/messaging integration.

### Construction Script execution (MVP)

Construction Script needs two execution paths:

-   **Editor preview path:** when editing a Blueprint asset, changes to exposed variables and component properties should re-run Construction Script on the preview instance.
-   **Runtime path:** when a Blueprint instance is created (placed in scene load or spawned), Construction Script runs once before `BeginPlay`.

Rule of thumb (v1): keep Construction Script side-effects limited to the Blueprint’s own prefab hierarchy.

### Runtime context provided to nodes

Define a runtime context object that nodes can access:

-   `scene` access
-   `self_handle` (`Handle<Node>`)
-   time (`dt`)
-   logging
-   resource manager access (optional)

## Node library (what nodes exist)

Start with a small, good set that enables real gameplay.

### Category: Events

-   `BeginPlay`
-   `Tick(dt)`
-   (later) `OnCollision`, `OnTriggerEnter`, `OnInputAction`.

### Category: Component access (Unreal-like)

To manipulate attached “components”, add a simple access pattern:

-   `Get Component By Name` (child node lookup in the prefab hierarchy)
-   `Get Component By Type` (later; requires a safe querying story)

Then combine with property nodes:

-   `Set Property (Reflect Path)` on the returned component handle

### Category: Actor

-   `Self`
-   `Get Name`
-   `Get/Set Transform` (position/rotation/scale)
-   `Get Parent`, `Get Children`

### Category: World

-   `Spawn Actor (Blueprint)`
-   `Destroy Actor`
-   `Find Actor By Name/Tag`

### Category: Variables

-   `Get Variable`
-   `Set Variable`

### Category: Flow

-   `Branch`
-   (later) `Sequence`, `ForEach`, `Delay` (latent)

### Category: Properties (generic)

This is the “access everything” lever for _fields_:

-   `Get Property (Reflect Path)`
-   `Set Property (Reflect Path)`

Where “Reflect Path” can be a string like `"local_transform.position.x"`.

Practical constraints:

-   Support this first for `Self` and direct node handles.
-   Add caching (resolve path once at compile time).

### Category: Methods / Actions (exposed)

Because method calls are not generically reflectable, use:

-   `Call Function (Registered)`

Backed by a Rust registry:

-   `NodeLibrary` registers functions with:
    -   name/category
    -   inputs/outputs
    -   whether it is `pure` or `impure`
    -   an implementation closure that receives the runtime context

## Editor experience (Blueprint authoring)

### 1) Creating nodes/components like your screenshot

Your screenshot shows the **Create** menu that creates various node types.

That menu is built from constructors in `editor/src/menu/create.rs` using `SerializationContext.node_constructors`.

Blueprint authoring should let you:

-   Create a new Blueprint asset.
-   Open it in a Blueprint editor.
-   Add “components” by adding child nodes (Camera/Physics/Light/etc.) using the same constructors.

### 2) Blueprint Editor layout (recommended)

Keep it Unreal-like and simple:

-   **Left:** Components (prefab tree)
-   **Center-top:** Viewport preview
-   **Center-bottom:** Event Graph
-   **Right:** My Blueprint + Details

### 3) Blueprint instances in scenes

Add a new Create menu entry:

-   `Create → Blueprint Actor...`

Flow:

-   User selects a Blueprint asset.
-   Editor instantiates the prefab into the current scene.
-   Root node gets `BlueprintScript` attached and a reference to the asset.

Also support Unreal-like placement:

-   Drag & drop the `.blueprint` asset from **Asset Browser** into the viewport.
-   Show a placement preview (the editor already has this pattern for models in `editor/src/scene/mod.rs`).

Implementation note:

-   You can implement this either by:
    -   adding a custom item in `editor/src/menu/create.rs`, or
    -   registering a new node constructor variant into the engine’s `SerializationContext`.

## Debugging (minimum)

For Unreal-like debugging, plan for two tiers:

### Debug v1 (must-have)

-   Validate graphs in editor and show errors by node.
-   During play, highlight the currently executing node and the last executed wire.
-   Log runtime errors with node + pin references.
-   Select a **Debug Object** (which Blueprint instance you’re inspecting) and show its variable values.

### Debug v2 (strongly recommended)

-   Breakpoints on nodes
-   Step / Continue / Stop
-   Watch window for variables
-   Call stack (current function/macro)

## Safety and scalability rules

To keep “access everything” usable:

-   Use **curated nodes** for common tasks.
-   Use **generic property nodes** for the long tail.
-   Use **opt-in registry** for methods.
-   Add an **editor allowlist** to hide unsafe/noisy functions.

## Step-by-step build sequence (recommended)

### Step 1 — Core graph + unit tests

-   Implement graph model, validation, and a tiny interpreter.
-   Add a unit test: `BeginPlay → Print("Hello")`.

### Step 2 — BlueprintScript runtime

-   Implement `BlueprintScript: ScriptTrait`.
-   Run graph on `BeginPlay` and `Tick`.
-   Run Construction Script once before `BeginPlay`.

### Step 3 — Actor/world nodes

-   Implement `Self`, `Get/Set Transform`, `Spawn Blueprint`.

### Step 4 — Editor: graph panel

-   Create dock panel, canvas, node palette.
-   Support saving/loading graph data.

### Step 5 — Editor: Blueprint asset (prefab + graph)

-   Add Blueprint asset type.
-   Add Blueprint editor UI with components tree + graph.
-   Add viewport preview panel (preview scene containing the prefab instance).
-   Add My Blueprint panel for variables/functions/macros/dispatchers.
-   Add Details panel with Actor tick settings + defaults/overrides.
-   Re-run Construction Script when editing (preview instance refresh on changes).

### Step 6 — Create menu integration

-   Add `Create → Blueprint Actor...`.
-   Instantiate prefab + attach script.

### Step 7 — “Access everything” expansion

-   Add `Get/Set Property (Reflect Path)` nodes with compile-time caching.
-   Add function registry + `Call Function (Registered)` node.

## Open questions (answer to finalize the exact plan)

1. Do you want Blueprints to be editable only in-editor, or also modifiable at runtime?
2. Should Blueprint Actors be saved as a dedicated `.blueprint` asset, or reuse existing prefab formats plus a sidecar graph file?
3. Which 3–5 engine systems must be accessible first: Physics, Animation, UI, Audio, Navigation?

4. For v1, is “Reapply Blueprint to Instances” acceptable as a manual action (instead of automatic live updating)?
