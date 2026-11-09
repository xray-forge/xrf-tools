# xrf-environment

Reads a game's environment configs, everything under `configs\environment`, the way its engine reads them:

- weather cycles and weather effects;
- suns and their flares;
- thunderbolts;
- ambients, with their sound channels and effects;
- Anomaly's sun table;
- which cycles each level plays.

It also mixes a cycle at a time of day as the engine does; the level viewer's weather plays through it.

```rust,no_run
use xrf_engine_target::XrayEngine;
use xrf_environment::{EnvironmentCatalog, EnvironmentReader, WeatherDescriptor};
use xrf_ltx::LtxProject;

fn main() -> xrf_error::XrfResult {
  let project = LtxProject::open_at_path("gamedata/configs")?;
  let catalog: EnvironmentCatalog = EnvironmentReader::read(&project, XrayEngine::Vanilla)?;

  // Every problem found, placed at its file, section and key.
  for finding in &catalog.findings {
    println!("{:?} {}: {}", finding.rule, finding.file, finding.message);
  }

  // A keyframe as authored, and as the engine holds it once loaded.
  if let Some(cycle) = catalog.find_cycle("default_clear") {
    let first = WeatherDescriptor::new(&cycle.keyframes[0], catalog.engine);

    println!("{} keyframes, the first at {} s", cycle.keyframes.len(), first.time);
  }

  Ok(())
}
```

## Read the catalog

`EnvironmentReader::read` reads every environment config of an [xrf-ltx](../xrf-ltx/README.md) project into one
`EnvironmentCatalog`, resolving each under the project's dialect.

- A config that is missing or will not resolve is a finding, not an error. `read` fails only when the project cannot be
  listed or a path cannot be built.
- `read_opt` takes `EnvironmentReadOptions`: the engine, a `JobHandle` for cancellation, and `is_explained`.
- `read_cycle` reads one cycle or effect, with the findings of its own config.
- `read_level_weather` answers the cycles a level can be lit under: its `game.ltx` section's `weathers`, or `[default]`
  when it has none, resolved against the weather graphs.

Discovery rules:

- DLTX attachments (`mod_` files) are not cycles; they patch a base cycle.
- A file under `environment\ambients` whose sections `ambients.ltx` already reaches through `#include` is a shared
  ambient, not a level's.

## What the catalog holds

One catalog serves a viewer that plays a cycle, an editor that changes it, and a verifier that reports on it.

- Each section is kept as authored (`EnvironmentSection`): the keys the engine reads, parsed as it parses them, and the
  rest as written.
- With `is_explained`, each value also records where it came from. This resolves every config afresh instead of
  through the project's cache, so leave it off when you only check.
- `WeatherDescriptor` is a keyframe as the engine holds it after loading: in radians, with every default applied.

## Grounded in the engine

Each section kind declares its keys once, with `declare_environment_keys!`: how the engine parses each
(`EnvironmentValueKind`) and, per engine, whether it reads it and its default (`EnvironmentKeyUse`: required, optional,
conditional, or unread). The reader, the findings and an editor all use that one table.

Values are stored as the engine holds them. `CInifile` never refuses a value it can read at all, so a malformed number
keeps whatever `atof` takes from it and a short vector holds zeros for its missing components. The finding says what was
written.

Findings come in three rules:

| Rule                          | Meaning                                                              |
| ----------------------------- | -------------------------------------------------------------------- |
| `EnvironmentRule::Engine`     | The engine refuses it, or loads something other than what is written |
| `EnvironmentRule::Reference`  | A section names another the engine cannot find                       |
| `EnvironmentRule::Convention` | It loads, but breaks a convention strongly enough to be a mistake    |

Convention covers the engine's own `! Invalid` colour warnings, values it clamps, and unevenly written flare lists.

The engine loads suns, thunderbolt collections, thunderbolts, ambients, sound channels and ambient effects only when a
keyframe names them, directly or through another definition. A finding in a definition nothing names is demoted to
`Convention`, with "nothing references it" added to its message.

Parsing details matching the engine:

- Lists split as `_GetItemCount` counts them: `a,` is one item, and counting stops at a doubled comma, so `a,,b` is `a`
  and an empty item.
- An ambient's channels are `AmbientKey::list_channels`: the list under the spelling its engine reads. A Shadow of
  Chernobyl ambient, which OpenXRay plays as its own channel, uses the `sound_channels.ltx` section of its own name.
- The strike settings' `altitude` is a range on OpenXRay and one number on Monolith
  (`ThunderboltSettingsKey::get_altitude`). Both clamp `min_dist_factor` to `MAX_DIST_FACTOR`.

Whether textures, sounds and models exist is checked by `xrf-gamedata`, which holds the asset index; its `environment`
check reports these findings by rule.

## Two engines

`XrayEngine::Vanilla` is OpenXRay and stock Call of Pripyat; `XrayEngine::Extended` is Anomaly's Monolith (see
[xrf-engine-target](../xrf-engine-target/README.md)). The caller names the engine; it is never inferred here.

They disagree on:

- which keys a keyframe needs, and which extra keys mean anything;
- where the sun comes from: keyframe angles, or `sun_positions.ltx`;
- whether `system.ltx` stands in for a missing definition;
- how a level's `weathers` key resolves.

## Mix and play weather

`WeatherMixer` mixes a cycle's keyframes at a `WeatherMixPoint` (a time of day and a view position) into a `WeatherMix`,
as `CEnvironment::lerp` does.

- `mix` picks the keyframes around the time with `select` (the engine's `SelectEnvs` on a forced start) and mixes them.
- `mix_pair` mixes a pair the caller keeps from frame to frame, as the engine does.
- `select_by` selects over anything that stands at a time of day.
- The sun comes from a `WeatherSunSource`: lerped between the keyframes' directions, OpenXRay's astronomical sun, or
  Monolith's hourly table.
- `WeatherModifier`s (a level's `level.env_mod` volumes) are weighted as `CEnvModifier::sum` does. The caller supplies
  them, so this crate does not depend on the level format.

`playback` steps the rest of what the engine plays: the keyframe pair being blended (`WeatherPair`,
`WeatherPlayedKeyframe`) and a weather effect laid over the cycle (`WeatherEffectTimeline`, `WeatherEffectStart`).

## Module map

| Module       | Holds                                                                                      |
| ------------ | ------------------------------------------------------------------------------------------ |
| `section`    | One section as authored and its reader; `key` the key vocabulary, `finding` what was wrong |
| `weather`    | Cycles, effects, keyframes, and a keyframe as the engine loads it                          |
| `definition` | What a keyframe names: `sun`, `thunderbolt` and `ambient`                                  |
| `level`      | Which cycles a level plays                                                                 |
| `mixer`      | A cycle mixed at a time of day                                                             |
| `playback`   | Weather played over time: the blended keyframe pair and overlaid effects                   |
| `catalog`    | Everything read at once; `catalog::read` fills it                                          |

## Not read

- Shadow of Chernobyl's weather layout (cycles listed in `system.ltx`'s `[weathers]` and `[weather_effects]`, with
  older key spellings). OpenXRay still loads it, but every shipped `system.ltx` leaves out the
  `weathers\environment.ltx` that would declare it.
- `openxray.ltx`. Its `sun_dir_azimuth` turns a keyframe's sun where `sun_azimuth` is not written, but it is zero in
  every shipped tree, as is this crate's default.

## Checks

Run `cargo test --locked -p xrf-environment` from the repository root. The optional `typescript-bindings` feature
derives `specta` types for the frontend's generated bindings.

Sources: the [reader](src/catalog/environment_reader.rs), [catalog](src/catalog/environment_catalog.rs), and
[mixer](src/mixer/weather_mixer.rs).
