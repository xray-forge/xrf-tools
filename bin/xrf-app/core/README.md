# xrf-app-core

The infrastructure every plugin of the XRF desktop application builds on, belonging to no domain: the mounted asset
roots and their changes, documents opened as sessions in owner scopes, the worker pool and its jobs, the loopback
transport, where the application writes, its log, and the windows plugins draw into. Every plugin crate depends on it;
it depends on none of them.

## Layout

- `assets`: the mounted asset roots (`AssetMountState`) every domain resolves and reads through, and the changes a
  commit fans out to its subscribers.
- `session`: documents opened as sessions, grouped into `opening` (a slot publishing snapshots), `handles` (a domain's
  openings, `Sessions`), `scopes` (the owner scopes that release them, `SessionScopes`), `parts` (values read apart from
  a session's own) and `changes` (what a commit moves them by).
- `execution`: `ExecutionState`, how much of the machine the application may use and the one pool it uses it through.
- `jobs`: `JobRegistry`, identity, exclusion and cancellation for work that outlives one command response.
- `transport`: the loopback HTTP server that bulk bytes cross, so a large answer never blocks the window's thread.
- `directories`: `AppDirectories`, the application's own data, preferences and caches, and where tools write by
  default.
- `logging`: the log, written to the console and to the run's log file, panics included.
- `window`: the window a command was invoked from, native window handles, and the windows a close was approved for.
- `ipc`: the macro every plugin crate expands its command domains through, and core's own domains.
- `plugins`: core's own plugins, `AssetsPlugin`, `JobsPlugin`, `SessionsPlugin` and `TransportPlugin`.

A plugin crate declares its command domains once, in its `ipc/domains.rs`: each wire name beside its Rust command
path, its bulk routes and the kinds of job it runs. `define_runtime_domains` expands that list into a module per domain
with its plugin builder, dispatch handler, routes and job kinds; the application's build script reads the same file to
declare the inline plugins and their permissions.

## In the application

The application manages core's state before any plugin sets up: `ExecutionState`, `AssetMountState`, `SessionScopes`,
`AppDirectories`, `WindowHandles`, `WindowCloseApprovals` and the `JobRegistry`. Core's plugins answer four command
domains: `assets` (probing and listing roots, engine detection, text commits and change subscriptions, with the bulk
routes `read_asset` and `read_texture`), `jobs` (list, attach and cancel), `sessions` (binding openings to scopes and
releasing them) and `transport` (the endpoint the bulk routes are served at).

The `typescript-bindings` feature derives the Specta types and command bindings. The `fixtures` feature gives the plugin
crates' tests helpers for opening sessions and reading what a session read.

## Checks

Run `cargo test -p xrf-app-core` from the repository root.
