# xrf-engine-target

Names which engine a game data tree is meant for. Two engines read the same configs differently, and a tree does not
say which it targets, so a caller states it instead of this workspace guessing.

```rust
use xrf_engine_target::XrayEngine;

// OpenXRay and the stock Call of Pripyat engine.
assert_eq!("vanilla".parse::<XrayEngine>(), Ok(XrayEngine::Vanilla));

// Anomaly's Monolith engine and the extended builds that share its readers.
assert_eq!(XrayEngine::Extended.as_str(), "extended");
```

Named, never inferred: a patched Anomaly tree and a vanilla one look alike from the outside. The config dialect
(`--dltx`) is a separate choice; an extended tree usually resolves with DLTX, and the two stay two values.
