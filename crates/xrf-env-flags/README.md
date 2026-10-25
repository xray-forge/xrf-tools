# xrf-env-flags

One table, `EnvFlags::ALL`, of every environment variable the XRF workspace reads or honours: what each does, the
values it accepts, its default, which binaries or build steps read it, and on which builds or platforms. Workspace code
reads a runtime variable only through its constant here, never through `std::env`, so every variable parses the same
way. A test writes the table out as the repository's [ENVIRONMENT.md](../../ENVIRONMENT.md); never edit that file by
hand.

## Read a variable

Each variable is an associated constant of `EnvFlags`, typed by the value it takes. The type decides the read method,
so a variable cannot be read as another kind:

| Type         | Method        | Returns                                         |
| ------------ | ------------- | ----------------------------------------------- |
| `SwitchFlag` | `read_switch` | `Option<bool>`                                  |
| `ChoiceFlag` | `read_choice` | `Option<&'static str>`, in the table's spelling |
| `PathFlag`   | `read_path`   | `Option<PathBuf>`                               |
| `TextFlag`   | `read_text`   | `Option<String>`                                |

```rust,no_run
use xrf_env_flags::EnvFlags;

let backend: Option<&'static str> = EnvFlags::RENDER_BACKEND.read_choice();
let is_ray_query_off: bool = EnvFlags::RENDER_RAY_QUERY.read_switch() == Some(false);
let log_filter: Option<String> = EnvFlags::RUST_LOG.read_text();
```

`None` means unset, empty, or invalid, and the caller applies the default the table documents. Values are trimmed.
Switches take `1`, `true`, `on` or `yes` for on and `0`, `false`, `off` or `no` for off; choices take their names; both
in any case. A non-empty invalid value is logged as a `log` warning, so it shows only once the binary has set up its
logger. `read_path` returns a non-Unicode path exactly as the system gives it. Every typed flag dereferences to its
`EnvFlag`, whose `get_raw` returns the value as set, uninterpreted.

## Kinds of variable

`EnvFlagKind` says who sets a variable and when it is read. It picks the section of `ENVIRONMENT.md` a variable is
listed in and decides what a running binary reports.

| Kind          | Meaning                                                                     |
| ------------- | --------------------------------------------------------------------------- |
| `Runtime`     | Read by a running binary; set by whoever starts it.                         |
| `PassThrough` | Read by a third-party library (`RUST_LOG`, WebView2's); the app reports it. |
| `Build`       | Read during the build, by a build script, `cargo make` or CI.               |
| `Internal`    | Set by a build script for the compiler; listed so the name is not reused.   |

Build scripts read their build variables with `std::env` (xrf-build-info's emitter does); the table still documents
them.

## Add a variable

1. Declare a constant on `EnvFlags` with the constructor matching its `EnvFlagValue`: `SwitchFlag::new` takes
   `EnvFlagValue::Switch`, `ChoiceFlag::new` a `Choice`, and so on. The constructors are `const fn`s that assert the
   match, so a mismatch fails the build. Name the workspace's own `XRF_<AREA>_<NAME>`; the test
   `names_the_workspaces_own_by_area` accepts the areas `RENDER`, `APP`, `CLI`, `BUILD` and `DLSS`, so a new area goes
   there too. wgpu's instance switches keep wgpu's `WGPU_*` names.
2. List it in `EnvFlags::ALL`, in the order the document shows it. Write `readers` as binary names (`xrf-app`,
   `xrf-cli`): `list_read_by` matches them exactly.
3. Read it through the constant where it is needed.
4. Run `cargo test -p xrf-env-flags`. If `ENVIRONMENT.md` is out of date, the test rewrites it and fails once; commit
   the file.

Test-only and benchmark-only variables such as `XRF_BENCH_CORPUS` stay with their tests, out of the table.

## Report the variables

`EnvFlags::list_readings` returns an `EnvFlagReading` for every runtime and passed-through variable, set or not, in
`ALL`'s order: its `value` as set, the `applied` value in the table's spelling, and `is_set` and `is_refused` (set but
invalid). The desktop app logs the set ones at startup and lists them in Settings, About. `EnvFlags::list_read_by`
returns the runtime and passed-through variables one reader reads; `xrf-cli --help` ends with them.
`EnvFlags::to_markdown` renders `ENVIRONMENT.md`. The `typescript-bindings` feature derives Specta types for
`EnvFlagKind` and `EnvFlagReading`.
