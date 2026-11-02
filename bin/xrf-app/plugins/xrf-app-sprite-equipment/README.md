# xrf-app-sprite-equipment

Equipment sprite sheets in the XRF desktop application: the inventory icon sheet (such as `ui\ui_icon_equipment`)
opened as a session of its document with the configuration naming what sits on it, edited, previewed and reloaded, and
packed back into a texture as a job.

## Layout

One type to a file, grouped by concern:

- `sprite/`: the opened sheet.
  - `EquipmentSprites`: the sheets open, each held by the tool that opened it; a reload is a handle of its own, which
    replaces the one it reloaded once it commits.
  - `EquipmentSpriteOpen`: everything one opening of a sheet was asked to read.
  - `EquipmentSheetSource` and `EquipmentSheetLocation`: where a sheet is read from (a loose `.dds` file or a sheet of
    the roots named by its engine reference), and where it turned out to be and whether anything can write there.
  - `EquipmentSpriteDocument` and `EquipmentSpriteMetadata`: one opened sheet with the picture it is drawn from, and
    what the editor is told about it.
- `request/`: the requests the commands and the route take, one to a file.
- `commands/`: opening, restoring, reloading and closing a sheet, and packing every declared inventory icon into one
  sheet.
- `routes/`: `read_preview`, the bulk route serving an open sheet as the PNG it is drawn from.
- `plugin` and `ipc`: `SpriteEquipmentPlugin` and its command domain.
- `tests`: the crate's tests.

## In the application

`SpriteEquipmentPlugin` manages `EquipmentSprites`. The `sprite-equipment` domain serves the bulk route
`read_preview` and runs the job `sprite-equipment.pack`.

## Checks

Run `cargo test -p xrf-app-sprite-equipment` from the repository root.
