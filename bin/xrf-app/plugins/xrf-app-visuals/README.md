# xrf-app-visuals

Visuals in the XRF desktop application: OGF models opened with their motions, each a session of its own, browsed by
folder, and shown in a viewport through `xrf-app-render`'s units. The visual format is `xrf-ogf`, and `xrf-visual`
packs a model for the renderer.

## Layout

One type to a file, grouped by what is opened:

- `model/`: the opened model.
  - `VisualModels`: the models open, each held by the tool that opened it.
  - `SelectedVisual` and `SelectedVisualDescription`: one opened model (the visual packed for the renderer, what
    posing needs of it) and what the viewer shows of it, both made once at the open.
  - `VisualRenderSource`: the open model as the native renderer draws it, a scene of one object and no level.
- `pose/`: `VisualPoses` and `PosedVisualMotion`, the motions viewers pose their models by, each baked for an open
  model and held as an opening of the viewer's own, since what is played is view state rather than part of the model.
- `browse/`: `VisualBrowses`, the roots each model browser lists, held apart from any model it opens.
- `commands/`: opening, restoring and closing a browse and a model; listing, opening, describing and closing a model's
  motions; and showing a model in a viewport.
- `plugin` and `ipc`: `VisualsPlugin` and its command domain.

## In the application

`VisualsPlugin` manages `VisualModels`, `VisualPoses` and `VisualBrowses` and answers the `visuals` domain. It runs no
jobs.

## Checks

Run `cargo test -p xrf-app-visuals` from the repository root.
