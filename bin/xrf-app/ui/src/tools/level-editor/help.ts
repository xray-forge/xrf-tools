import { EToolId, IToolHelp } from "@/core/routing/tool";

export const LEVEL_EDITOR_HELP: IToolHelp = {
  summary:
    "Opens an SDK Level Editor scene - the `.level` and the part files beside it, as the SDK or the level " +
    "decompiler wrote them - and shows what it holds: every tool with its objects, each object's place and properties, " +
    "and what the SDK's editor would skip of its parts. It is read-only: it neither edits nor saves the scene, and it " +
    "does not draw it yet.",
  workflow: [
    "Pick the scene's `.level`, `rawdata\\levels\\<level>.level`; its parts are read from the directory of its name " +
      "beside it.",
    "Leave the objects roots empty to look for library objects in `objects` beside the scene's directory, or name " +
      "them in search order.",
    "The Outliner lists the tools in the SDK's order, each with its objects; filter by an object's name, the library " +
      "object or effect it places, or its entity's section, and pick one to show it in the Inspector.",
    "The Scene panel shows the `.level` itself: its GUID, build parameters, camera and every tool's part files.",
  ],
  nuances: [
    "A part the SDK's editor would skip - one of another scene's GUID, of another tool version or class, or one that " +
      "does not read - is listed in Problems and marked on its tool in the Outliner; the rest of the scene opens.",
    "The AI map, detail objects and wall marks are counted, not read into the page.",
    "A group's members are listed under their own tools, each naming its group.",
    "The scene is read again when its files change on disk.",
  ],
  limitations: [
    "Nothing is edited or written: the scene, its objects and its parts stay as they are on disk.",
    "The scene is not drawn yet: the viewport stands empty, and objects are chosen from the Outliner.",
  ],
  relatedTools: [EToolId.LEVEL_DECOMPILER, EToolId.LEVEL_COMPILER],
};
