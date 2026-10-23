# xrf-build-info

Embeds build metadata in XRF binaries, so a built binary can state its own provenance: package version, source commit,
build kind, compiler settings, build time, and workflow run. `xrf-cli` prints it for `--version` and in its reports;
the desktop app logs it at startup, shows it in Settings, About, and colours its executable icon by the build kind.

The crate has two halves. `emit()` runs in a consuming package's build script and records the metadata as compile-time
environment variables; the `build_info!()` macro reads them back into a `BuildInfo` in that package. Everything is
fixed at compile time: reading or displaying the record runs no Git command and reads no runtime environment.

## Record a build

Add the crate as both a normal and a build dependency of the binary:

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

  print!("{build}");

  if let Some(commit) = build.short_commit() {
    println!("Source revision: {commit}");
  }
}
```

Call the macro in the package whose build script ran `emit()`. Called from a shared library, it would read that
library's compile-time environment and version instead.

## Read the record

`version`, `kind` and `is_dirty` are always present; every other field is an optional static string. Values describe the
build, not the machine running the binary.

| Field          | Source                                                                             |
| -------------- | ---------------------------------------------------------------------------------- |
| `version`      | The consuming package's `CARGO_PKG_VERSION`                                        |
| `kind`         | `XRF_BUILD_KIND`, as described below                                               |
| `commit`       | `GITHUB_SHA`, falling back to `git rev-parse HEAD` when unset                      |
| `reference`    | `GITHUB_REF_NAME`, falling back to `git rev-parse --abbrev-ref HEAD` when unset    |
| `is_dirty`     | Whether `git status --porcelain` prints anything                                   |
| `built_at`     | `XRF_BUILD_TIMESTAMP`, or the emitter's current UTC time rounded to a whole second |
| `target`       | Cargo's `TARGET` triple                                                            |
| `rustc`        | Output of `$RUSTC --version`, using `rustc` when `RUSTC` is unset                  |
| `profile`      | Cargo's `PROFILE`                                                                  |
| `optimization` | `opt-level=<OPT_LEVEL>, lto=<value>, codegen-units=<value>`, described below       |
| `run_id`       | `GITHUB_RUN_ID`                                                                    |

`Display` writes `<version> (<kind>)`, then one labelled line per present field. The commit line carries the full
commit, ` on <reference>` when known, and `(dirty)`; without a commit, the reference and dirty flag are not printed.
Every line ends with a newline, so use `print!`, not `println!`.

`short_commit()` returns the first seven bytes of the commit, the whole value if shorter, or `None` without one. It
assumes an ASCII hash: a `GITHUB_SHA` override is not validated, and non-ASCII text can make the slice panic.

`BuildInfo` and `BuildKind` serialize with Serde in camelCase (`builtAt`, `"development"`). The `typescript-bindings`
feature derives Specta types for the frontend bindings.

## Set build metadata

Set overrides in the environment that runs Cargo; [ENVIRONMENT.md](../../ENVIRONMENT.md) lists them.

- `XRF_BUILD_KIND` takes exactly `development` or `optimized`, case-sensitive; unset, empty or anything else is
  `BuildKind::Local`. It is a label only: it selects no Cargo profile and detects nothing. `cargo make` and CI set it;
  `BuildKind::from_recorded` maps it the same way for other build scripts.
- `XRF_BUILD_TIMESTAMP` is copied unparsed; use RFC 3339. Unset, the emitter records when the build script runs.
- `GITHUB_SHA`, `GITHUB_REF_NAME` and `GITHUB_RUN_ID` are set by GitHub Actions.

Unset `GITHUB_SHA`, `GITHUB_REF_NAME` and `XRF_BUILD_TIMESTAMP` fall back as the table shows; an explicitly empty value
leaves the field absent instead. A failed Git or compiler query also leaves its field absent rather than failing the
build.

In the optimization summary, `lto` and `codegen-units` are `CARGO_PROFILE_RELEASE_LTO` and
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS`, or `profile default` when unset; the emitter does not read `Cargo.toml`. A
missing `OPT_LEVEL` is `unknown`. The release variables are reported whenever set, even in a non-release build, so the
summary is not a full account of the compiler's effective settings.

## Git state and rebuilds

Git is optional: a build from a source archive or without Git still records version, time and Cargo metadata. A
detached checkout reports `HEAD` as its reference. Neither full history nor a revision count is needed.

Dirty state always comes from the local checkout, even when the commit and reference come from GitHub variables, so it
can be true in CI, and it counts untracked files whether or not they reach the binary. If Git cannot answer, `is_dirty`
is false, which alone does not prove a clean checkout.

The emitter tells Cargo to rerun it when one of its environment inputs changes, or when the Git directory's `HEAD` or
`logs/HEAD` changes (a checkout or a commit). It does not watch the index or working tree, so a source-only edit can
keep the previous dirty flag and time. When a fresh snapshot matters, change `XRF_BUILD_TIMESTAMP` or touch the
consuming package's `build.rs`.

Without `emit()`, the macro still compiles under Cargo and reads the version; other fields come from any `XRF_BUILD_*`
values present at compilation, otherwise absent, with `kind: Local` and `is_dirty: false`.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-build-info
cargo doc --locked -p xrf-build-info --no-deps
```

Source: the [emitter](src/emit.rs), the [record, display and macro](src/build_info.rs), and the
[kind mapping](src/build_kind.rs). The [CLI build script](../../bin/xrf-cli/build.rs) and the
[desktop build-info command](../../bin/xrf-app/src/plugins/system/diagnostics/commands/get_build_info.rs) show the two
halves in use.
