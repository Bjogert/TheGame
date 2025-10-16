# NPC Module

Provides the scaffolding for non-player characters (NPCs). The current focus is identity data, a lightweight debug spawner, and baseline locomotion so placeholder villagers can move to their work areas.

## Contents
- `components.rs` - defines `NpcId`, `Identity`, scheduling data, the `NpcIdGenerator` resource, and the `NpcLocomotion` component used by movement systems.
- `plugin.rs` - wires the module into the Bevy app and spawns debug NPCs after the world environment loads.
- `systems.rs` - holds `spawn_debug_npcs`, schedule ticking, and the `drive_npc_locomotion` system.

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
- Debug NPCs use capsule meshes, start at pre-defined positions on the ground plane, and log activity changes approximately every five seconds of simulation time.
- `NpcLocomotion` steers villagers toward destinations provided by other systems (currently profession crates), moving only along the XZ plane while respecting the scaled simulation delta.
- `Identity` carries a unique `NpcId`, display name, and placeholder age. Extend this struct as more simulation data becomes available.

## Follow-ups
- Replace debug meshes with animated GLTF assets when art is ready.
- Persist NPC identities via the planned SQLite layer (Milestone M2).
- Upgrade locomotion into full navigation (pathfinding, avoidance) once the world contains more complex destinations than static crates.
