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
- `--record <file>` records the session to that file; `--replay <file>` steps a recorded session again on the same
  tree, taking the level, seed and mode from the recording ([Recording and replaying](#recording-and-replaying)).
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

The main thread plays through a chain of presentation layers (`GameLayers`, each a `GameLayer`), topmost first, over
what they share (`GameLayerContext`: the window, the view, the settings, the console, the flow), which play owns and
lends each layer in turn, so no layer holds another:

- `GameControl`, with `--mcp` or `--listen`: the control endpoint. It feeds the input an agent plays to every layer as
  the window's own, and once a frame answers its requests: `ready` once the renderer reports the level ready, then
  `status`, `pose` (a camera command to the loop, the look faced with it), `console`, `capture` (the window's next
  frame to `appdata/xrf-game/captures/`) and `quit`.
- `GameDeveloper`: the console's drawer, the log's records in it, the frame and camera readouts, and the load's view.
- `GamePlayerLayer`: the game itself. The mouse turns the look, the keys bound become the loop's movement commands (the
  player held while loading), and each frame the loop is checked for still stepping and what its steps told is played.

A frame of `GamePlay` waits for the platform's events until a layer's next work is due (`GameLayer::get_due`) or
another thread wakes it through the platform's waker (`PlatformWaker`): the loop as it publishes each frame and as it
stops, the load's thread with each thing it opened, the endpoint's reader with each request, the renderer as the level
first stands ready. So the main thread runs about 130 times a second while a game plays and 30 otherwise, and a request
is taken within 0.1 ms of arriving. It then offers each event down the chain until a layer keeps it
(`GameInputFate`); moves the flow and then every layer on, from the top; and has them draw, from the bottom. A layer
that seizes the keyboard and the mouse, as the console does opening, makes the layers under it let go of what they
hold and frees the cursor (`GameCursor`) until it yields them. What no layer keeps is the window's own: its close, its
resize, and Escape, which quits until a menu takes it. A new layer, a HUD or a menu, is its type and a `push` where
`GameRun` builds the chain.

The context owns the settings store (`xrf-game-settings`' `GameSettingsStore`) outright and lends it to the console
(`GameConsole`) a line at a time, whichever layer enters the line, the drawer or the bridge. `GamePlay` applies what
changed once a frame, by group: the view's groups to the renderer, the input group to the running game, which owns
its keys (its tree's controls with the player's bindings over them) and binds them again; the run saves what changed
as it quits.

The game's one-off sounds, the player's footsteps, reach the level's sound over a channel opened as the level is
shown (`xrf-world`'s `WorldSoundCues`): the main thread reads a sound's file the first time it is played and queues
the cue, and the world starts it at its next frame, so playing one never takes the world's lock, which the render
thread holds while it runs the world for a frame (about 65 µs a frame on Zaton, 0.3 ms at most).

A loop that faulted or stopped ends the run with its reason. On quit it stops the loop, detaches and stops the view,
then drops the window.

## Recording and replaying

A session is reproduced from what it started with and the commands its ticks took. `--record <file>` writes them as
the loop steps (`GameRecorder`, a witness of the loop): one JSON value a line (`GameRecordingLine`), the header first
(`GameRecordingHeader`: the format's version, the level, the seed, the mode, the tick rate), then each command with the
tick that took it (`GameRecordedCommand`), a digest of the state every hundred ticks, and the tick the session stopped
at. Lines reach the disk with each digest, and a recording cut short by a crash ends at its last line.

`GameStateDigest` stands for the state between two ticks: the seed and time, every table of the object registry, what
A-Life keeps between updates, the level's place on the game graph and the player's movement, hashed (FNV-1a) from the
state's `Debug` text so a new field is covered without a line of its own. The physics world is covered through the
player it moves.

`--replay <file>` opens what the header says on the tree `--root` names (`GameReplay`), steps the recorded commands at
their ticks with live input let go and the view facing where the session faced, and compares the state with each
recorded digest (`GameReplayCheck`). The log says how it went: `The replay followed its recording to its end at tick
N: all M state digests matched`, or an error naming the tick of the first digest that differed. From the recording's
end the game takes input again, so a replay can be played on from.

A recording is a debugging aid, not a save: one of another format version is refused, and a build whose steps changed
parts from an older recording at the first digest. A level change is not recorded yet; a recording ends at one.

## Files

- Its own files sit under `appdata/xrf-game/` beside the executable when that directory accepts files, else in its
  local data directory (`com.xrf.game`): `settings.ltx`, and `cache/` with the renderer's pipeline cache
  (`GameDirectories`).
- Its logs are `xrf-game.<UTC start>.<process>.log` in the `appdata/logs/` it shares with the desktop app, through
  [xrf-logging](../../../crates/xrf-logging/README.md) (`GameLogSetup`).

## Checks

Run `cargo test --locked -p xrf-game` from the repository root. Opening a level, drawing it and shutting down are
checked by running the game on a tree.
