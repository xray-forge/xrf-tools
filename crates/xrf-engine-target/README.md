# xrf-engine-target

Names which engine a game data tree is meant for. Two engines read the same configs differently, and a tree does not
declare which it targets, so a caller either names one or leaves it to detection. Readers whose behaviour differs by
engine take an `XrayEngine`: [xrf-environment](../xrf-environment/README.md), particle simulation, and the
`xrf-gamedata` checks built on them. The CLI spells the choice `--engine auto|vanilla|extended`.

```rust
use xrf_engine_target::{XrayEngine, XrayEngineChoice, XrayEngineEvidence, XrayEngineResolution};

// OpenXRay and the stock Call of Pripyat engine.
assert_eq!("vanilla".parse::<XrayEngine>(), Ok(XrayEngine::Vanilla));

// Anomaly's Monolith engine and the extended builds that share its readers.
assert_eq!(XrayEngine::Extended.as_str(), "extended");

// A named choice is taken as it is; `auto` runs detection.
let resolution: XrayEngineResolution = XrayEngineChoice::Extended.resolve(XrayEngineResolution::undetected);

assert_eq!(resolution.engine, XrayEngine::Extended);
assert_eq!(resolution.evidence, XrayEngineEvidence::Named);
```

## Named or inferred

`XrayEngineChoice` is `auto` (the default), `vanilla` or `extended`; `get_named` answers the engine a choice names, if
any. `XrayEngineDetector::detect(roots, read_config)` infers the engine for `auto` and answers an
`XrayEngineResolution`: the engine, the `XrayEngineEvidence` that decided it, and the file or logical path that showed
it (`describe` renders all of it as one line, such as `extended (Anomaly executables found)`). `read_config` reads a
logical path through whatever the caller already mounted, answering `None` where nothing resolves, so archived configs
count; it is called only when the installation layout decides nothing.

```rust,no_run
use xrf_engine_target::{XrayEngineChoice, XrayEngineDetector, XrayEngineResolution};
use xrf_vfs::{XrayMountMode, XrayRoots, XraySearch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let roots = XrayRoots::one("C:/Games/Anomaly".into(), XrayMountMode::Auto);
  let search: XraySearch = roots.to_search_plan()?.open()?;
  let resolution: XrayEngineResolution =
    XrayEngineChoice::Auto.resolve(|| XrayEngineDetector::detect(&roots, |path| search.read(path).ok()));

  println!("{}", resolution.describe());

  Ok(())
}
```

A root's installation is the root itself where it holds an fsgame file (`fsgame.ltx`, or an Enhanced Edition's
`fsgame_<game>.ltx`), otherwise its nearest ancestor that does. The first rule that matches wins:

1. `AnomalyExecutables`: `AnomalyLauncher.exe` in the installation, or `bin/AnomalyDX*.exe` or `bin/VerifiedDX11.exe`.
   Names compare without case. Extended.
2. `AnomalyFsgame`: the installation's fsgame file declares `$warfare_presets$`. Extended.
3. `AtmosfearCycles`: `configs\environment\dynamic_weather_graphs.ltx` has a `[weather_cycles]` section, which the
   weather graphs play on Extended. Extended.
4. `NoSigns`: Vanilla.

These rules are deliberately not signs of Monolith, because another reference tree ships them:

- `$arch_dir_addons$`, which IX-Ray declares;
- `$arch_dir_resource$`, which Call of Chernobyl declares;
- `weathers = atmosfear` in `game_maps_single.ltx`, which Call of Chernobyl writes while its engine reads weather as
  Vanilla does;
- `environment\sun_positions.ltx`, which Gunslinger for OpenXRay ships;
- DLTX `mod_*.ltx` files, which mean mods rather than an engine.

A patched tree can still look like the other engine's, so a caller that knows better names the engine. The config
dialect (`--dltx`, [xrf-dltx](../xrf-dltx/README.md)) is a separate choice; an extended tree usually resolves with
DLTX, and the two stay two values.

## Checks

Run `cargo test --locked -p xrf-engine-target` from the repository root. The detection rules are pinned by the tests in
[the detector](src/xray_engine_detector.rs), one fixture tree per rule and per deliberate non-sign. The optional
`typescript-bindings` feature derives `specta` types for the frontend's generated bindings.
