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
    "Inspect the Header, Materials, Bones, and Motions panels; toggle wireframe, UV checkerboard, bump " +
      "shading, or the skeleton overlay.",
    "If the model carries motions, open the `Motions` panel: names are grouped into families you can open, " +
      "or filter them. Double click a name, or press `Enter` on it, to pose it, then play, scrub, or step it.",
  ],
  nuances: [
    "The camera fits the first model shown and then holds position across model switches; `Reset camera` " +
      "refits it to the current model.",
    "The mesh detail slider walks each submesh's progressive edge-collapse chain - it is view quality, not " +
      "authored LODs - and the chosen fraction survives model switches.",
    "Playback lives in the `Motions` panel rather than under the viewport, so what a model turns out to " +
      "contain never resizes the preview. A motion keeps playing while another panel is open.",
    "Motion names are listed the first time the `Motions` panel is shown for a model, since naming them means " +
      "reading every referenced animation file. A character can name thousands, so they are grouped by the token " +
      "each name starts with - `norm`, `cr`, `loophole`, `animpoint` - which is a convention these files follow " +
      "rather than anything the format states; a set named some other way lands in fewer, larger families. " +
      "Filtering searches every name and opens whatever it matched.",
    "One click selects a motion and does nothing else, as in every tree here; posing reads and bakes the motion, " +
      "so it waits for a double click or `Enter`. The posed motion is marked in the list.",
    "Picking another motion carries the play/pause state over; dragging the frame slider pauses. Motions are " +
      "baked one at a time by the backend, so switching is serialized.",
    "The duration beside the frame counter is the time the engine takes: the motion's frames at 30 fps over the " +
      "playback speed its file declares, so a motion authored at speed `1.2` reports less time than its frames " +
      "span. The `Playback rate` control is separate - it changes how fast the motion is being looked at and " +
      "never what is reported.",
    "Textures the renderer can read directly (`DXT1/3/5`, `ETC1`, `BC6H`, uncompressed BGRA/BGR) upload as-is; " +
      "other layouts are decoded by the backend to PNG and upload without a mip chain, which the Materials " +
      "panel reports - such textures shimmer at distance.",
    "A texture shipped without mips uses linear filtering deliberately; sampling its incomplete mip chain " +
      "would render black.",
    "The Materials panel reports the bump material the game builds for each texture from its `.thm`: the " +
      "declaration, the bump and `bump#` files it binds, and whether either fell back to the engine's flat dummy " +
      "or its missing-texture placeholder. A `Dummy bump` or `Bump missing` chip is a surface that renders flat " +
      "in the game while paying for the bump shader.",
    "A bumped material is shaded the way the game's R2/R3 deferred renderer shades it in its HQ variants: the " +
      "viewer samples the same `bump` and `bump#` pair through the same decode, rotates the result through the " +
      "authored tangent basis, and shows gloss as specular response only. Lighting stays the viewer's own. The " +
      "`Bump` toggle draws the same surface flat for comparison; on a dummy pair the two look alike, which is the " +
      "point. Parallax (`r2_parallax_h`, `0.02` in the game) and detail textures are not drawn, and the Materials " +
      "panel says so on the materials that ask for them.",
    "The type a `.thm` declares gates the whole descriptor: a `Bump Map` or `Cube Map` typed descriptor is skipped " +
      "by the engine however complete its bump declaration, and the panel says so rather than reporting a bump.",
    "`Height` on a bumped material is the authored virtual height, which the renderer never reads; parallax depth " +
      "comes from the `r2_parallax_h` console variable.",
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
    "Parallax and detail materials shade as plain bump; a bump in a layout the renderer cannot upload is drawn " +
      "flat and reported, since the png fallback would lose the mip chain a bump relies on.",
    "Bump declarations are read from `.thm` files only. A `textures.ltx` beside the textures declares bumps and " +
      "detail associations too, and is not read; the Materials panel says so when the searched roots hold one.",
    "`Model` mode accepts only loose files on disk; a model inside an archive is reached through `Folder` " +
      "mode, and `Browse folder` is unavailable for archive-sourced models.",
  ],
  relatedTools: [EApplicationId.VISUALS_SEQUENCER, EApplicationId.ARCHIVES_EXPLORER],
};
