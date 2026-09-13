# xrf-build-info

Records build provenance for XRF binaries: commit, build kind, compiler settings, timestamp, and CI information.

## Record a build

Add this crate to both the binary's dependencies and build dependencies. Call `xrf_build_info::emit()` from its
`build.rs`, then call `xrf_build_info::build_info!()` in the binary to obtain a `BuildInfo`.

In a workspace binary's `Cargo.toml`:

```toml
[dependencies]
xrf-build-info = { workspace = true }

[build-dependencies]
xrf-build-info = { workspace = true }
```

In `build.rs`:

```rust,no_run
fn main() {
  xrf_build_info::emit();
}
```

In the binary:

```rust
use xrf_build_info::{BuildInfo, build_info};

fn main() {
  let build: BuildInfo = build_info!();

  println!("{build}");

  if let Some(commit) = build.short_commit() {
    println!("Source revision: {commit}");
  }
}
```

The display starts with the binary's package version and build kind, followed by the metadata available for that build.

## Metadata and overrides

The macro must run in the consuming crate: it reads that crate's compile-time environment. The version comes from
`CARGO_PKG_VERSION`. Git and CI metadata are optional; missing values stay absent.

| Input                                             | Recorded value              |
| ------------------------------------------------- | --------------------------- |
| `GITHUB_SHA`, otherwise Git HEAD                  | Commit                      |
| `GITHUB_REF_NAME`, otherwise Git branch           | Reference                   |
| `GITHUB_RUN_ID`                                   | Workflow run                |
| `XRF_BUILD_TIMESTAMP`, otherwise the current time | Build timestamp             |
| `XRF_BUILD_KIND`                                  | Build classification        |
| Cargo target, profile, and compiler environment   | Compiler and target details |

The revision count comes from complete Git history and is omitted for shallow clones. Optimization settings that Cargo
does not expose are reported as profile defaults unless the supported release-profile overrides are present.

## Limitations and checks

`XRF_BUILD_KIND` recognizes `development` and `optimized`; absent or unrecognized values become `local`. This labels
the build and does not choose Cargo's optimization profile.

Values describe compilation time, not the machine running the binary. The emitter tolerates unavailable Git metadata; it
does not require a checkout. Without an emitter call, the macro still reads the package version but has less metadata.

The `typescript-bindings` feature adds frontend type metadata. Run `cargo test --locked -p xrf-build-info` from the
repository root.

See the [emitter](src/emit.rs) for environment overrides and the [record and macro](src/build_info.rs) for fields.
