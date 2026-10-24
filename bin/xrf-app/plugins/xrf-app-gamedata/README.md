# xrf-app-gamedata

Verification of a gamedata project in the XRF desktop application, run as a job. The checks themselves are
`xrf-gamedata`; this crate runs the selected ones and reports their verdicts to the page.

## Layout

- `verify/`: what a verification reports.
  - `GamedataVerifySummary`: what a whole verification reports back to the page.
  - `GamedataCheckSummary`: one check's verdict, as the page shows it.
- `request/`: `GamedataVerifyRequest`, what a verification was asked to do.
- `commands/`: `verify_project`, which runs the selected checks over a project, one verification at a time.
- `plugin` and `ipc`: `GamedataPlugin` and its command domain.

## In the application

`GamedataPlugin` holds no state of its own. The `gamedata` domain runs the job `gamedata.verify`.

## Checks

Run `cargo test -p xrf-app-gamedata` from the repository root.
