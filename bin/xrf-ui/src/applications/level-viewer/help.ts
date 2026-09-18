import { IApplicationHelp } from "@/core/routing/application";

export const LEVEL_VIEWER_HELP: IApplicationHelp = {
  summary:
    "Lists the compiled levels a game root holds, opens one, and flies it. Sectors stream in as the camera reaches them; opening a level reads no geometry at all.",
  nuances: [
    "Move with `W`/`A`/`S`/`D`, rise and fall with `E`/`Q`, hold `Shift` to cross ground quickly, and drag with the pointer to look.",
    "Surfaces are coloured by the shader table entry that draws them, so the same material reads the same way everywhere.",
    "The readout is measured, not estimated: frame time, resident sectors, draw calls, triangles and geometry held.",
  ],
  limitations: [
    "Levels are drawn untextured. Textures and the baked lightmaps are not applied yet, so lighting is a stand-in.",
    "Only levels shipping `level.geom` are offered, since a level without it has no geometry to draw.",
  ],
};
