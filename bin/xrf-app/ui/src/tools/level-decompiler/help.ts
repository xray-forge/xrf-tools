import { EToolId, IToolHelp } from "@/core/routing/tool";

export const LEVEL_DECOMPILER_HELP: IToolHelp = {
  summary:
    "Turns a compiled level back into an SDK Level Editor scene: the `.level` and its parts, one `.object` per " +
    "sector of static geometry with its sectors and portals, the trees as multi-use objects with their impostors, " +
    "the occluders, ladders and breakables, lights, glows, details, the AI map, and every entity. A sidecar beside the " +
    "scene records where each piece came from and what was inferred. It is what `xrf-cli level decompile` does.",
  workflow: [
    "Pick the compiled level's directory, `gamedata\\levels\\<level>`.",
    "Pick the scene file to write, `rawdata\\levels\\<level>.level`; its name is the level's.",
    "Leave the objects and textures roots empty to write beside the scene as the SDK lays rawdata out, or name them.",
    "Add an object library, such as the SDK's `rawdata\\objects`, to dress surfaces and find detail models by it.",
    "`Decompile` reports what it wrote and lists what the scene could not hold, with why.",
  ],
  nuances: [
    "Surfaces are inferred by default: each face takes the game material of the collision face it lies on and a " +
      "compiler shader fitting what the level shows. Defaults give every surface the editor's `default`.",
    "The game materials and compiler shaders come from the level's own gamedata (`gamemtl.xr`, `shaders_xrlc.xr`).",
    "Entities come from the root spawn `all.spawn` where the level's gamedata holds one, else from `level.spawn`.",
    "Decompiling the same level twice gives the same scene, down to its GUID.",
  ],
  limitations: [
    "An existing scene, its objects and textures are refused unless overwriting is chosen.",
    "Placed objects are not recovered from the merged geometry: each sector comes back as one shell.",
    "Detail models no library object draws are left out, since the level keeps their place in the atlas, not their " +
      "texture.",
    "Spawn packets of Clear Sky's and older versions are left out one by one.",
  ],
  relatedTools: [EToolId.LEVEL_COMPILER, EToolId.LEVEL_VIEWER],
};
