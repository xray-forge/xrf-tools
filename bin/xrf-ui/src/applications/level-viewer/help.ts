import { IApplicationHelp } from "@/core/routing/application";

export const LEVEL_VIEWER_HELP: IApplicationHelp = {
  summary:
    "Explore a compiled level from a game installation or gamedata directory, including archives. Fly through the scene, preview its weather, and inspect rendering problems. The viewer does not change game files.",
  workflow: [
    "Choose the game's installation or gamedata directory as `Game root` and select `List levels`.",
    "Choose a `Level` and its configuration `Dialect`: `LTX` for standard configs, or `DLTX` to apply `mod_*.ltx` patches. Select `Open`. If the list is empty, check that the root contains compiled levels with a `level.geom` file.",
    "`Engine` is `Auto` unless you choose otherwise: it reads `Extended` for an installation with Anomaly's executables or `fsgame.ltx`, or data with Atmosfear's `[weather_cycles]`, and `Vanilla` otherwise, and says which it found. A `Vanilla` or `Extended` override is remembered for that root. The opened level's engine decides its weather, sun, particles and bloom.",
    "Wait for the level to load, then click the viewport to use the flight controls. The whole level is loaded at once; the `Renderer` panel shows what is resident, what each frame costs, and the settings the renderer actually applied.",
    "Use the `Weather` panel to compare times of day. If something looks wrong, check `Problems` for failed assets and the `Surfaces` side panel for the shader and textures used to draw it.",
  ],
  nuances: [
    "Move with `W`/`A`/`S`/`D`, rise with `E`, descend with `Q`, and hold `Shift` to move faster. Drag with the left mouse button or hold the arrow keys to look around. Click the viewport again after using a panel to return keyboard focus to it.",
    "Click a surface or a spawned object in the viewport to select it: it is outlined, an object also boxed, and the open `Surfaces` or `Spawn` panel shows it. Choosing an object in the `Spawn` panel selects it too. Press `Escape` with the viewport focused to clear the selection.",
    "Use `Camera` to set field of view, speed in metres per second, the `Shift` boost, and pointer sensitivity. `Go to` accepts a position in metres and heading and pitch in degrees. Its paste field also accepts named coordinates such as `x 0 y 10 z 0 h 90 p 0`.",
    "Click a toolbar toggle to switch its feature on or off; right-click it to open its settings. Group buttons such as `Shading` and `Overlays` open on a click. Some rendering features must first be enabled in the application's Settings.",
    "The `Shading` toolbar group chooses what the viewport shows: the final frame, `Clay` (every surface one grey), `Shader colours` (a consistent colour for each shader table entry across sectors), or one of the targets the frame is built from. It also switches wireframe, bumps, and wall marks. Detail textures, transparency, and blending follow each surface's shader.",
    "The `Weather` panel selects a cycle and controls its time and playback speed. A new level starts at noon with the clock paused; weather choices and time are remembered per level. The preview includes lighting, fog, sky, clouds, wind, rain, wet surfaces, lightning, and local weather overrides from `level.env_mod`.",
    "Editing a weather value in a toolbar popover switches to `Manual`, starting from the weather currently shown. Select `Weather` under `Lit by` to return to the cycle. Manual values also provide the fallback when no cycle can play. The weather's sun, its glow and lens flares are drawn with the sky as in the game. `Overlays` > `Sun` marks the direction of the level's sunlight with a dot, which helps at night and with values set by hand.",
    "With the vanilla engine target, the sun follows the weather keyframes unless `Dynamic sun` is enabled to compute its position from the time of day. The extended target uses the game's sun table. Manual mode uses the angles you set. `Use the level's compiled sun` in the `Sun` settings restores the direction used to bake the level's lighting, when available.",
    "The `Sun` toolbar button lights up while the sky draws a sun or a moon. Its settings switch the lens flares and the sunshafts, pick the lens flare a keyframe draws, and set the sunshafts' density, their quality, and a minimum density they never fall below. The quality and minimum are remembered per level with its weather.",
    "`Look` chooses how the level is exposed, lit and image-corrected: the game's console defaults, the application's settings, OpenXRay's or Anomaly's, or values edited by hand. Once values have been edited, `Edited by hand` stays offered to return to after trying another look.",
    "A weather effect temporarily replaces the cycle, then returns to it. Changing the clock's time ends the effect. Lightning uses real time, so bolts can still strike while the weather clock is paused.",
    "The `Surfaces` side panel lists shader table entries by ID, including their rendering pass and whether their definition came from a script or `shaders.xr`. `Problems` lists texture failures, unresolved or unsupported surface definitions, and geometry skipped while reading loaded sectors. Weather configuration findings appear in the `Weather` panel.",
    "Frame timings appear at the top left of the viewport; camera coordinates appear at the bottom right. These readouts do not intercept dragging. Hide both with `Overlays` > `Readouts`. The status bar reports the backend's and the webview's memory; `Renderer` provides detailed counts, timings, GPU memory, and the applied settings.",
    "Objects a new game releases (`new_game_setup.ltx` `[remove_objects]` and `dynamic_item_spawn.ltx` `[replace_items]`) are listed dimmed in the `Spawn` panel with their reason, and drawn only with `Spawn` > `Released`.",
  ],
  limitations: [
    "Weather sounds and ambient particle effects do not play.",
    "Rain wets exposed surfaces within 20 metres of the camera with the vanilla engine target, or 25 metres with the extended target. Rain impacts use shadow-casting geometry viewed from above, so they can differ from the game's collision-based impacts.",
    "Lightning bolts end at the plane `y = 0`, without checking the level's collision geometry. Their endpoints may not match the terrain.",
    "Items that a new game places in place of `[replace_items]`, and objects released by hardcoded script logic, are not shown.",
    "Grass does not cast shadows. Its wind variation is fixed by position, so individual tufts may sway differently from the game.",
  ],
};
