# xrf-dltx

Resolves Monolith and Anomaly DLTX patches as an `LtxDialect`, the dialect interface of [xrf-ltx](../xrf-ltx/README.md).

DLTX lets a mod change a config without editing it: a `mod_system_*.ltx` file beside `system.ltx` overrides, deletes,
or extends sections of the tree `system.ltx` resolves. Only the Monolith engine and Anomaly read it; vanilla Call of
Pripyat and OpenXRay do not. The crate exports two names: `DltxDialect` and `select_ltx_dialect`.

## Resolve a patched config

```rust,no_run
use xrf_dltx::select_ltx_dialect;
use xrf_ltx::{LtxProject, LtxProjectOptions};

fn main() -> xrf_error::XrfResult {
  let options = LtxProjectOptions::default().with_dialect(select_ltx_dialect(true));
  let project = LtxProject::open_at_path_opt("gamedata/configs", options)?;
  let entry = project.config_path("system.ltx")?;
  let resolved = project.read_resolution(&entry)?;

  println!("{} resolved sections", resolved.ltx.len());

  for diagnostic in &resolved.diagnostics {
    println!("[{}] {}", diagnostic.section, diagnostic.message);
  }

  Ok(())
}
```

Resolve the base entry point, such as `system.ltx`, rather than a patch file: under DLTX a project lists its patch files
as attachments (`LtxProject::ltx_attachments`), not entry points. Standard LTX treats a patch file as an ordinary config
and refuses to resolve it, with an error that names the DLTX option.

## Select the dialect

Call `select_ltx_dialect(true)` to obtain an `Arc<dyn LtxDialect>` for DLTX, or pass `false` for standard LTX, and set
it with `LtxProjectOptions::with_dialect` before opening a project. Select it once, where a binary reads its `--dltx`
flag: a directory layout alone does not say whether patches should apply, and downstream crates use resolved values
without depending on this crate. `DltxDialect` is also available directly; it names itself `dltx`.

DLTX is not standard LTX plus a patch step. It resolves base data differently even when no patch file exists, so never
resolve as standard LTX first and apply DLTX to the result. Differences that change values:

| Behaviour               | Standard LTX               | DLTX                                                         |
| ----------------------- | -------------------------- | ------------------------------------------------------------ |
| Include priority        | Read order                 | By depth: the root file beats a file it includes             |
| Inheritance             | Parent must precede        | Lazy: forward references resolve after the full read         |
| Missing parent          | Error                      | An empty section, with a diagnostic                          |
| Duplicate key in a file | Last line wins             | Lowest depth wins, then the later line                       |
| Output order            | As authored                | Sections sorted by name, fields by key                       |
| Provenance on request   | Declared or inherited      | Winning file, load depth, and operation for every field      |

Section names are case-folded but parent names are not, so `[a]:Base` never finds `[base]`; DLTX reports that as a
missing parent that contributes no fields.

## Patch files

A patch for a root is `mod_<stem>_*.ltx` in the root's own directory, matched without ASCII case; only the root gets
patch files, not the configs it includes. When a longer base exists, it claims the names that read as its own: with
`system.ltx` and `system_foo.ltx` both present, `mod_system_foo_bar.ltx` patches only `system_foo.ltx`. Patches apply in
ascending name order and the alphabetically last file wins a conflict.

Given this section in the base tree:

```ini
[wpn_base]
cost = 100
ammo_class = ammo_a, ammo_b

[wpn_ak74]:wpn_base
cost = 4000
```

a `mod_system_patch.ltx` beside `system.ltx` can override and extend it:

```ini
![wpn_ak74]
cost = 9999
>ammo_class = ammo_c
```

The resolved `wpn_ak74` has `cost = 9999` and `ammo_class = ammo_a,ammo_b,ammo_c`; `wpn_base` is unchanged. Lists are
split on commas and trimmed, so an edited list is written back without spaces.

| Statement      | Effect                                                                     |
| -------------- | -------------------------------------------------------------------------- |
| `![section]`   | Override; dropped with a diagnostic when nothing declares the section      |
| `@[section]`   | Override, creating an empty base section when nothing declares one        |
| `!![section]`  | Delete, after everything else resolves, so children keep what they took    |
| `!key`         | Delete the key; a value written after it is discarded                      |
| `>key = a, b`  | Append to a comma list                                                     |
| `<key = a, b`  | Remove from a comma list                                                   |
| `[section]:!p` | Remove parent `p`                                                          |

## Provenance and failures

Plain project reads return the resolved config without field provenance. `LtxProject::resolve_explained`, or
`LtxResolveRequest::with_provenance` on the dialect directly, records for every field an `LtxFieldOrigin::Loaded` with
the winning file, its load depth, and the operation prefix.

Diagnostics report what the engine does silently (an orphan override, a missing parent, a field before any section) and
sit beside a successful result. Conditions the engine refuses to start on, such as a duplicate plain section or an
inheritance cycle, return `Err`.

The dialect reads documents and directory listings through `xrf_ltx::LtxDocumentSource` and performs no file I/O itself,
so the same rules run over loose files, archives, or test documents held in a map.

## Checks

Run `cargo test --locked -p xrf-dltx` from the repository root.

See the [dialect](src/dltx_dialect.rs), [patch discovery](src/discovery/dltx_discovery.rs), and
[compatibility tests](src/tests).
