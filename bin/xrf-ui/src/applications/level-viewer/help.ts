import { IApplicationHelp } from "@/core/routing/application";

export const LEVEL_VIEWER_HELP: IApplicationHelp = {
  summary:
    "Lists the compiled levels a game root holds, opens one, and flies it. Sectors stream in as the camera reaches them; opening a level reads no geometry at all.",
  nuances: [
    "Move with `W`/`A`/`S`/`D`, rise and fall with `E`/`Q`, hold `Shift` to cross ground quickly, and drag with the pointer to look.",
    "Surfaces are drawn the way their blender says: dressed in their own texture, modulated by the detail texture the engine tiles over it, and cut out, blended, added or multiplied as the shader table asks. The toolbar switches each of those off to compare.",
    "A compiled level carries occlusion but no light, so what lights it is the viewer's own sun - steer it in the Lighting panel, or take the direction the level was compiled against. The `Sun` toggle draws it in the sky, so its bearing is read rather than inferred from the surfaces.",
    "Surfaces are drawn with the gloss the engine writes for them, `def_gloss`, which is two of two hundred and fifty five: a level's geometry has no sheen, however low the sun is put.",
    "The readout is measured, not estimated: frame time, resident sectors, draw calls, triangles and geometry held.",
    "Problems lists what could not be drawn as the level asked: textures the roots could not answer for, shader table entries the library could not describe, and drawables the packer could not read.",
  ],
  limitations: [
    "The sun term the engine reads out of a lightmap and out of the vertex colour is not applied, so a surface facing away from the viewer's sun is lit only by the ambient standing in for the sky.",
    "Lighting is not read from the weather configs, so a level is never shown at a particular hour of a particular weather the way the game shows it.",
    "Only levels shipping `level.geom` are offered, since a level without it has no geometry to draw.",
  ],
};
