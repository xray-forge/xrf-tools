import { EToolId, IToolHelp } from "@/core/routing/tool";

export const LEVEL_EDITOR_HELP: IToolHelp = {
  summary:
    "Opens an SDK Level Editor scene - the `.level` and the part files beside it, as the SDK or the level " +
    "decompiler wrote them - draws its objects, and shows what it holds: every tool with its objects, each object's " +
    "place and properties, and what the SDK's editor would skip of its parts. It is read-only: it neither edits nor " +
    "saves the scene.",
  workflow: [
    "Pick the scene's `.level`, `rawdata\\levels\\<level>.level`; its parts are read from the directory of its name " +
      "beside it.",
    "Leave the objects roots empty to look for library objects in `objects` beside the scene's directory, or name " +
      "them in search order.",
    "Leave the game roots empty to read textures and shaders from `gamedata` beside the scene's `rawdata`, or name " +
      "the installation or gamedata directories that hold them in search order.",
    "The viewport draws every scene object as the library object it places, compiled from its `.object`, and frames " +
      "the camera on them as the scene opens, or on the helpers where the scene has none; fly it as in the level " +
      "viewer, its speed and lens under Camera.",
    "A spawn point whose entity names a visual found in the game data stands as that model, posed as the level " +
      "viewer poses it, and keeps any volume attached to it.",
    "What has no geometry is drawn as a helper: spawn points by their volume or an icon, ways by their arrowed links " +
      "and points, shapes, sound environments and fog volumes by their volumes, glows by their radius, portals by " +
      "their outline, and lights, sound sources and particles by an icon. Show switches them all, or tool by tool, " +
      "and counts each tool's.",
    "Surfaces the game never draws - those whose compiler shader keeps them for collision or shadows alone, such as " +
      "a tree's collision hull or a level's invisible walls - are left out until Show's Collision surfaces draws them.",
    "Surfaces dressed in the SDK's marker textures (prop\\prop_fake_*), which only show the editor what a level holds " +
      "unseen, such as a sun blocker or an occluder's sheet, are left out until Show's Editor-only textures draws them.",
    "The Outliner lists the tools in the SDK's order, each with its objects; filter by an object's name, the library " +
      "object or effect it places, or its entity's section, and pick one to show it in the Inspector.",
    "Click an object in the viewport to select it, Alt held to pick what stands behind glass: the Outliner opens its " +
      "tool and shows its row, and the selection is outlined, a helper lifted. The helper under the pointer is lifted " +
      "while helpers are shown.",
    "Double-click an Outliner row, or press Enter on it, to select its object and frame the camera on it; F frames " +
      "the selection. The eye on a row hides its object until shown again or another scene opens; Reveal hidden " +
      "objects shows them all.",
    "The Scene panel shows the `.level` itself: its GUID, the roots it reads from, build parameters, camera and every " +
      "tool's part files.",
    "Build opens the level compiler holding the scene, its level directory `gamedata\\levels\\<level>` beside the " +
      "scene's `rawdata`, the textures its decompile wrote there, and the game data the scene is drawn with other than " +
      "that one; tick the steps there and build.",
  ],
  nuances: [
    "The scene is lit by a fixed sun and an even ambient light, not by a weather or baked light: it shows shape and " +
      "placement, not the final look.",
    "A scene object whose library object is missing, does not read or does not compile is not drawn and is listed in " +
      "Problems. A texture the game data lacks draws a placeholder and is listed there too; what does not compile and " +
      "the textures are found once the scene has loaded, as in the level viewer.",
    "A scene that draws nothing - one holding no objects, none whose library object was found, or only objects that " +
      "draw neither a model nor a helper - says which in the viewport.",
    "A spawn point's icon says what is known without the game's configs: the actor's start, a respawn point, or any " +
      "other entity. A sound source's spheres are not drawn: a scene keeps the distances its sound's file last gave.",
    "A part the SDK's editor would skip - one of another scene's GUID, of another tool version or class, or one that " +
      "does not read - is listed in Problems and marked on its tool in the Outliner; the rest of the scene opens.",
    "The AI map, detail objects and wall marks are counted, not read into the page.",
    "A group's members are listed under their own tools, each naming its group.",
    "The scene is read again when its files change on disk, and drawn again when its `.level` or part files do.",
  ],
  limitations: [
    "Nothing is edited or written: the scene, its objects and its parts stay as they are on disk.",
    "Sectors, groups, the AI map, detail objects and wall marks are not drawn, so they are neither picked in the " +
      "viewport nor framed. One object is selected at a time.",
  ],
  relatedTools: [EToolId.LEVEL_DECOMPILER, EToolId.LEVEL_COMPILER],
};
