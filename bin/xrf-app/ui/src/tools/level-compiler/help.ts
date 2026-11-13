import { EToolId, IToolHelp } from "@/core/routing/tool";

export const LEVEL_COMPILER_HELP: IToolHelp = {
  summary:
    "Builds an SDK Level Editor scene into a level directory as the Level Editor's Build does: the game files " +
    "(`level.spawn`, `level.game`, particles, sounds and the rest) always, and, given the objects roots and game " +
    "materials, the `build.prj` the SDK's xrLC compiles with `level.ltx`, the occluders, details, the AI map, the tree " +
    "impostors and the breakable parts. It is what `xrf-cli level build` does, and with compile steps chosen it then " +
    "runs `xrf-cli level compile` on the level it wrote.",
  workflow: [
    "Pick the scene's `.level`; its parts are read from the directory beside it.",
    "Pick the level directory to write into.",
    "Add the objects roots the scene's objects are read from, such as the scene's own `rawdata\\objects` and the SDK's.",
    "Pick the game materials, `gamemtl.xr`, to write `build.prj`, and the textures root where the scene holds details " +
      "or tree impostors.",
    "To compile as well, tick the steps (geometry, details, AI map) and pick the game data the compilers read, and " +
      "the SDK's `editors` directory for the geometry or the AI map.",
    "`Build` reports every file written and what it replaced, and lists any part it could not make; with steps " +
      "ticked, `Build and compile` also reports each step's time.",
  ],
  nuances: [
    "Without the game materials only the game files are written, as for a scene of no geometry.",
    "Proof lighting writes 3 lightmap pixels a metre, 1 jitter sample and quality 1 sky and sun into `build.prj`, " +
      "whatever the scene sets: xrLC lights in minutes rather than hours, for checking a level, not shipping it.",
    "The impostor atlas grows to 4096 a side by default, as Anomaly's largest levels ship it; the SDK atlas limit " +
      "keeps SDK 0.7's 2048 and refuses a level of more than 128 impostors.",
    "Files the build writes that the directory already holds are replaced only when overwriting is chosen, and a file " +
      "of the build's own it would not write again is refused, since the SDK's Build would move it aside.",
    "The steps are checked before any starts: each step's inputs, and a texture description (`.thm`) for every " +
      "texture `build.prj` names. Stopping a step kills its compiler and keeps what was built.",
    "The details are lit by the tools themselves, as the SDK's xrDO_Light lights them: on the GPU chosen in the " +
      "renderer settings where it traces rays, seconds on a small level and under a minute on `jupiter` as shipped; " +
      "on the CPU otherwise, or when chosen, the reference, every slot within one step of the GPU's. An existing " +
      "`level.details` is replaced only when overwriting is chosen.",
  ],
  limitations: [
    "The geometry and AI map steps run the SDK's own xrLC and xrAI in a work folder of their own, so they need " +
      "SDK 0.7 and take its time; they run at idle priority, so they slow down while the machine is busy.",
    "The objects roots are typed rather than picked.",
  ],
  relatedTools: [EToolId.LEVEL_DECOMPILER, EToolId.LEVEL_VIEWER],
};
