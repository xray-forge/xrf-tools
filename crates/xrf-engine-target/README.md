# xrf-engine-target

Names which engine a game data tree is meant for. Two engines read the same configs differently, and a tree does not
declare which it targets, so a caller either names one or leaves it to detection.

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

`XrayEngineChoice` is `auto` (the default), `vanilla` or `extended`. `detect_engine(roots, read_config)` infers the
engine for `auto` and answers an `XrayEngineResolution`: the engine, the `XrayEngineEvidence` that decided it, and the
file or logical path that showed it. `read_config` reads a logical path through whatever the caller already mounted,
so archived configs count, and it is called only when the installation layout decides nothing.

The installation is each root, and the asset roots are centred on, when it holds `fsgame.ltx`, otherwise its nearest
ancestor that does. The first rule that matches wins:

1. `AnomalyExecutables`: `AnomalyLauncher.exe` in the installation, or `bin/AnomalyDX*.exe` or `bin/VerifiedDX11.exe`.
   Names compare without case. Extended.
2. `AnomalyFsgame`: `fsgame.ltx` declares `$warfare_presets$`. Extended.
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
dialect (`--dltx`) is a separate choice; an extended tree usually resolves with DLTX, and the two stay two values.
