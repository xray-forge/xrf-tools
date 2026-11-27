# xrf-game-settings

What the game draws with and how its window opens: its presets, its settings file, and the changes a run is started
with. The game binary loads them; the console and the developer layer will read and edit the same settings.

## The settings

`GameSettings` holds the renderer's own contract where it has one (`render`, the view's features as `view`,
`upscaling`) and how the game's window opens (`window`). A preset (`GamePreset`, parsed from its name in any case) sets
every one of them over the renderer's defaults; a settings file changes what differs from it, and a run's changes
change that again.

`GameSettings::load(files, preset, sets)` reads the first of `files` that exists, starts from `preset` where the run
names one, else the file's, else the default, and applies the file's values, then each of `sets`, written
`<path>=<value>`.

## The settings file

The file names the preset in `[settings]` and a group of settings by its dotted path, each key one setting of it,
written as X-Ray writes keys:

```ini
[settings]
preset = high

[view.shadows]
resolution = 2048

[view.shadows.contact]
steps = 16

[window]
mode = windowed
width = 1600
height = 900
```

Each value is read as the type the preset holds at its key: a flag (`true`, `on`, `yes`, `1` and their opposites), a
number, a variant as the renderer names it (`pbr_neutral` or `pbrNeutral`), or a list, comma-separated, which replaces
the preset's whole. A change is kept only where the whole settings still read back with it: a key the settings hold no
setting for, or a value the setting does not take, is logged and left out, and the rest still applies.

## Checks

Run `cargo test --locked -p xrf-game-settings` from the repository root.
