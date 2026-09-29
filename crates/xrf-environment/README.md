# xrf-environment

Reads a game's environment configs, everything under `configs\environment` its engine reads, the way that engine
reads them: weather cycles and effects, suns and their flares, thunderbolts, ambients with their sound channels and
effects, Anomaly's sun table, and which cycles a level plays.

```rust,no_run
use xrf_engine_target::XrayEngine;
use xrf_environment::{EnvironmentCatalog, EnvironmentReader, WeatherDescriptor};
use xrf_ltx::LtxProject;

let project = LtxProject::open_at_path("gamedata/configs").unwrap();
let catalog: EnvironmentCatalog = EnvironmentReader::read(&project, XrayEngine::Vanilla).unwrap();

// Every problem found, placed at its file, section and key.
for finding in &catalog.findings {
  println!("{}: {}", finding.file, finding.message);
}

// A keyframe as authored, and as the engine holds it once loaded.
let cycle = catalog.find_cycle("default_clear").unwrap();
let noon = WeatherDescriptor::new(&cycle.keyframes[12], catalog.engine);
```

## One catalog, three readers

A viewer plays a cycle from the catalog, an editor shows and changes it, and a verifier reports its findings. Each
section is kept as authored (`EnvironmentSection`): the keys the engine reads, parsed as it parses them, the rest as
written, and where each came from when asked (`EnvironmentReadOptions::is_explained`). `WeatherDescriptor` is a
keyframe as the engine holds it after loading, in radians and with every default applied.

## Engine-grounded

Each section kind declares its keys once (`declare_environment_keys!`): how the engine parses each, and per engine
whether it reads it and what it holds without it. `CInifile` never refuses a value it can read at all, so a malformed
number is kept as the number `atof` takes from it, and the finding says what was written.

- `EnvironmentRule::Engine` — the engine refuses to load it, or loads it as something other than what is written.
- `EnvironmentRule::Reference` — a section names another the engine cannot find.
- `EnvironmentRule::Convention` — the engine loads it, against a convention strong enough to be a mistake: its own
  `! Invalid` colour warnings, a value it clamps, a flare list written unevenly.

Texture, sound and model existence belong to whoever holds the asset index, `xrf-gamedata`.

## Two engines

`XrayEngine::Vanilla` is OpenXRay and stock Call of Pripyat; `XrayEngine::Extended` is Anomaly's Monolith. They
disagree on which keys a keyframe needs, where the sun comes from (keyframe angles, or `sun_positions.ltx`), which
extra keys mean anything, whether `system.ltx` stands in for a missing definition, and how a level's `weathers` key
resolves. The engine is named by the caller, never inferred.
