# xrf-particles

Reads, writes and edits `particles.xr` libraries, and simulates their effects as the engine does; the level viewer draws
through it.

A library holds named effects and groups:

- An **effect** (`ParticleEffect`, the engine's `CPEDef`) is a list of actions (a source emitting particles, gravity, a
  kill by age, and so on) stepped over a pool of particles, drawn as sprites of one shader and texture.
- A **group** (`ParticleGroup`) plays several effects, each from its start to its stop time on the group's clock, and
  can spawn child effects when a particle is born or dies.

Used by the `xrf-cli particle` commands (`info`, `list`, `find`, `inspect`, `verify`, `unpack`, `pack`, `repack`,
`re-unpack`), `xrf-gamedata`'s `particles` and `particles-usage` checks, and the renderer and world crates that play
level and ambient effects.

## Read a library and play an effect

```rust,no_run
use glam::{Mat4, Vec3};
use xrf_chunk::XRayByteOrder;
use xrf_engine_target::XrayEngine;
use xrf_particles::{ParticleEngineRules, ParticleLibrary, ParticleObject, ParticleUpdateContext, ParticlesFile};

fn main() -> xrf_error::XrfResult {
  let file = ParticlesFile::read_from_path::<XRayByteOrder, _>(&"gamedata/particles.xr")?;

  println!("{} effects, {} groups", file.effects.effects.len(), file.groups.groups.len());

  let library = ParticleLibrary::from(file);
  let rules = ParticleEngineRules::new(XrayEngine::Vanilla, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT);
  let context = ParticleUpdateContext {
    library: &library,
    rules: &rules,
    collider: None,
  };

  if let Some(instance) = library.create("explosions\\explosion_01", 1) {
    let mut object = ParticleObject::new(instance);

    object.update_parent(&Mat4::IDENTITY, Vec3::ZERO);
    object.play(0, &context);
    object.advance(100, Vec3::ZERO, true, &context);

    for effect in object.get_instance().get_effects() {
      println!("{}: {} live particles", effect.get_definition().name, effect.get_pool().len());
    }
  }

  Ok(())
}
```

## The file

`ParticlesFile` is the whole library: a `header` (version 1), an `effects` chunk and a `groups` chunk.

- Read with `read_from_path`, `read_from_bytes`, `read_from_file` or `read_from_chunk`; write with `write_to_path` or
  `write_to`. Each takes the byte order as a type parameter (`xrf_chunk::XRayByteOrder` for game files).
- Strings are Windows-1251.

Reading follows `CPSLibrary::Load`:

- The header is required. A library without effects or without groups simply has none.
- Effects must be version 1 and groups version 3, the only versions the engine loads.
- An effect's sprite, frame, time limit, collision, velocity scale and rotation are optional, read only under their
  flags as the engine does. Two of GAMMA's effects draw no sprite.
- A group's effects and time limit are optional, reading as none and zero.

Where the engine would silently drop data, reading refuses instead, so a library writes back without loss:

- a root chunk the engine does not read;
- a first-generation chunk, which the engine skips;
- bytes left over in a chunk.

A missing required chunk or a malformed chunk is a parsing error naming it; malformed input never panics.

Write order:

- Effects and groups are sorted by name when read and written in that order, whatever order the file had.
- A group's Shadow of Chernobyl effects (`effects_old`) go to chunk 7, where that game reads them.

## Edit a library

Edits follow the SDK's particle editor:

- `ParticlesEffectsChunk` and `ParticlesGroupsChunk` look records up by exact name (`get`, `get_mut`), insert them in
  name order (`insert`), and remove them (`remove`).
- `ParticlesFile::insert_effect` and `insert_group` also keep a name unique across both lists, as the SDK does.
- `rename_effect` renames an effect and every group reference to it, children included; `rename_group` renames a group.
- `validate` lists the faults the SDK refuses to save (`ParticlesFileIssue`): an effect with no `KillOld` action, a
  group playing an undefined effect, or a child flag with no child named. No shipped library has any.

### Editor actions

An effect keeps two copies of its actions:

- `actions` — what the engine plays.
- `editor_data` — the SDK's editor actions: named, switchable property sets the SDK compiles into `actions` on every
  save. Every shipped effect carries both.

Working with the editor copy:

- `read_editor_actions` decodes it into `ParticleEditorAction`s.
- `set_editor_actions` replaces it and recompiles `actions` from the enabled actions, as the SDK saves.
- `ParticleEditorAction::new` starts an action at the SDK's defaults (`ParticleEditorSchema`).
- `get_float`, `get_vector`, `get_domain`, `get_bool`, `get_int` and `set` read and change properties by name and kind.
- `compile` derives the engine's action; `ParticleEditorDomain::compile` ports `pDomain`'s constructor.

The engine never reads editor data, so it is kept as stored and decoded only on request. A library whose editor data
does not decode still reads.

How faithful it is:

- In the vanilla, Clear Sky, CoP-EE, Anomaly, Gunslinger, GAMMA and xrf libraries, every effect's editor actions decode,
  re-encode byte for byte, and compile to its stored actions.
- CoC and SGM editor data also re-encodes byte for byte, but 1,119 of their effects compile to something other than
  what they play. Mostly, a converter wrote target colours as `Alpha, Scale, Color, TimeFrom, TimeTo`, which the SDK
  (like this crate) reads in its own order. Saving those effects from their editor data changes how they play.

### Export as text

`export_to_path` unpacks a library into `header.ltx`, `effects.ltx` and `groups.ltx` in a directory: one section per
effect and group (tagged with `$type`), with sub-sections such as `<effect>.action.<index>` and `<effect>.sprite`.
`import_from_path` reads the directory back. The records are also `serde` types, for JSON.

## Simulation

`ParticleLibrary` holds a file's effects and groups by exact, case-sensitive name.

- `create(name, seed)` returns a stopped `ParticleInstance` of the effect with that name, else the group
  (`model_CreateParticles`), or `None`. `create_effect` looks for effects only.
- The seed starts the instance's random sequence, so a run is repeatable.

Instances follow the engine's `CParticleEffect` and `CParticleGroup`:

- `play` starts it. `stop(true)` silences the sources and lets live particles finish; `stop(false)` clears them.
- `update_parent` places it in world space by a transform and passes the parent's velocity to its sources.
- `update(frame_milliseconds, context)` accumulates time and takes whole engine-length steps, at most three per update
  (99 ms at the default rate), carrying the remainder. An effect with a time limit stops itself, deferred, when it runs
  out.
- A group plays each enabled effect as its clock passes the effect's `time_0`, and stops it at `time_1`. Its time limit
  is `ParticleGroup::get_time_limit`: the file's, or the latest `time_1` when the file gives none. Zero or less loops.

`ParticleObject` wraps an instance like the engine's `CParticlesObject`:

- `play` takes a first update as if 33 ms had passed.
- `advance` updates every frame while in view, and otherwise on a distance-based schedule, from every 30 ms near the
  view to every 525 ms at 200 m and beyond.

`ParticleUpdateContext` carries what an update reads from outside: the library (for a group's effects and children),
the `ParticleEngineRules`, and an optional `ParticleCollider`.

- Without a collider, colliding effects collide with nothing.
- A collider answers the nearest surface along a ray, as `CObjectSpace::RayPick` does. It must be `Send + Sync`, since
  effects may step on several threads.

## Engine differences

`ParticleEngineRules` names the engine ([xrf-engine-target](../xrf-engine-target/README.md)) and its update
coefficient:

- OpenXRay steps every 33 ms and ignores the coefficient.
- Monolith scales the step by `particle_update_mod`, clamped to its console range of 0.04 to 10, truncating the
  milliseconds as the engine casts them.

```rust
use xrf_engine_target::XrayEngine;
use xrf_particles::ParticleEngineRules;

let vanilla = ParticleEngineRules::new(XrayEngine::Vanilla, 2.0);
let extended = ParticleEngineRules::new(XrayEngine::Extended, 0.5);

assert_eq!(vanilla.get_step_milliseconds(), 33);
assert_eq!(extended.get_step_milliseconds(), 16);
assert_eq!(ParticleEngineRules::new(XrayEngine::Extended, 0.0).get_update_coefficient(), 0.04);
```

## Flags

Effect and group flags are raw `u32` fields. `ParticleEffectFlags` and `ParticleGroupEffectFlags` read them as the
engine's `Flags32` does: `is` asks for every bit of a mask, and `list_names` names the set bits as the engine's `dfXXX`
constants do.

## Checks

Run `cargo test --locked -p xrf-particles` from the repository root. Malformed-input and editor tests build their bytes
by hand rather than through the writer, so a reader cannot pass just by agreeing with its own writer.

Sources: the [file](src/particles_file.rs), [effect](src/data/effect/particle_effect.rs),
[group](src/data/group/particle_group.rs), [editor action](src/data/editor/particle_editor_action.rs),
[library](src/simulation/particle_library.rs), [engine rules](src/simulation/particle_engine_rules.rs), and the
[malformed-input](src/tests/malformed.rs), [editor](src/tests/editor.rs) and [library editing](src/tests/library.rs)
tests.
