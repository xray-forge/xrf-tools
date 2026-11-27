# xrf-game

The native game binary, at its earliest prototype: it opens a level of an X-Ray game tree in a window and walks the
actor over it, or flies the player's camera. Its library holds how this program is put together, kept apart from `main` so it is tested:
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
- `--mode fly` (the default) flies a free camera; `walk` starts on foot, the actor walking the level's collision.
- `--preset <name>` starts from a preset in place of the settings file's; `--set <path>=<value>`, repeated, changes
  one setting for this run, over the settings file, never written back.
- `--mcp` turns the control endpoint on: requests on standard input, answers on standard output, as the MCP bridge
  ([xrf-game-mcp](../mcp/README.md)) starts the game. Its input ending quits the game. Built only with the `mcp`
  feature (on by default); without `--mcp` the game reads no input.

The cursor is grabbed for mouse look, turned at `input.sensitivity` (0.0025 rad a count). The keys are the tree's own,
its `configs\default_controls.ltx` (`GameControls`: `bind` and `bind_sec` lines, X-Ray's key names read as SDL's,
`GameKeyNames`), Call of Pripyat's where the tree has none: W, S, A and D move, Space jumps (rises in flight), Left
Ctrl sinks in flight, Left Shift walks (flies faster), X toggles sprinting; N switches between walking and flight, as
an engine's noclip does. The player's own bindings (`input.bindings`, `input.bindings_secondary`, set by the console's
`bind` and `bind_sec`) are laid over them (`GameBindings`), and the settings file keeps only those. Escape or the
window's close quits.

## The run

What the game holds lives as long as one of three things. The host, for the whole run: the shell (`GameShell`: the
window and the renderer) and `GameHost` (the tree, its configs read once and kept, its game materials, the audio
server). The game, what a save holds: `GameProgress` (the seed, the game's own time `GameTime`, the object registry,
A-Life, the player's object), the player's movement, and the hook registry its systems act through
(`xrf-game-hooks`, every point registered with its system's own behaviour at the game's start, which a script will
override by name). The level, built again for each: `GameLevelState` (the physics
world, the game graph's level) and what the world shows of it. `GameCommand::ReplaceLevel` hands the loop a level built
beside it in whole; the game goes on over it.

The arguments build a `GameLoadRequest` (the tree, the level, a new game and its seed, how the player moves), and the
load runs on a thread of its own (`GameOpening`) while the main thread starts the shell and draws the loading view:
the flow (`GameFlow`) is Loading from the launch, the game starts under the view once opened, and is Playing once the
level is streamed. `GameSession` opens what the load asks on the mounted tree: the level, the sound (`GameSound`:
Steam Audio from `.sdks` or beside the executable, dry without it), what the world shows, and a new game's
`GameState`: every object of the tree's `all.spawn` in the object registry (`xrf-game-objects`), the spawn's actor as
the player's object, the player's movement at the level's start, and A-Life (`xrf-game-alife`) by the tree's
`[alife]`, which switches the level's objects online around the player's feet; the game's day runs by `[alife]`'s
`start_time` and `time_factor` (`GameClock`). The spawn is read once for its objects, its game graph and its patrols
(`GameTree::read_spawn`, `GameSpawn`); every config the game reads (`[alife]`, `[actor]`, `[hud_sound]`, the controls,
the simulation's) comes through `GameConfigs`, the tree's `configs` in its dialect (a DLTX mod's overrides applied)
with `system.ltx` resolved, read by `xrf-game-config`, and what does not read is logged as issues naming its file,
section and key. The simulation's
start squads are then created with their members (`xrf-game-simulation`'s `SimulationStart`), every draw keyed by the
game's seed: `--seed` names one, or a new one is made from the clock and logged, so a run can be repeated. That new
game (`GameStart`) is read beside the level on the session's pool, and so is the level's physics world
(`xrf-physics`): its `level.cform` as one mesh, the faces of passable game materials (`gamemtl.xr`) left out, which
takes the longest: Anomaly's `zaton` opens in about 1.9 s, 157,722 of its 2.5 million faces passable, with 393 squads
of 937 members across its levels (seed 7).
`GameRun` starts the load, then the shell: the window through `xrf-game-platform`, the view (`GameView`: the render
queue with the game's pipeline cache, its world, the viewport filling the window), and once opened the game loop
stepping the session's state.

What the loop publishes (`GameFrame`) is the player's camera and velocity, the game's day and every online object's
pose, in id order; what happens once is an event (`GameEvent`: the player's steps, played in order by
`GameFootsteps`, and the objects A-Life switches, which nothing shows yet), queued in order; what a tool asks is read
from the state between ticks (the control endpoint's status). The look is presentation's (plan decision 26, `GameLook`): the mouse turns it on the main thread at once, the world's view reads it
at each render frame's start (`GameViewFeed`), its position the latest tick's carried forward by its velocity, and the
loop is sent the heading whole once a frame it changed (`MovementCommand::Face`). A look reaches the view in 5.7 ms at
the median, 11 at p90, against 12 and 19 when the loop turned the view a tick at a time and the view blended a tick
behind (Zaton, 2026-11-27). The day blends a tick behind.

The run owns the settings store (`xrf-game-settings`' `GameSettingsStore`); the console (`GameConsole`) is its client,
and the bridge's lines run through the console. `GamePlay` applies what changed once a frame, by group: the view's
groups to the renderer, the input group to the running game, which owns its keys (its tree's controls with the
player's bindings over them) and binds them again; the run saves what changed as it quits.

`GamePlay` reads each platform event through the bindings into the loop's movement commands (the player held while
loading) and, with `--mcp` or `--listen`, answers the control endpoint between frames (`GameControl`): `ready` once the
renderer reports the level ready, then `status`, `pose` (a camera command to the loop, the look faced with it),
`capture` (the window's next frame to `appdata/xrf-game/captures/`) and `quit`. A loop that faulted or stopped ends the
run with its reason. On quit it stops the loop, detaches and stops the view, then drops the window.

## Files

- Its own files sit under `appdata/xrf-game/` beside the executable when that directory accepts files, else in its
  local data directory (`com.xrf.game`): `settings.ltx`, and `cache/` with the renderer's pipeline cache
  (`GameDirectories`).
- Its logs are `xrf-game.<UTC start>.<process>.log` in the `appdata/logs/` it shares with the desktop app, through
  [xrf-logging](../../../crates/xrf-logging/README.md) (`GameLogSetup`).

## Checks

Run `cargo test --locked -p xrf-game` from the repository root. Opening a level, drawing it and shutting down are
checked by running the game on a tree.
