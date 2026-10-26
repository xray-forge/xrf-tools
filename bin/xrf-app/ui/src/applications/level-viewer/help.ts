import { IApplicationHelp } from "@/core/routing/application";

export const LEVEL_VIEWER_HELP: IApplicationHelp = {
  summary:
    "View a compiled level from a game installation or gamedata directory, archives included. Fly through it, preview its weather, and find what draws wrong. The viewer never changes game files.",
  workflow: [
    "Set `Game root` to the game's installation or gamedata directory and select `List levels`. An empty list means the root holds no compiled level, one with a `level.geom` file.",
    "Choose a `Level` and a `Dialect`, then select `Open`. `LTX` reads standard configs; `DLTX` also applies `mod_*.ltx` patches.",
    "Leave `Engine` on `Auto` unless it guesses wrong. It picks `Extended` for Anomaly's executables or `fsgame.ltx`, or for Atmosfear's `[weather_cycles]`, and `Vanilla` otherwise. The engine decides the level's weather, sun, particles and bloom; an override is remembered for that root.",
    "Once the level loads, click the viewport and fly through it. The whole level loads at once.",
    "Compare times of day in the `Weather` panel. When something looks wrong, check `Problems` for failed assets and `Surfaces` for the shader and textures that draw it.",
  ],
  nuances: [
    "Fly with `W`/`A`/`S`/`D`, rise with `E`, descend with `Q`, and hold `Shift` to go faster. Look around by dragging with the left mouse button or with the arrow keys. After using a panel, click the viewport to give it the keyboard back.",
    "`Camera` sets the field of view, the speed in metres per second, the `Shift` boost, and the pointer sensitivity in degrees per pixel. `Go to` moves to a position in metres, facing a heading and pitch in degrees; its paste field accepts text such as `x 0 y 10 z 0 h 90 p 0`.",
    "Click a surface or a spawned object to select it and show it in the open `Surfaces` or `Spawn` panel. Hold `Alt` to click through glass. Choosing an object in `Spawn` selects it too.",
    "A toolbar button with a dot switches its feature on or off; right-click it for its settings. A button with a triangle opens a menu. The toolbar changes this view only, starting from Settings > Rendering; `Reset to the preset` returns a popover to those defaults. A feature switched off in Settings says so in its popover.",
    "`Shading` chooses what the viewport shows: the final frame, `Clay`, `Shader colours`, `Overdraw`, or a stage of the frame such as `Albedo range`, `Shadow cascades` or `Luminance`. FSR's views appear only while FSR draws. It also switches wireframe, bumps, parallax and wall marks.",
    "`Look` sets exposure, lighting, image correction, bloom and the sky's debanding from a preset: the game's defaults, XRF's, OpenXRay's, Anomaly's, the application's settings, or `Edited by hand`.",
    "The `Weather` panel picks a cycle and sets its time and speed. A level opens at noon with the clock paused, and its weather and time are remembered. The preview covers lighting, fog, sky, clouds, wind, rain, wet surfaces, lightning, and the overrides in `level.env_mod`.",
    "Editing a weather value in a toolbar popover switches the lighting to `Manual`, starting from what is shown. Switch back to `Weather` to return to the cycle, or select `Reset to the weather` to undo a popover's edits. Manual values also light a level whose weather cannot play.",
    "With the `Vanilla` engine, the sun follows the weather keyframes, unless `Dynamic sun` places it by the time of day. `Extended` uses the game's sun table, and `Manual` uses your angles. `Use the level's compiled sun` in the `Sun` settings restores the direction the level's lighting was baked with. `Overlays` > `Sun` marks the sunlight's direction with a dot.",
    "The `Sun` button lights up while the sky draws a sun or a moon. Its settings control the lens flares and the sunshafts; their quality and minimum density are remembered per level.",
    "A weather effect replaces the cycle for a while; changing the time ends it. Lightning runs on real time, so it strikes even while the clock is paused.",
    "`Surfaces` lists shader table entries by ID, with their pass and whether a script or `shaders.xr` defined them. `Problems` lists failed textures, unresolved or unsupported surfaces, and geometry skipped while loading. Weather configuration problems appear in the `Weather` panel.",
    "Objects a new game removes (`new_game_setup.ltx` `[remove_objects]` and `dynamic_item_spawn.ltx` `[replace_items]`) are dimmed in `Spawn` with the reason, and drawn only with `Spawn` > `Released`.",
    "Frame timings show at the top left of the viewport and camera coordinates at the bottom right; `Overlays` > `Readouts` hides both. The status bar shows the memory used by the backend (`B`), the webview (`W`) and the GPU (`G`); hover over it for details. The `Renderer` panel has the full counts, timings, GPU memory and applied settings.",
    "`F9` (`File` > `Take screenshot`) saves the viewport's frame, without its overlays, to the screenshots folder, with a JSON report beside it of the view and what the frame cost; the passes are timed on the GPU for the shot. The note that pops up opens, reveals, copies or deletes it.",
    "`F11` (`File` > `Capture map…`) draws the level straight down, tile by tile, under the game's map weather, or under the current weather held at noon where the game has none. `File` > `Capture cubemap…` draws the six faces around the camera. Both save to the screenshots folder and export as DDS (BC7 by default), PNG or TGA; a map can also carry smart terrains, restrictors, anomalies, level changers, patrols and graph points over it or alone, alone also as an SVG.",
    "`File` > `Export view…` writes the camera, weather, look, toolbar and settings to a JSON file, in the application's folder of views by default. `Import view…` lists what a file would change and applies the values you keep ticked.",
    "`Overlays` > `Spot meter` reads the light at the centre of the viewport into the readouts: its stops above or below middle grey as exposed, and the albedo, gloss and distance of the surface there. `Overlays` > `Calibration props` stands a grey and a white sphere, a mirror ball and a colour chart three metres ahead of the camera, lit like the level, to judge its light by.",
  ],
  limitations: [
    "Weather sounds do not play.",
    "Rain wets exposed surfaces only within 20 metres of the camera with the `Vanilla` engine, or 25 metres with `Extended`. Raindrops hit shadow-casting geometry seen from above rather than the game's collision, so impacts can differ.",
    "Lightning bolts end at `y = 0` rather than at the terrain.",
    "Items a new game puts in place of `[replace_items]`, and objects released by hardcoded scripts, are not shown.",
    "Grass sways by position, so single tufts can move differently from the game.",
  ],
};
