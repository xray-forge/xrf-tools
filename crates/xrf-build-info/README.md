# xrf-build-info

Embeds build provenance in XRF binaries: package version, source commit, build kind, compiler settings, build time and
CI run.

- `xrf-cli` prints it for `--version` and in its reports.
- The desktop app logs it at startup, shows it in Settings and About, and colours its icon by build kind.

Everything is fixed at compile time. Reading the record runs no Git command and reads no runtime environment.

## Record a build

The crate has two halves: `BuildInfoEmitter::emit()` records the metadata from a build script, and the `build_info!()`
macro reads it back into a `BuildInfo`.

Add the crate as both a normal and a build dependency of the binary:

```toml
[dependencies]
xrf-build-info = { workspace = true }

[build-dependencies]
xrf-build-info = { workspace = true }
```

In `build.rs`:

```rust,no_run
use xrf_build_info::BuildInfoEmitter;

fn main() {
  BuildInfoEmitter::emit();
}
```

In the binary:

```rust
use xrf_build_info::{BuildInfo, build_info};

fn main() {
  let build: BuildInfo = build_info!();

  print!("{build}");

  if let Some(commit) = build.short_commit() {
    println!("Source revision: {commit}");
  }
}
```

Call the macro in the package whose build script ran the emitter. Called from a library, it reads that library's
version and compile-time environment instead.

## Read the record

`version`, `kind` and `is_dirty` are always present; every other field is an optional static string. The values
describe the build, not the machine running it.

| Field          | Source                                                                   |
| -------------- | ------------------------------------------------------------------------ |
| `version`      | The consuming package's `CARGO_PKG_VERSION`                              |
| `kind`         | `XRF_BUILD_KIND` (see below)                                             |
| `commit`       | `GITHUB_SHA`, else `git rev-parse HEAD`                                  |
| `reference`    | `GITHUB_REF_NAME`, else `git rev-parse --abbrev-ref HEAD`                |
| `is_dirty`     | Whether `git status --porcelain` prints anything                         |
| `built_at`     | `XRF_BUILD_TIMESTAMP`, else the emitter's current UTC time to the second |
| `target`       | Cargo's `TARGET` triple                                                  |
| `rustc`        | `$RUSTC --version`, or `rustc --version`                                 |
| `profile`      | Cargo's `PROFILE`                                                        |
| `optimization` | `opt-level=<OPT_LEVEL>, lto=<value>, codegen-units=<value>` (see below)  |
| `run_id`       | `GITHUB_RUN_ID`                                                          |

Output helpers:

- `Display` writes `<version> (<kind>)`, then one labelled line per present field. The commit line adds
  ` on <reference>` and `(dirty)` when known; without a commit, neither is printed. Every line ends with a newline, so
  use `print!`.
- `short_commit()` returns the first seven characters of the commit, or `None`. A `GITHUB_SHA` override is not checked
  to be a hash.
- `BuildInfo` and `BuildKind` serialize in camelCase (`builtAt`, `"development"`). The `typescript-bindings` feature
  adds Specta types for the frontend.

## Set build metadata

Set these in the environment that runs Cargo; [ENVIRONMENT.md](../../ENVIRONMENT.md) lists them.

- `XRF_BUILD_KIND`: exactly `development` or `optimized`, case-sensitive. Anything else, empty or unset, is
  `BuildKind::Local`. It is only a label and selects no Cargo profile. `cargo make` and CI set it, and
  `BuildKind::from_recorded` maps it the same way for other build scripts.
- `XRF_BUILD_TIMESTAMP`: copied as is; use RFC 3339.
- `GITHUB_SHA`, `GITHUB_REF_NAME`, `GITHUB_RUN_ID`: set by GitHub Actions.

An unset variable falls back as the table shows; an explicitly empty one leaves the field absent. A failed Git or
compiler query also leaves its field absent rather than failing the build.

The optimization summary is a hint, not a full account:

- `lto` and `codegen-units` come from `CARGO_PROFILE_RELEASE_LTO` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS`, or read
  `profile default` when unset. `Cargo.toml` is not read.
- They are reported whenever set, even in a non-release build.
- A missing `OPT_LEVEL` reads `unknown`.

## Git state and rebuilds

Git is optional. Without it, a build still records version, time and Cargo metadata. A detached checkout reports `HEAD`
as its reference; no history or revision count is needed.

The dirty flag always comes from the local checkout:

- It can be true in CI, even when the commit comes from GitHub variables.
- It counts untracked files, whether or not they reach the binary.
- It is false when Git cannot answer, so `false` alone does not prove a clean checkout.

The emitter reruns when one of its environment inputs changes or when Git's `HEAD` or `logs/HEAD` changes (a checkout
or a commit). It does not watch the working tree, so a source-only edit can keep the old dirty flag and time. For a
fresh record, change `XRF_BUILD_TIMESTAMP` or touch the package's `build.rs`.

Without the emitter, the macro still compiles and reads the version. Other fields come from any `XRF_BUILD_*` values
present at compilation, with `kind: Local` and `is_dirty: false`.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-build-info
cargo doc --locked -p xrf-build-info --no-deps
```

Sources: the [emitter](src/build_info_emitter.rs), the [record, display and macro](src/build_info.rs), and the
[kind mapping](src/build_kind.rs). In use: the [CLI build script](../../bin/xrf-cli/build.rs) and the
[desktop build-info command](../../bin/xrf-app/plugins/xrf-app-system/src/diagnostics/commands/get_build_info.rs).
