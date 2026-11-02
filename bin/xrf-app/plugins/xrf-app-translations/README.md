# xrf-app-translations

Translation projects in the XRF desktop application: a project opened as a session, the edits of one file saved into
it, and a project verified, formatted, built and imported as jobs. The translation formats and passes are
`xrf-translation`; this crate holds a project open, follows its files, and answers the editor.

## Layout

One type to a file, grouped by concern:

- `project/`: the opened project.
  - `TranslationProjects`: the translations projects open, each held by the tool that opened it and read again by a
    commit that moved a file it read.
  - `TranslationProjectRead`: how a project is read through the shared roots, recording every file and listing it
    reads.
- `save/`: `TranslationSavePlan` and `TranslationSaveOutcome`, what one save was addressed to, taken as a single read
  before the filesystem work begins, and how the save ended.
- `format/`: formatting a project.
  - `TranslationsFormatRun`: normalizing the JSON translation sources under a directory, or checking that they are.
  - `TranslationFormatGuard`: what keeps a formatting run off the files an open editor holds, since a save after the
    format would write the editor's pre-format buffers back.
- `request/`: the requests the commands take, one to a file.
- `commands/`: opening, restoring and closing a project; reporting which layout a set of roots looks like; checking
  that a language can hold a value; saving one file's pending edits; and verifying, formatting, checking the format
  of, building and importing a project.
- `plugin` and `ipc`: `TranslationsPlugin` and its command domain.
- `tests`: saves and formatting runs against an open project.

## In the application

`TranslationsPlugin` manages `TranslationProjects`. An open project records every file it read and reads them again
when a commit moves one, a save among them. The `translations` domain runs the jobs `translations.verify`,
`translations.format`, `translations.check-format`, `translations.build` and `translations.parse` (importing one
language's XML string tables into JSON sources).

## Checks

Run `cargo test -p xrf-app-translations` from the repository root.
