import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const VISUALS_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Read-only 3D browser for `ogf` models: meshes, materials and textures, skeleton, and motion playback. " +
    "Browse a whole root - archive volumes included - or open one loose file. Nothing is ever written.",
  workflow: [
    "Pick a mode: `Folder` lists every visual under a root (such as `gamedata` or a `meshes` directory), " +
      "`Model` opens one loose `.ogf`.",
    "Open a model with a double click in the tree, `Enter`, or the filter; the viewport shows its bind pose. " +
      "A model that fails to open states why and offers `Retry`; one click only selects, so its row stays " +
      "selected too.",
    "Inspect the Header, Materials, Bones, and Motions panels; toggle wireframe, UV checkerboard, or the " +
      "skeleton overlay.",
    "If the model carries motions, pick one in the animation bar and play, scrub, or step it.",
  ],
  nuances: [
    "The camera fits the first model shown and then holds position across model switches; `Reset camera` " +
      "refits it to the current model.",
    "The mesh detail slider walks each submesh's progressive edge-collapse chain - it is view quality, not " +
      "authored LODs - and the chosen fraction survives model switches.",
    "Motion names are listed on demand when a model lands, since naming them means reading every referenced " +
      "animation file; the picker is an autocomplete because a character can reference thousands.",
    "Picking another motion carries the play/pause state over; dragging the frame slider pauses; clearing the " +
      "motion restores the bind pose. Motions are baked one at a time by the backend, so switching is serialized.",
    "The duration beside the frame counter is the time the engine takes: the motion's frames at 30 fps over the " +
      "playback speed its file declares, so a motion authored at speed `1.2` reports less time than its frames " +
      "span. The `Playback rate` control is separate - it changes how fast the motion is being looked at and " +
      "never what is reported.",
    "Textures the renderer can read directly (`DXT1/3/5`, `ETC1`, `BC6H`, uncompressed BGRA/BGR) upload as-is; " +
      "other layouts are decoded by the backend to PNG and upload without a mip chain, which the Materials " +
      "panel reports - such textures shimmer at distance.",
    "A texture shipped without mips uses linear filtering deliberately; sampling its incomplete mip chain " +
      "would render black.",
    "Hiding a bone hides everything parented to it, the way the engine collapses an unattached addon; the " +
      "panel offers dedicated switches for addon bones such as `wpn_scope`. Bones past the engine's 64-bone " +
      "visibility mask are flagged: hiding them is viewer-only state.",
    "A submesh the format cannot express is skipped with a stated reason in Materials; the rest of the model " +
      "still renders.",
    "References the browsed tree cannot answer are looked for in the configured game data and then in the game " +
      "installation, so a mod tree carrying only what it changed still resolves the textures it did not.",
    "The session survives a reload; leaving the application closes the backend selection.",
  ],
  limitations: [
    "Read-only: no editing, saving, or export.",
    "Only OGF format version 4 is supported.",
    "At most 4 skin links per vertex; unknown vertex layouts skip that submesh.",
    "Authored LOD reference geometry is parsed but not drawn.",
    "Cubemap `dds` textures are refused rather than guessed at.",
    "`Model` mode accepts only loose files on disk; a model inside an archive is reached through `Folder` " +
      "mode, and `Browse folder` is unavailable for archive-sourced models.",
  ],
  relatedTools: [EApplicationId.VISUALS_SEQUENCER, EApplicationId.ARCHIVES_EXPLORER],
};
