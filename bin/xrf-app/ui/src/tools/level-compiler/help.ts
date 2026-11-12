import { EToolId, IToolHelp } from "@/core/routing/tool";

export const LEVEL_COMPILER_HELP: IToolHelp = {
  summary:
    "Builds an SDK Level Editor scene into a level directory as the Level Editor's Build does: the game files " +
    "(`level.spawn`, `level.game`, particles, sounds and the rest) always, and, given the objects roots and game " +
    "materials, the `build.prj` the SDK's xrLC compiles with `level.ltx`, the occluders, details, the AI map, the tree " +
    "impostors and the breakable parts. It is what `xrf-cli level build` does; the lighting compile is still xrLC's.",
  workflow: [
    "Pick the scene's `.level`; its parts are read from the directory beside it.",
    "Pick the level directory to write into.",
    "Add the objects roots the scene's objects are read from, such as the scene's own `rawdata\\objects` and the SDK's.",
    "Pick the game materials, `gamemtl.xr`, to write `build.prj`, and the textures root where the scene holds details " +
      "or tree impostors.",
    "`Build` reports every file written and what it replaced, and lists any part it could not make.",
  ],
  nuances: [
    "Without the game materials only the game files are written, as for a scene of no geometry.",
    "Proof lighting writes 3 lightmap pixels a metre, 1 jitter sample and quality 1 sky and sun into `build.prj`, " +
      "whatever the scene sets: xrLC lights in minutes rather than hours, for checking a level, not shipping it.",
    "The impostor atlas grows to 4096 a side by default, as Anomaly's largest levels ship it; the SDK atlas limit " +
      "keeps SDK 0.7's 2048 and refuses a level of more than 128 impostors.",
    "Files the build writes that the directory already holds are replaced only when overwriting is chosen, and a file " +
      "of the build's own it would not write again is refused, since the SDK's Build would move it aside.",
  ],
  limitations: [
    "Compiling `build.prj` into geometry, lightmaps and collision, lighting details and compiling the AI map are " +
      "still the SDK's xrLC, xrDO_Light and xrAI.",
    "The objects roots are typed rather than picked.",
  ],
  relatedTools: [EToolId.LEVEL_DECOMPILER, EToolId.LEVEL_VIEWER],
};
