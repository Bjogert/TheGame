# NPC Module

Provides the scaffolding for non-player characters (NPCs). Currently the focus is identity data and a lightweight debug spawner so we can see placeholder villagers in the scene.

## Contents
- `components.rs` – defines `NpcId`, `Identity`, and the `NpcIdGenerator` resource.
- `plugin.rs` – wires the module into the Bevy app and spawns debug NPCs after the world environment loads.
- `systems.rs` – holds the `spawn_debug_npcs` system (and future NPC-related systems).

## Usage
- Register the plugin after `WorldPlugin`:
  ```rust
  App::new()
      .add_plugins((
          DefaultPlugins,
          CorePlugin::default(),
          WorldPlugin,
          NpcPlugin,
      ))
      .run();
  ```
- Debug NPCs use capsule meshes, stand at pre-defined positions on the ground plane, and log activity changes approximately every five seconds of simulation time.
- `Identity` carries a unique `NpcId`, display name, and placeholder age. Extend this struct as more simulation data becomes available.

## Follow-ups
- Replace debug meshes with animated GLTF assets when art is ready.
- Persist NPC identities via the planned SQLite layer (Milestone M2).
- Introduce scheduling/needs systems (S1.1b+) that read from `Identity` and future components.


