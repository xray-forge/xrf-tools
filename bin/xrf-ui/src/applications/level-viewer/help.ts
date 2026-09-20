import { IApplicationHelp } from "@/core/routing/application";

export const LEVEL_VIEWER_HELP: IApplicationHelp = {
  summary:
    "Lists the compiled levels a game root holds, opens one, and flies it. Sectors stream in as the camera reaches them; opening a level reads no geometry at all.",
  nuances: [
    "Move with `W`/`A`/`S`/`D`, rise and fall with `E`/`Q`, hold `Shift` to cross ground quickly, and drag with the pointer to look. `Camera` in the toolbar sets the field of view, the walking speed, what the modifier multiplies it by, and how far a pixel of pointer movement turns.",
    "Surfaces are drawn the way their shader says: dressed in their own texture, modulated by the detail texture the engine tiles over it, and cut out, blended, added or multiplied as it asks. None of that is a toggle - a surface that cuts out always cuts out, and the detail texture is half of what an X-Ray surface is rather than a comparison - so the one thing the toolbar switches off is the texture itself.",
    "A shader that ships a renderer script is that script: the engine looks for `shaders/r2/<name>.s` before it reads a class out of `shaders.xr`, and uses it instead when it is there. That is what makes a lamp's light planes additive, a wall mark composite rather than cover, and a self-lit surface draw nothing at all - each of which its class alone would draw as opaque black. The Materials panel names the script a surface was read from.",
    "A compiled level carries occlusion but no light, so what lights it is the viewer's own sun - steer it in the Lighting panel, or take the direction the level was compiled against. The `Sun` toggle draws it in the sky, so its bearing is read rather than inferred from the surfaces.",
    "Surfaces are drawn with the gloss the engine writes for them, `def_gloss`, which is two of two hundred and fifty five: a level's geometry has no sheen, however low the sun is put.",
    "A surface with no texture on it - with `Textures` off, or wherever a file could not be read - is drawn in a colour of its shader table entry, the same colour in every sector. A surface that has its texture takes its colour from it.",
    "The readout is measured, not estimated: frame cost sits over the top left of the viewport and the camera position over the bottom right, both inert - they cannot be clicked, dragged or selected, so a drag that starts on one still flies the camera. `Readout` takes both off together, for a clean look at the level. The status bar carries what is resident, with whatever the viewer is doing put in front of it rather than replacing it.",
    "Problems lists what could not be drawn as the level asked: textures the roots could not answer for, shader table entries the library could not describe, and drawables the packer could not read.",
  ],
  limitations: [
    "The sun term the engine reads out of a lightmap and out of the vertex colour is not applied, so a surface facing away from the viewer's sun is lit only by the ambient standing in for the sky.",
    "Lighting is not read from the weather configs, so a level is never shown at a particular hour of a particular weather the way the game shows it.",
    "Detail objects - the grass and litter a level plants through `level.details` - are not drawn at all, so a level reads as its geometry alone where the game shows a field.",
    "Only levels shipping `level.geom` are offered, since a level without it has no geometry to draw.",
  ],
};
