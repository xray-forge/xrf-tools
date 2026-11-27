# xrf-game

The native game binary, at its earliest prototype: it opens a level of an X-Ray game tree in a window and flies the
player's camera over it. Its library holds how this program is put together, kept apart from `main` so it is tested:
arguments, directories, logging, the key bindings, the tree and level it opens, the view it draws, the game's systems
composed into the loop's state, and the run that ties them. What it draws with is
[xrf-game-settings](../settings/README.md)'.

## Running

```powershell
cargo run --profile dev-optimized -p xrf-game -- --root C:\Games\anomaly --level zaton
```

- `--root <path>` names the game tree: an installation with its archives (Anomaly), a folder of volumes, or loose
  `gamedata`, mounted and its engine detected as the tools do (`GameTree`). Without `--level` the game lists the
  tree's levels and exits.
- `--level <name>` names the level by its folder under `levels` (`GameLevel`). The camera starts at the level's start:
  an arrival or the actor's eye under open sky, nearest its centre.
- `--mode fly` (the default) flies a free camera; `walk` comes with the physics engine and refuses to start for now.
- `--preset <name>` starts from a preset in place of the settings file's; `--set <path>=<value>`, repeated, changes
  one setting for this run, over the settings file, never written back.
- `--mcp` turns the control endpoint on: requests on standard input, answers on standard output, as the MCP bridge
  ([xrf-game-mcp](../mcp/README.md)) starts the game. Its input ending quits the game. Built only with the `mcp`
  feature (on by default); without `--mcp` the game reads no input.

The cursor is grabbed for mouse look. W, S, A and D fly, Space rises, Left Ctrl sinks, Left Shift flies faster, as
Call of Pripyat binds them (`GameBindings`); Escape or the window's close quits.

## The run

`GameSession` opens what a run plays, before any window: the tree and level, the sound (`GameSound`: Steam Audio from
`.sdks` or beside the executable, dry without it), what the world shows, and a new game's `GameState`: every object of
the tree's `all.spawn` in the object registry (`xrf-game-objects`), the player's movement at the level's start, and
A-Life (`xrf-game-alife`) by the tree's `[alife]`, which switches the level's objects online around the actor. The
spawn is read once for its objects, its game graph and its patrols (`GameTree::read_spawn`, `GameSpawn`); every config
the game reads, `[alife]` and the simulation's, comes through `GameConfigs`, the tree's `configs` in its dialect (a
DLTX mod's overrides applied) with `system.ltx` resolved, and the simulation's issues are logged. The simulation's
start squads are then created with their members (`xrf-game-simulation`'s `SimulationStart`), every draw keyed by the
game's seed: `--seed` names one, or a new one is made from the clock and logged, so a run can be repeated. That new
game (`GameStart`) is read beside the level on the session's pool: Anomaly's `zaton` opens in about 0.7 s with 393
squads of 937 members across its levels (seed 7).
`GameRun` is the shell around it: the window through `xrf-game-platform`, the view (`GameView`: the render queue with
the game's pipeline cache, its world, the viewport filling the window), and the game loop stepping the session's state.
The world draws the loop's camera and day through `GameViewFeed`. `GamePlay` then reads each platform event through the bindings into the
loop's commands and, with `--mcp`, answers the control endpoint between frames (`GameControl`): `ready` once the
renderer reports the level ready, then `status`, `pose` (a camera command to the loop), `capture` (the window's next
frame to `appdata/xrf-game/captures/`) and `quit`. On quit it stops the loop, detaches and stops the view, then drops
the window.

## Files

- Its own files sit under `appdata/xrf-game/` beside the executable when that directory accepts files, else in its
  local data directory (`com.xrf.game`): `settings.ltx`, and `cache/` with the renderer's pipeline cache
  (`GameDirectories`).
- Its logs are `xrf-game.<UTC start>.<process>.log` in the `appdata/logs/` it shares with the desktop app, through
  [xrf-logging](../../../crates/xrf-logging/README.md) (`GameLogSetup`).

## Checks

Run `cargo test --locked -p xrf-game` from the repository root. Opening a level, drawing it and shutting down are
checked by running the game on a tree.
