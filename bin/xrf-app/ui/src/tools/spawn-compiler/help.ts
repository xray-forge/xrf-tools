import { EToolId, IToolHelp } from "@/core/routing/tool";

export const SPAWN_COMPILER_HELP: IToolHelp = {
  summary:
    "Builds a game's root spawn from its built levels, as SDK 0.7's `xrAI -s` does and as `xrf-cli spawn build` " +
    "does: each level's `level.ai`, `level.spawn` and `level.game` joined into one spawn holding the game graph, its " +
    "cross tables, every entity, the zones' artefact points and the patrols, written as `<name>.spawn`. The same " +
    "levels give the same file, byte for byte.",
  workflow: [
    "Pick the game data holding `configs\\game_levels.ltx`, which names each level's id, caption and offset.",
    "Pick the directory holding the built levels' directories, such as the `levels` a level compiler run writes " +
      "into; every level the game lists that is built there is offered.",
    "Tick the levels the spawn holds, choose the level a new game starts on, and name the spawn.",
    "Pick the directory the spawn is written into, then `Build`.",
    "The result counts the spawn's objects, artefact points and patrols, and lists what the build reported: graph " +
      "points left out, connections to levels not built, and restrictors cutting an AI map apart.",
  ],
  nuances: [
    "A level compiler run hands its level over: `Build spawn` in its result opens this tool holding the levels " +
      "directory the level was built into, the game data it was compiled with, and a `spawns` directory beside the " +
      "levels.",
    "What xrAI refuses is refused, naming the level and the entity: a level changer leading to a level not built or " +
      "to a graph point its level lacks, no actor or two, two entities of one story id, two patrols of one name. Tick " +
      "the level the changer leads to, or build each level the changers join.",
    "A start level other than the actor's own moves the actor to the first game vertex of that level a search " +
      "of the game graph reaches from its own.",
    "Stopping the job stops it between reading the levels, building the spawn and writing it; a spawn already " +
      "being written is finished.",
    "A spawn the directory already holds, the game's own `all.spawn` or any other, in any case of its name, is " +
      "written over only when overwriting is ticked; otherwise the build is refused before it reads a level.",
  ],
  limitations: ["Builds a whole spawn; patching one level inside an existing spawn is not offered."],
  relatedTools: [EToolId.LEVEL_COMPILER, EToolId.SPAWN_EDITOR],
};
