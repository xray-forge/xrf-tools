# xrf-dltx

Resolves Monolith-style DLTX patches through the dialect interface in [xrf-ltx](../xrf-ltx/README.md).

## Resolve a patched config

```rust,no_run
use xrf_dltx::select_ltx_dialect;
use xrf_ltx::{LtxProject, LtxProjectOptions};

fn main() -> xrf_error::XrfResult {
  let options = LtxProjectOptions::default().with_dialect(select_ltx_dialect(true));
  let project = LtxProject::open_at_path_opt("gamedata/configs", options)?;
  let entry = project.config_path("system.ltx")?;
  let resolved = project.read_full(&entry)?;

  println!("{} resolved sections", resolved.len());

  Ok(())
}
```

Resolve the base entry point, such as `system.ltx`, rather than opening its patch as a standalone config.

## Base files and attachments

A patch beside `system.ltx` can be named `mod_system_a.ltx`. Given this section in the base tree:

```ini
[wpn_ak74]
cost = 10000
```

The attachment can override it:

```ini
![wpn_ak74]
cost = 9000
```

The resolved section then has `cost = 9000`. This is one override case, not the complete patch language; the
compatibility tests also cover creation, deletion, list operations, inheritance, and load order.

## Select the dialect

Call `select_ltx_dialect(true)` to obtain an `Arc<dyn LtxDialect>` for DLTX, or pass `false` for standard LTX.
Set it on `LtxProjectOptions::dialect` before opening a project. `DltxDialect` is also available directly.

The dialect asks an `LtxDocumentSource` for documents and directory listings; it performs no file I/O itself.
Results use the shared `LtxResolution` model. Request provenance with `LtxResolveRequest::with_provenance`, or use
`LtxProject::resolve_explained` when working through a project.

Standard LTX and DLTX resolve from the original documents. Do not resolve as standard LTX first and then apply DLTX
to that result: the dialects differ in how loading and inheritance affect the winning fields.

## Provenance and failures

Normal project reads return the resolved config without field provenance. `resolve_explained` also records origins and
load decisions in a `LtxResolution`; use it for inspection where that additional work is needed.

Diagnostics are warnings; fatal load or resolution failures return `Err`. Select the dialect explicitly: a directory
layout alone does not identify whether patches should apply.

Refused patch operations return resolution errors. A returned diagnostic is additional information beside a result,
not a replacement for handling `Err`.

## Checks

Run `cargo test --locked -p xrf-dltx` from the repository root. Storage enters through `LtxDocumentSource`, so the same
rules can run over filesystem, archive, or test documents.

See the [dialect](src/dltx_dialect.rs), [selector](src/select_ltx_dialect.rs), and [compatibility tests](src/tests).
