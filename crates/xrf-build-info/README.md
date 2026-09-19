# xrf-build-info

Embeds build metadata in XRF binaries: package version, source commit, build classification, compiler settings,
timestamp, and workflow run. The CLI uses it for version output and reports; the desktop app uses it in About.

The crate has two parts: `emit()` records metadata from a consuming package's build script, and `build_info!()` reads
that package's compile-time environment into a `BuildInfo`. Reading or displaying the record at runtime performs no
Git commands or metadata discovery.

## Record a build

Add the crate to both dependency sections: the build script calls the emitter, and the binary uses the record and
macro. The XRF workspace already declares the path dependency centrally.

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

  print!("{build}");

  if let Some(commit) = build.short_commit() {
    println!("Source revision: {commit}");
  }
}
```

Call the macro in the consuming package. Calling it inside a shared library instead would read that library's
compile-time environment and package version.

## Read the record

`version`, `kind`, and `is_dirty` are always present. All other fields are optional static strings. Values describe
the build, not the machine currently running the binary.

| Field          | Source                                                                             |
| -------------- | ---------------------------------------------------------------------------------- |
| `version`      | The consuming package's `CARGO_PKG_VERSION`                                        |
| `kind`         | `XRF_BUILD_KIND`, interpreted as described below                                   |
| `commit`       | `GITHUB_SHA`, falling back to `git rev-parse HEAD` when unset                      |
| `reference`    | `GITHUB_REF_NAME`, falling back to `git rev-parse --abbrev-ref HEAD` when unset    |
| `is_dirty`     | Whether `git status --porcelain` produces nonempty output                          |
| `built_at`     | `XRF_BUILD_TIMESTAMP`, or the emitter's current UTC time rounded to a whole second |
| `target`       | Cargo's `TARGET` triple                                                            |
| `rustc`        | Output of `RUSTC --version`, using `rustc` when `RUSTC` is unset                   |
| `profile`      | Cargo's `PROFILE` value                                                            |
| `optimization` | A summary of `OPT_LEVEL` and the supported release-profile environment overrides   |
| `run_id`       | `GITHUB_RUN_ID`                                                                    |

`Display` starts with `<version> (<kind>)`, then writes available fields on separate labelled lines. The commit line
includes the full commit, the reference when present, and `(dirty)` when applicable. A reference or dirty flag without
a commit is not printed separately. Each emitted line ends with a newline, so `print!("{build}")` avoids an extra
blank line.

`short_commit()` returns the first seven bytes of the commit, or the whole value if shorter; it returns `None` when
the commit is absent. It assumes an ASCII Git hash. Commit overrides are not validated, so arbitrary non-ASCII text
can make this byte slice panic.

`BuildInfo` and `BuildKind` implement Serde serialization and deserialization with camelCase names. For example,
`built_at` serializes as `builtAt`, and `BuildKind::Development` as `"development"`. The optional `typescript-bindings`
feature adds Specta type metadata for frontend binding generation.

## Set build metadata

Set overrides in the environment used to invoke Cargo. `XRF_BUILD_KIND` recognizes exactly `development` and
`optimized`; unset, empty, or unrecognized values produce `BuildKind::Local`. Matching is case-sensitive. The kind is
a label: it does not select a Cargo profile, enable optimization, or detect whether a build runs in CI.

`XRF_BUILD_TIMESTAMP` is copied as supplied. Use an RFC 3339 timestamp for consistent consumers; the emitter does not
parse or validate an override. Without one, it records the time the build script runs.

The optimization summary has this form:

```text
opt-level=<OPT_LEVEL>, lto=<value>, codegen-units=<value>
```

LTO and codegen units come from `CARGO_PROFILE_RELEASE_LTO` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS`. Without those
variables, each is reported as `profile default`; the emitter does not read their resolved values from `Cargo.toml`.
Missing `OPT_LEVEL` is reported as `unknown`. The release override variables are reported whenever set, even for a
non-release build, so the summary is not a complete account of the compiler's effective settings.

Unset `GITHUB_SHA`, `GITHUB_REF_NAME`, and `XRF_BUILD_TIMESTAMP` allow their fallbacks. Explicitly empty values
suppress the corresponding emitted field instead of selecting a fallback. Git failures and unsuccessful
compiler-version queries also leave those optional fields absent rather than failing the build.

## Git state and rebuilds

Git is optional: builds from a source archive or a machine without Git can still record version, timestamp, and Cargo
metadata. A detached checkout can report `HEAD` as its reference. No revision count is recorded, and complete Git
history is not required.

Dirty state comes from the checkout even when the commit and reference come from GitHub variables. It can be true in
CI and includes untracked files reported by Git, whether or not they become part of the binary. If Git cannot answer,
`is_dirty` is false; that value alone does not prove a clean checkout.

The emitter asks Cargo to rerun when its override inputs change, or when the discovered Git directory's `HEAD` or
`logs/HEAD` changes. It does not watch the Git index or every working-tree file. A source-only edit can therefore
leave the previous dirty flag or timestamp embedded if Cargo reuses the build-script output. Force that script to
rerun when a fresh snapshot is required, for example by updating the timestamp override or touching the consuming
package's `build.rs`.

Without `emit()`, the macro still compiles under Cargo and reads the package version. Other fields depend on any
`XRF_BUILD_*` values already present at compilation; otherwise they are absent, with `kind: Local` and
`is_dirty: false`. Changing runtime environment variables never updates an already compiled record.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-build-info
cargo doc --locked -p xrf-build-info --no-deps
```

See the [emitter and input tracking](src/emit.rs), [record, display, and macro](src/build_info.rs), and
[build-kind mapping](src/build_kind.rs). The [CLI build script](../../bin/xrf-cli/build.rs) and
[desktop metadata command](../../bin/xrf-app/src/plugins/system/diagnostics/commands/get_build_info.rs) show the two
halves in use.
