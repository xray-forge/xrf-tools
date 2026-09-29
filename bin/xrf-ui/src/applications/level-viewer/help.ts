import { IApplicationHelp } from "@/core/routing/application";

export const LEVEL_VIEWER_HELP: IApplicationHelp = {
  summary:
    "Lists the compiled levels a game root holds, opens one, and flies it. Sectors stream in as the camera reaches them; opening a level reads no geometry at all.",
  nuances: [
    "Move with `W`/`A`/`S`/`D`, rise and fall with `E`/`Q`, hold `Shift` to cross ground quickly, and drag with the pointer to look. `Camera` in the toolbar sets the field of view, the walking speed, what the modifier multiplies it by, and how far a pixel of pointer movement turns.",
    "Surfaces are drawn the way their shader says: dressed in their own texture, modulated by the detail texture the engine tiles over it, and cut out, blended, added or multiplied as it asks. None of that is a toggle - a surface that cuts out always cuts out, and the detail texture is half of what an X-Ray surface is rather than a comparison - so the one thing the toolbar switches off is the texture itself.",
    "A shader that ships a renderer script is that script: the engine looks for `shaders/r2/<name>.s` before it reads a class out of `shaders.xr`, and uses it instead when it is there. That is what makes a lamp's light planes additive, a wall mark composite rather than cover, and a self-lit surface draw nothing at all - each of which its class alone would draw as opaque black. A pass a script declares is also drawn unlit, as the engine draws it: it names its own shaders and runs after the lighting rather than into it, so a mark multiplied into a wall is neutral where its texture is mid grey instead of brightening the rectangle it covers. The Materials panel names the script a surface was read from.",
    "A compiled level carries occlusion but no light, so its weather lights it: the Weather panel plays the cycle the level's `weathers` resolves to, or any other of the game's, at a time of day it can scrub, run and speed up, and remembers per level. Switched to `Manual`, or where the weather does not read, the toolbar's `Sun`, `Fog` and `Wind` light it instead; their toggles work either way. The `Sun` toggle draws the sun in the sky, so its bearing is read rather than inferred from the surfaces.",
    "On a vanilla game the sun stands where the keyframes put it, or where OpenXRay computes it with `Dynamic sun`; on an extended one, where the game's sun table puts it. The engine target is set in Settings.",
    "A weather effect plays over the cycle from the clock's time as the game plays one: a few real seconds into the cycle's next keyframe, the effect's own keyframes, then back into the cycle. Scrubbing the clock ends it.",
    "Surfaces are drawn with the gloss the engine writes for them, `def_gloss`, which is two of two hundred and fifty five: a level's geometry has no sheen, however low the sun is put.",
    "A surface with no texture on it - with `Textures` off, or wherever a file could not be read - is drawn in a colour of its shader table entry, the same colour in every sector. A surface that has its texture takes its colour from it.",
    "The readout is measured, not estimated: frame cost sits over the top left of the viewport and the camera position over the bottom right, both inert - they cannot be clicked, dragged or selected, so a drag that starts on one still flies the camera. `Readout` takes both off together, for a clean look at the level. The status bar carries what is resident, with whatever the viewer is doing put in front of it rather than replacing it.",
    "Surfaces lists the level's whole shader table, one row per entry, numbered by the shader id its surfaces refer to it by: what each is drawn as, and whether that was read from a renderer script or from the blender class in `shaders.xr`. It is where to look when a surface is drawn in a way the game does not.",
    "Problems lists what could not be drawn as the level asked: textures the roots could not answer for, shader table entries the library could not describe, and drawables the packer could not read.",
  ],
  limitations: [
    "The sun term the engine reads out of a lightmap and out of the vertex colour is not applied, so a surface facing away from the viewer's sun is lit only by the ambient standing in for the sky.",
    "The weather plays its light, fog, sky, clouds and wind, its effects, its rain and the surfaces it wets, its thunder and the level's own `level.env_mod` overrides; neither its sounds nor its particles play.",
    "Rain wets what it reaches near the camera, as the game's R3 and R4 renderers do: 20 metres out, 25 on the extended engine, where the level's shadow casters seen from above leave it open.",
    "Rain stops at what stands over it as the level's shadow casters see it from above, rather than at the collision form the game's drops are ray-picked against, so it lands on a surface whose shape the two draw differently.",
    "Bolts strike on real time, as the game's do, so they strike on a paused clock too. A bolt reaches down to the ground plane at height zero, where the game ray-picks the level's collision form first.",
    "Grass casts no shadow, as the game draws it by default (`r2_sun_details` off), and a waving tuft picks its wave by its place where the game picks it at random.",
    "Only levels shipping `level.geom` are offered, since a level without it has no geometry to draw.",
  ],
};
