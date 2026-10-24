# xrf-cli

The XRF command line: one binary over the workspace crates, grouped by domain (`archive`, `dds`, `dialog`, `docs`,
`externs`, `gamedata`, `level`, `ltx`, `ogf`, `omf`, `particle`, `profile`, `spawn`, `sprite`, `thm`, `translation`).
It holds no format or asset rule of its own: a command parses its arguments, calls a crate, tells a person what
happened and deposits a typed result for `--json` or `--report`.

## Layout

- `application`: clap assembly and dispatch. It builds the root command with the reporting flags, resolves the output
  a command writes through, runs the matched command and turns its result into an exit code and an envelope.
- `registry`: every domain's `CommandGroup`, in the order help lists them, and `CommandGroup::list_all` for
  `docs generate`.
- `commands/<domain>/`: the domain's `<domain>_group.rs` and one directory per command, holding `command.rs`, its
  `report.rs` and the report's nested shapes. `packed_file/` holds the pack, unpack, repack and verify commands particle
  and spawn libraries share.
- `core/arguments`: the argument groups commands declare, each a type with `declare` and `of`.
- `core/command`: `GenericCommand`, `CommandContext`, `CommandError` and `CommandGroup`.
- `core/report`: the result envelope, where it goes, and `ReportVerdict`, which turns a check report into console lines
  and an exit code.
- `core/reports`: report shapes several commands share.
- `core/terminal`: the terminal's output streams, the logging job long commands report progress through, and how a
  mount or an unreachable file is described.

## Exit codes

`0` success, `1` the command's own failure (an unreadable input, a refused write), `2` a usage error from clap, `3` a
check that judged content and found something.

## Checks

Run `cargo test -p xrf-cli` from the repository root. `cargo make doc-cli` writes the generated command reference to
`target/doc-commands`.
