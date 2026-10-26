import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const TEXTURES_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Read-only browser for game textures and the `thm` descriptors beside them: what each declares, which bump pair " +
    "the engine binds for it, and what the files behind that pair actually are. Browse a whole game, a gamedata " +
    "tree, or a directory of loose files - archive volumes included - or open one texture. Nothing is ever written.",
  workflow: [
    "Pick a mode: `Game` lists the game the way the engine mounts it - its archive volumes and the loose tree in " +
      "front of them - `Gamedata` lists every texture under a gamedata tree, `Directory` lists the `.dds` files of " +
      "any directory by their own paths, and `Texture` opens one loose `.dds` or the `.thm` beside it.",
    "Narrow the tree with the filter chips above it - `Degraded`, `Unreadable`, `Skipped type` and the rest - or " +
      "type into the filter field to search every name.",
    "Open a texture with a double click in the tree, `Enter`, or the filter. One click only selects, as in every " +
      "tree here.",
    "Read the `Material` panel for what the descriptor declares, the `Files` panel for the three files behind it - " +
      "the texture, the bump, and its `bump#` companion - and the `Channels` panel for the two bump planes and the " +
      "three values the engine reads out of them.",
    "Turn `Lit surface` on in the toolbar to see what the engine makes of the pair. `Body` chooses what the texture " +
      "is laid on, how many times it repeats and how its alpha is read; `Lighting` sets the light, drag to orbit, " +
      "hold `Shift` and drag to move the light, and turn `Bump` off to compare the same body flat.",
    "A texture carries no answer about its own alpha, so `Body` offers the engine's three. `Ignored` is the plain " +
      "deferred shader, which takes three components of `tbase` and never samples the fourth - a texel marked " +
      "transparent is drawn in whatever colour sits under it, which for a DXT1 block in one bit alpha mode is " +
      "black. `Cut out` is every `_aref` shader: `clip(D.w - def_aref)`, a hard cut at `200 / 255`, still in the " +
      "opaque pass and never blended. `Blended` is the forward path, for the blenders that leave the base pass. " +
      "Which one applies to a file is decided by the blender of whatever surface binds it, and the same texture is " +
      "cut out on a fence and ignored on a crate.",
  ],
  nuances: [
    "A row is placed where its file is, at the logical path the engine holds it under, so the tree starts at " +
      "`textures\\` the way every other tree here starts at its own directory. The `.dds` and the `.thm` of one " +
      "texture are still one row rather than two, because the engine binds both by one reference - the path below " +
      "`textures\\` without the extension, which is what a mesh declares and what the `Material` panel names.",
    "A declared bump pair is folded under the texture declaring it rather than listed beside it: `wall_bump` and " +
      "`wall_bump#` are packed planes belonging to `wall`, not pictures of their own.",
    "A file named like a bump half that no descriptor declares stays a row of its own and is marked `Unreferenced`. " +
      "The engine will never bind it, which is worth finding.",
    "The rows arrive before the marks do. Listing a root is a walk of an index, while reading every descriptor in it " +
      "takes seconds on a large installation, so the tree is browsable first and gains its marks and its filter " +
      "counts when that sweep lands.",
    "The marks beside a name are dots rather than words to keep a row one line; hovering one names it. The filter " +
      "chips above carry the same names and say how many textures each would show.",
    "`Degraded` is the mark worth hunting: the engine takes the bump shader path and binds its flat dummy for a " +
      "half that is missing, so the surface renders flat in the game while still paying for the bump.",
    "The type a `.thm` declares gates the whole descriptor. A `Bump Map` or `Cube Map` typed descriptor is skipped " +
      "by the engine however complete its bump declaration, and is marked `Skipped type` rather than reported as " +
      "bumped.",
    "`Height` on a bumped material is the authored virtual height, which the renderer never reads; parallax depth " +
      "comes from the `r2_parallax_h` console variable.",
    "Whatever `Also search in` names is searched behind the root being browsed and folds into the same listing, so a " +
      "mod tree carrying only what it changed still resolves - and reports - the bumps it did not.",
    "Textures the engine loads but no reference names, such as a level's lightmaps, are counted in the status bar " +
      "rather than listed: they sit outside `textures\\`, so no descriptor can describe them.",
    "The `Channels` panel draws through the same decode the lit surface is shaded by, on small unlit quads: a " +
      "surface that looks wrong is traced back to the plane it came from without a second implementation to " +
      "disagree with. The two raw tiles are the files as uploaded, so a wrong plane and a wrong decode are told " +
      "apart. Nothing there is colour-managed; the numbers are drawn as they are read.",
    "The lit surface is the same shading the visuals viewer uses, on a generated body rather than a mesh: one " +
      "shader kernel decodes `normal.gloss` and `error.height` for both, so a texture judged here and the model " +
      "binding it cannot disagree. The body's tangent basis is derived from its own uvs, with `v` running downwards " +
      "as X-Ray stores its rows.",
    "Turning `Light` off draws the body under a flat ambient, which is the same picture the flat preview shows. " +
      "That is what makes the two modes comparable: with the light on you are looking at an interpretation, and the " +
      "switch says which of the two you have. Nothing is shaded with the light off, so the `Bump` toggle goes with " +
      "it.",
    "`Plane` is a thin slab rather than a true plane, so orbiting past its edge shows a dark side rather than an " +
      "empty viewport. Its proportions are the texture's own, and only its front face carries the texture.",
    "The session survives a reload; the texture that was open does not, because the backend parks no selection. " +
      "Leaving the application closes the browsed roots.",
  ],
  limitations: [
    "Read-only: no editing, saving, or export. Changing what a descriptor declares is the textures editor, which " +
      "opens the same tree and writes only when you save.",
    "The channel tiles have no texel readout. Hovering one names no numbers, because a bump pair the SDK wrote is " +
      "always DXT5 and no cpu copy of a compressed plane exists to read.",
    "A `.dds` layout the backend cannot decode shows no picture; its descriptor and its files are still reported.",
    "Bump declarations are read from `.thm` files only. A `textures.ltx` beside the textures declares bumps and " +
      "detail associations too, and is not read; a notice names it when the browsed roots hold one.",
    "`Texture` mode accepts only loose files on disk. A texture inside an archive is reached by browsing, through " +
      "`Game` or `Gamedata` mode.",
    "A single file is opened by its engine reference, which is its path below a `textures` directory. The file " +
      "therefore has to sit under one; a `.dds` on a desktop belongs to no tree and names no texture. A tree holding " +
      "only textures is enough, and whatever `Also search in` names is still searched behind it for the bump pair.",
  ],
  relatedTools: [EApplicationId.TEXTURES_EDITOR, EApplicationId.VISUALS_EXPLORER, EApplicationId.ARCHIVES_EXPLORER],
};
