# xrf-app-configs

LTX configuration projects in the XRF desktop application: a project opened as a session of the roots it reads, its
resolved sections and the findings of its verification, and the formatting and verification of a directory as jobs. The
reading, resolution and checks themselves are `xrf-ltx`, `xrf-dltx` and `xrf-ltx-inspect`; this crate holds them open
and answers the page.

## Layout

One type to a file, grouped by concern:

- `project/`: the opened project.
  - `ConfigsProjects`: the configs projects open, each held by the tool that opened it.
  - `OpenedConfigsProject`: how a project is assembled over the shared asset roots and listed, recording what it read
    so that a commit can read it again.
  - `ConfigsProject` and `ConfigsProjectDescriptor`: one opened project with whatever has been resolved out of it, and
    what an open answers under the opening's identity.
  - `ConfigsResolvedRoot`: one entry point's resolution and the answers read from it since.
  - `ConfigsDocument`: one config as the authored view renders it.
- `format/`: `ConfigsFormat`, formatting the configs a set of roots holds, or checking that they are formatted.
- `request/`: the requests the commands take, one to a file.
- `commands/`: one file per command: opening, restoring and closing a project; reading a document, resolved sections
  and a section's scheme; listing findings and resolved sections; verifying, formatting and checking the format of a
  directory.
- `plugin` and `ipc`: `ConfigsPlugin` and its command domain.
- `tests`: formatting committed through the shared roots.

## In the application

`ConfigsPlugin` manages `ConfigsProjects`. An open project is read again on the pool once a commit moves a config it
read, since an installation's configs take a while. The `configs` domain runs the jobs `configs.verify`,
`configs.format` and `configs.check-format`.

## Checks

Run `cargo test -p xrf-app-configs` from the repository root.
