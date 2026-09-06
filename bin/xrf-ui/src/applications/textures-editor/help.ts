import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const TEXTURES_EDITOR_HELP: IApplicationHelp = {
  summary:
    "Editor for the `thm` descriptor beside a texture: what type it is, which bump pair and detail texture it " +
    "declares, and the authoring parameters the converter reads. Opens the same tree the explorer does, and writes " +
    "only when you save.",
  workflow: [
    "Pick a mode: `Folder` opens a root to work through, `Texture` opens one loose `.dds` or the `.thm` beside it.",
    "Choose a texture in the tree. The `Descriptor` panel binds to its `.thm`, or offers a new one where the texture " +
      "has none.",
    "Change what you came to change. `Primary` holds what the engine reads at load time; `Authoring metadata` holds " +
      "everything only the converter reads, which is most of a descriptor.",
    "`Save` writes the file. `Discard` puts the form back to what is on disk.",
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
  ],
  limitations: [
    "A texture served out of an archive can be read here but not written; its rows say so. Extract it first.",
    "The descriptor only. Re-encoding a texture, generating a bump pair and building one from a source image are " +
      "backed by the tools already but not yet on this screen.",
    "No undo beyond `Discard`, which returns the whole form to what the file says. Field-level undo is not built.",
    "Switching to another texture with unsaved changes discards them without asking. The prompt is not built yet.",
    "The `bump` and `detail` names are not checked against the roots yet, so a name that resolves to nothing is " +
      "written as typed.",
  ],
  relatedTools: [EApplicationId.TEXTURES_EXPLORER, EApplicationId.VISUALS_EXPLORER],
};
