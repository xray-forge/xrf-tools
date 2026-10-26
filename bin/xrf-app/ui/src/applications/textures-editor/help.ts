import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const TEXTURES_EDITOR_HELP: IApplicationHelp = {
  summary:
    "Edit a texture and its `thm` descriptor, compare encoding formats, preview a chosen conversion and generate " +
    "its bump pair. Descriptor changes and a chosen encoding are written on Save; bump generation writes its " +
    "output files immediately.",
  workflow: [
    "Open one `.dds` texture or its `.thm` descriptor. Use `Also search in` when referenced assets live elsewhere.",
    "Edit the `Descriptor` panel, which offers a new descriptor when the texture has none.",
    "Change what you came to change. `Primary` holds what the engine reads at load time; `Authoring metadata` holds " +
      "everything only the converter reads, which is most of a descriptor.",
    "In `Formats`, use `Weigh formats` to compare candidates, then choose an encoding to preview it beside the original.",
    "Use `Bump` to generate a bump pair from a height map and optional normal and gloss inputs.",
    "`Save` writes the descriptor and any chosen encoding. `Discard` resets the draft and clears the encoding choice.",
  ],
  nuances: [
    "A texture with no `.thm` is the normal case, not an error: most textures have none. Saving one authors it, " +
      "starting from the defaults the SDK begins a new descriptor at.",
    "The form sends the fields you can edit, never the whole file. A save applies them onto the descriptor as it is " +
      "on disk, so the thumbnail and any chunk this reader does not model survive untouched - a descriptor read and " +
      "written back is byte for byte the file it was.",
    "A value no version of the SDK ever named is shown as `Unknown` with its number and can be saved back as it was. " +
      "Dropping it to the first entry of a list is how an editor quietly corrupts somebody else's descriptor.",
    "A flag toggle changes one bit. Bits the SDK never named, which another tool may have written, are left exactly " +
      "as they were.",
    "Width and height are read from the `dds` header and shown read-only. The descriptor carries its own copy, which " +
      "a converter refreshed and a hand-edited file may have wrong; the save copies the header's in.",
    "The save refuses a file that changed on disk since it was read, naming it and asking for a reload. Each file is " +
      "written beside its target and renamed over it, so a failure leaves the previous file rather than a partial one.",
    "Saving is a job: it survives a reload of the window, and the notification centre records what was written.",
    "Comparing formats and choosing a preview do not replace the texture. A chosen encoding is held until Save.",
    "Bump generation writes the `_bump` and `_bump#` files beside the texture, then updates the descriptor draft. " +
      "Save the draft to persist that reference; Discard does not undo the generated files.",
  ],
  limitations: [
    "A texture served out of an archive can be read here but not written; its rows say so. Extract it first.",
    // todo: Add source-image import, batch conversion, cubemap authoring and texture synchronization to this editor.
    "Source-image import for the base texture, batch conversion, cubemap authoring and texture synchronization are " +
      "planned extensions of this editor.",
    "No undo beyond `Discard`, which returns the whole form to what the file says. Field-level undo is not built.",
    "The `bump` and `detail` names are not checked against the roots yet, so a name that resolves to nothing is " +
      "written as typed.",
  ],
  relatedTools: [EApplicationId.TEXTURES_EXPLORER, EApplicationId.VISUALS_EXPLORER],
};
