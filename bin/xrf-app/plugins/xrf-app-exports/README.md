# xrf-app-exports

The externs a scripts project declares, in the XRF desktop application: a project opened as a session, each extern's
source read back, and a manifest of them written. Reading the externs is `xrf-export`; this crate holds the project
open and answers the page.

## Layout

- `project/`: `ExportsProjects`, the exports projects open, each held by the tool that opened it.
- `commands/`: opening, restoring and closing a project; reading the source an extern is declared in; and writing the
  manifest of every extern a project holds, in the format the output file's extension names.
- `plugin` and `ipc`: `ExportsPlugin` and its command domain.

## In the application

`ExportsPlugin` manages `ExportsProjects` and answers the `exports` domain. It runs no jobs.

## Checks

Run `cargo test -p xrf-app-exports` from the repository root.
