# xrf-app-system

Desktop integration in the XRF desktop application that belongs to no editor in particular, grouped by the question
each surface asks: what a filesystem path holds and where tools write, what this instance is and what it costs the
machine, how the window closes, and where the webview's log goes.

## Layout

- `paths`: filesystem paths as the application addresses them: what one currently holds, revealing one in the file
  manager, the application's data root, and where tools write when nobody named a destination.
- `diagnostics`: what this instance is and what it costs, as separate readings because they live for different times:
  the build (fixed when compiled), the host (`HostInfo`, fixed while the application runs), the current usage
  (`RuntimeSnapshot`, `DescendantUsage` for the processes below this one), per-process memory (`memory`, cheap enough to
  poll from a status bar; the webview's processes are read on Windows only, and the GPU's figures come from the
  renderer), the environment variables set, and `StartupLog`, the log's first lines.
- `window`: the window as its page sees it: approving a close the page asked about first, and what the page decided
  about leaving.
- `ui_log`: the webview's log as its page sends it: the most verbose level any of its records is written at
  (`get_ui_log_level`), so the page sends none below it, and the batches it sends (`write_ui_log`, at most 512 records,
  a larger one refused whole), each record filtered again and written to this run's UI log file
  (`xrf_app_core::logging::UiLog`).
- `plugin` and `ipc`: `SystemPlugin` and its command domain.

## In the application

`SystemPlugin` manages `MachineProbeState` and `WebviewProcessProbe`. The application writes `StartupLog` as it starts,
before any plugin is set up, and manages the `UiLog` its logger opened, which the `ui_log` commands take. The `system`
domain runs no jobs.

## Checks

Run `cargo test -p xrf-app-system` from the repository root.
