# xrf-app-dialogs

Dialog projects in the XRF desktop application: the dialogs a project's roots hold, each read with its phrases in a
chosen language. The dialog format is `xrf-dialog`; this crate opens a project as a session and answers the page.

## Layout

- `project/`: the opened project.
  - `DialogProjects`: the dialogs projects open, each held by the tool that opened it.
  - `OpenedDialogProject`: a project as an open read it, with the description made once at the open, which a restore
    answers again without describing every dialog afresh.
- `request/`: the requests the commands take, one to a file.
- `commands/`: opening, restoring and closing a project; reporting which layout a set of roots looks like, for the open
  form to preselect; and reading one dialog with every phrase it declares.
- `plugin` and `ipc`: `DialogsPlugin` and its command domain.

## In the application

`DialogsPlugin` manages `DialogProjects` and answers the `dialogs` domain. It runs no jobs.

## Checks

Run `cargo test -p xrf-app-dialogs` from the repository root.
