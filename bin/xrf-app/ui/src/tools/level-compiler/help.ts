import { EToolId, IToolHelp } from "@/core/routing/tool";

export const LEVEL_COMPILER_HELP: IToolHelp = {
  summary:
    "Builds an SDK Level Editor scene into a level directory, as the Level Editor's Build does: always the game files " +
    "(`level.spawn`, `level.game`, particles, sounds and the rest), and, given the objects roots and game materials, " +
    "the `build.prj` the level is compiled from, with `level.ltx`, the occluders, details, the AI map, the impostors " +
    "and the breakable parts. This is what `xrf-cli level build` does. With compile steps ticked, it then compiles the " +
    "level it wrote, as `xrf-cli level compile` does, every step by the tools' own compilers.",
  workflow: [
    "Pick the scene's `.level`; its parts are read from the directory beside it.",
    "Pick the level directory to write into.",
    "Leave the objects roots empty for a decompiled scene, whose objects lie in `rawdata\\objects` beside its " +
      "`levels`; add roots, such as the SDK's, for objects kept elsewhere.",
    "Pick the game materials, `gamemtl.xr`, to write `build.prj`, and the textures root where the scene holds details " +
      "or impostors, such as the one its decompile wrote.",
    "To compile as well, tick the steps (geometry, light, details, AI map), choose the light quality, and pick the " +
      "game data the compilers read.",
    "`Build` reports every file written and what it replaced, and lists any part it could not make; with steps " +
      "ticked, `Build and compile` also reports each step's time.",
  ],
  nuances: [
    "Without the game materials only the game files are written, as for a scene of no geometry.",
    "`build.prj` carries the scene's own build options, as the Level Editor writes them. For a quick look at a level, " +
      "choose the draft light quality: it lights at 3 lightmap texels a metre with one sample, under a sky and sun " +
      "regenerated at quality 1.",
    "The impostor atlas grows to 4096 a side by default, as Anomaly's largest levels ship it; the SDK atlas limit " +
      "keeps SDK 0.7's 2048 and refuses a level of more than 128 impostors.",
    "Files the build writes that the directory already holds are replaced only when overwriting is chosen, and a file " +
      "of the build's own it would not write again is refused, since the SDK's Build would move it aside.",
    "Before any step starts, the compile checks each step's inputs and a texture description (`.thm`) for every " +
      "texture `build.prj` names, each from the textures root first and the game data's `textures` after it. " +
      "Stopping the job stops the running step and keeps what was already written.",
    "The light is baked in the same pass as the geometry, so ticking the geometry ticks the light, and the light is " +
      "offered only with the geometry. Geometry compiled without its light gets flat neutral light: enough to walk " +
      "the level, not to judge its look.",
    "The light, the details and the AI map's covers trace their rays on the GPU chosen in the renderer settings where " +
      "it supports ray tracing, or on the CPU, the reference, when chosen or where no GPU can. Existing " +
      "`level.details` and `level.ai` are replaced only when overwriting is chosen.",
  ],
  limitations: [
    "The objects roots are typed rather than picked.",
    "Ticking the details for a level whose scene holds none stops the compile after the build: the build writes no " +
      "`build.details` to light.",
  ],
  relatedTools: [EToolId.LEVEL_DECOMPILER, EToolId.LEVEL_VIEWER],
};
