# xrf-particles

Reads and writes `particles.xr` libraries: the effects and groups the engine spawns, and the actions that drive them.
It also simulates them as the engine does, which is what the level viewer draws.

A library holds named effects and groups. An effect (`ParticleEffect`, the engine's `CPEDef`) is a list of actions,
such as a source emitting particles, gravity, or a kill by age, stepped over a pool of particles and drawn as sprites
of one shader and texture. A group (`ParticleGroup`) plays several effects, each from its own start time to its stop
time on the group's clock, and can spawn child effects on a particle's birth or death. Consumers: the `xrf-cli particle`
commands (`info`, `list`, `find`, `inspect`, `verify`, `unpack`, `pack`, `repack`, `re-unpack`), `xrf-gamedata`'s
`particles` and `particles-usage` checks, and the renderer and world crates that play level and ambient effects.

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

`ParticlesFile` is the whole library: a `header` (version 1), the `effects` chunk and the `groups` chunk. Read it with
`read_from_path`, `read_from_bytes`, `read_from_file` or `read_from_chunk`, and write it with `write_to_path` or
`write_to`; each takes the byte order as a type parameter, `xrf_chunk::XRayByteOrder` for game files. Strings are
Windows-1251.

Reading is strict. The root must hold exactly the three chunks; a first-generation particle chunk is refused as not
implemented; groups must be version 3; a required chunk that is missing, or an optional one (frame, time limit,
collision, velocity scale, description, rotation) that is malformed, is a parsing error naming it, and a chunk with
bytes left over is an error. A malformed library answers with an error, never a panic.

Effects and groups are sorted by name when read, and written in that order, so a library written back is ordered by
name whatever order it was read in.

`export_to_path` unpacks a library into `header.ltx`, `effects.ltx` and `groups.ltx` in a directory, a section per
effect and group (tagged with its `$type`) with sub-sections such as `<effect>.action.<index>` and `<effect>.sprite`;
`import_from_path` reads that directory back. The records are also `serde` types, for JSON.

## Simulation

`ParticleLibrary` holds a file's effects and groups by their exact, case-sensitive names. `create(name, seed)` answers
a stopped `ParticleInstance` of the effect by that name or else the group (`model_CreateParticles`), or `None` when the
library holds neither; `create_effect` looks for effects only. The seed starts the instance's random sequence, so a run
is repeatable.

Instances follow the engine's `CParticleEffect` and `CParticleGroup`:

- `play` starts it; `stop(true)` silences the sources and lets live particles play out, `stop(false)` clears them.
- `update_parent` places it in world space by a transform, handing its sources the parent's velocity.
- `update(frame_milliseconds, context)` accumulates time and takes whole steps of the engine's length, at most three
  per update (99 ms at the default rate), carrying the remainder. An effect with a time limit stops itself, deferred,
  when the limit runs out.
- A group plays each enabled effect as its clock passes the effect's `time_0` and stops it at `time_1`; its time limit
  is the file's, or the latest `time_1` where the file gives none. Zero or less means it loops.

`ParticleObject` wraps an instance as the engine's `CParticlesObject`: `play` takes a first update as if 33 ms had
passed, and `advance` updates it when it is in view and on a distance-paced schedule otherwise, from every 30 ms near
the view to every 525 ms at 200 m and beyond.

`ParticleUpdateContext` carries what an update reads from outside: the library (for a group's effects and children),
the `ParticleEngineRules`, and an optional `ParticleCollider`. Without a collider, colliding effects collide with
nothing; an implementation answers the nearest surface along a ray, as `CObjectSpace::RayPick` does, and must be
`Send + Sync` because effects may be stepped on several threads.

## Engines differ

`ParticleEngineRules` names the engine ([xrf-engine-target](../xrf-engine-target/README.md)) and its update
coefficient. OpenXRay steps every 33 ms. Monolith scales the step by `particle_update_mod`, clamped to its console
bounds of 0.04 to 10, truncating the milliseconds as the engine casts them; OpenXRay ignores the coefficient.

```rust
use xrf_engine_target::XrayEngine;
use xrf_particles::ParticleEngineRules;

let vanilla = ParticleEngineRules::new(XrayEngine::Vanilla, 2.0);
let extended = ParticleEngineRules::new(XrayEngine::Extended, 0.5);

assert_eq!(vanilla.get_step_milliseconds(), 33);
assert_eq!(extended.get_step_milliseconds(), 16);
assert_eq!(ParticleEngineRules::new(XrayEngine::Extended, 0.0).get_update_coefficient(), 0.04);
```

Effect and group flags are raw `u32` fields on the records; `ParticleEffectFlags` and `ParticleGroupEffectFlags` read
them as the engine's `Flags32` does, where `is` asks for every bit of a mask, and `list_names` names the bits set as the
engine's `dfXXX` constants do.

## Checks

Run `cargo test --locked -p xrf-particles` from the repository root. Malformed-input tests frame their chunks by hand
rather than through the writer, so a reader cannot pass by agreeing with its own writer.

See the [file](src/particles_file.rs), [effect](src/data/particle_effect.rs), [group](src/data/particle_group.rs),
[library](src/simulation/particle_library.rs), [engine rules](src/simulation/particle_engine_rules.rs), and
[malformed-input tests](src/tests/malformed.rs).
