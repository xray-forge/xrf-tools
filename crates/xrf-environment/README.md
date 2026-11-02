# xrf-environment

Reads a game's environment configs, everything under `configs\environment` its engine reads, the way that engine
reads them: weather cycles and effects, suns and their flares, thunderbolts, ambients with their sound channels and
effects, Anomaly's sun table, and which cycles a level plays. It also mixes a cycle at a time of day as the engine
does, which is what the level viewer's weather plays.

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
`EnvironmentCatalog`, resolving each config under the project's dialect. A config that is missing or will not resolve
is a finding, not an error; `read` fails only when the project cannot be listed or a path cannot be built.
`read_opt` takes `EnvironmentReadOptions`: the engine, a `JobHandle` for cancellation, and `is_explained`.
`read_cycle` reads one cycle or effect with the findings of its own config, and `read_level_weather` answers the
cycles one level can be lit under: its `game.ltx` section's `weathers`, or `[default]` where it writes none, resolved
against the catalog's weather graphs.

Cycle discovery drops DLTX attachments (`mod_` files), which patch a base cycle rather than being cycles of their own.
A file under `environment\ambients` whose sections `ambients.ltx` already reaches through its `#include`s is a shared
ambient, not a level's.

## One catalog, three readers

A viewer plays a cycle from the catalog, an editor shows and changes it, and a verifier reports its findings. Each
section is kept as authored (`EnvironmentSection`): the keys the engine reads, parsed as it parses them, the rest as
written, and where each came from when asked (`is_explained`, which resolves every config afresh rather than through
the project's cache, so a caller that only checks leaves it off). `WeatherDescriptor` is a keyframe as the engine holds
it after loading, in radians and with every default applied.

## Engine-grounded

Each section kind declares its keys once (`declare_environment_keys!`): how the engine parses each
(`EnvironmentValueKind`), and per engine whether it reads it and what it holds without it (`EnvironmentKeyUse`:
required, optional, conditional, or unread). The reader, the findings and an editor all read that one table.

Values are stored as the engine holds them. `CInifile` never refuses a value it can read at all, so a malformed number
is kept as the number `atof` takes from it, a vector written with missing components holds zeros for them, and the
finding says what was written.

- `EnvironmentRule::Engine`: the engine refuses to load it, or loads it as something other than what is written.
- `EnvironmentRule::Reference`: a section names another the engine cannot find.
- `EnvironmentRule::Convention`: the engine loads it, against a convention strong enough to be a mistake: its own
  `! Invalid` colour warnings, a value it clamps, a flare list written unevenly.

The engine loads suns, thunderbolt collections, thunderbolts, ambients, sound channels and ambient effects only when a
keyframe names them, directly or through another definition. A finding in a definition nothing names is demoted to
`Convention`, with "nothing references it" added to its message.

A list is split as `_GetItemCount` counts it: `a,` is one item, and the count stops at a doubled comma, so `a,,b` is
`a` and an empty item. An ambient's channels are `AmbientKey::list_channels`: the list under the spelling its engine
reads, or for a Shadow of Chernobyl ambient, which OpenXRay plays as its own channel, the `sound_channels.ltx` section
of the ambient's own name. The strike settings' `altitude` is a range on OpenXRay and one number on Monolith
(`ThunderboltSettingsKey::get_altitude`), and both clamp `min_dist_factor` to `MAX_DIST_FACTOR`.

Texture, sound and model existence belong to whoever holds the asset index, `xrf-gamedata`, whose `environment` check
reports these findings by rule.

## Two engines

`XrayEngine::Vanilla` is OpenXRay and stock Call of Pripyat; `XrayEngine::Extended` is Anomaly's Monolith (see
[xrf-engine-target](../xrf-engine-target/README.md)). They disagree on which keys a keyframe needs, where the sun comes
from (keyframe angles, or `sun_positions.ltx`), which extra keys mean anything, whether `system.ltx` stands in for a
missing definition, and how a level's `weathers` key resolves. The engine is named by the caller, never inferred.

## Mixing and playback

`WeatherMixer` mixes a cycle's `WeatherDescriptor` keyframes at a `WeatherMixPoint` (a time of day and a view position)
as `CEnvironment::lerp` does, into a `WeatherMix`. `mix` picks the keyframes around the time with `select` (the
engine's `SelectEnvs` on a forced start) and mixes them; `mix_pair` mixes a pair the caller holds from frame to frame,
as the engine does, and `select_by` selects over anything standing at a time of day. The sun comes from a
`WeatherSunSource`: lerped between the keyframes' own directions, OpenXRay's astronomical sun, or Monolith's hourly
table. `WeatherModifier`s (a level's `level.env_mod` volumes) are weighed as `CEnvModifier::sum` does; they are the
mixer's own input, so the crate does not depend on the level format.

`playback` steps the rest of what the engine plays: the keyframe pair being blended (`WeatherPair`,
`WeatherPlayedKeyframe`) and a weather effect laid over the cycle (`WeatherEffectTimeline`, `WeatherEffectStart`).

## Module map

- `section`: one section as authored, and the reader that parses one as the engine does. `section::key` is the
  vocabulary every section kind is read with: a key's value kind, and per engine whether it is read and what it
  defaults to; `section::finding` is what a read found wrong, and of which kind.
- `weather`: cycles, effects, their keyframes, and a keyframe as the engine loads it.
- `definition`: the definitions a keyframe names, `sun`, `thunderbolt` and `ambient`.
- `level`: which cycles a level plays.
- `mixer`: a cycle mixed at a time of day.
- `playback`: a weather played on: the pair of keyframes blended, and effects laid over the cycle.
- `catalog`: reading all of it at once; `catalog::read` is the one read that fills it.

## What is not read

Shadow of Chernobyl's weather layout, cycles listed in `system.ltx`'s `[weathers]` and `[weather_effects]` with
keyframes under the older key spellings, is not read: OpenXRay still loads it, but every shipped tree's `system.ltx`
leaves the `weathers\environment.ltx` that would declare it out. Neither is `openxray.ltx`, whose `sun_dir_azimuth`
OpenXRay turns a keyframe's sun by where `sun_azimuth` is not written: it holds zero in every shipped one, as the
crate's default does.

## Checks

Run `cargo test --locked -p xrf-environment` from the repository root. The optional `typescript-bindings` feature
derives `specta` types for the frontend's generated bindings.

See the [reader](src/catalog/environment_reader.rs), [catalog](src/catalog/environment_catalog.rs), and
[mixer](src/mixer/weather_mixer.rs).
