# xrf-game-settings

What the game draws with and how its window opens: its presets, its settings file, and the changes a run is started
with. The game binary loads them; its console reads and sets the same settings while it runs.

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
the preset's whole. `none` leaves an optional setting unset. A change is kept only where the whole settings still read
back with it: a key the settings hold no setting for, or a value the setting does not take, is logged and left out, and
the rest still applies. Once merged, the settings are held to the bounds the renderer holds them to as it takes them
(`hold_to_bounds`: `view.grass.radius = 120.5` reads back as a whole number), so they read as they are drawn with;
`get_view_options` gives the view's options they draw with.

## Saving

`GameSettings::save_changes(files, changes)` writes changes made while the game runs into the settings file, the first
of `files` that exists, else the first, made where missing. It reads the file's own settings and applies the changes
over them, so what a run was started with for itself (`--preset`, `--set`) is never saved, then writes the file whole
through a staged write: `[settings] preset`, and each setting that differs from that preset in its group's section
(`create_settings_ltx`). The console saves its changes this way with `save`, and as the game quits.

## Paths

`GameSettings::get_entries` lists every setting as the file writes it (`GameSettingEntry`: the dotted path in snake
case and the value as text), and `with_setting(path, value)` sets one the same way, held to its bounds, answering why
not where it cannot.
Every value listed sets back unchanged. The console's variables are these entries.

## Checks

Run `cargo test --locked -p xrf-game-settings` from the repository root.
