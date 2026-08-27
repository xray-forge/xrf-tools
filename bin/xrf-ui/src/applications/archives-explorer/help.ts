import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Read-only browser for packed game archives (`.db*` / `.xdb*`). Open a directory to browse every volume " +
    "merged into one virtual file tree, or open a single volume on its own. Files can be previewed in place " +
    "and selectively extracted to disk; archives themselves are never modified.",
  workflow: [
    "Pick a mode: `Directory` indexes every archive found recursively under the chosen folder, " +
      "`Archive` opens one volume.",
    "Browse or filter the file tree; selecting a file shows its preview and metadata, selecting a directory " +
      "shows its recursive size summary.",
    "Extract a single file from the preview header, or a whole directory (or the archive root) from the " +
      "directory view.",
  ],
  nuances: [
    "Directory mode merges all volumes into one name table the way the engine does: later volumes override " +
      "earlier entries, and volumes under a directory component named exactly `patches` sort last, so patch " +
      "content wins over everything.",
    "Text is decoded as Windows-1251, so Cyrillic configs read correctly.",
    "Previewable types: engine text formats (`ltx`, `script`, `xml`, shader sources, and similar), `dds` images " +
      "(decoded to PNG; the caption still reports the source format and mip count), `ogg` audio (including the " +
      "X-Ray engine parameters stored in the vorbis comment), and `ogf` models in a 3D viewport.",
    "An `ogg` without a recognized X-Ray comment still plays; the panel notes the engine would fall back to " +
      "built-in source defaults.",
    "Directory extraction strips the selected prefix: extracting `configs\\gameplay` writes its contents " +
      "directly into the chosen folder. Extract the archive root to preserve the full layout.",
    "Each open mode remembers its own last path; switching modes returns each field to where it last pointed.",
    "Compressed entries are decompressed transparently and verified against their CRC32 - a mismatch is an " +
      "error, not silent corruption.",
    "Navigation is locked while an extraction runs, and a new file selection is ignored while a read is still " +
      "in flight.",
  ],
  limitations: [
    "Strictly read-only: no editing, repacking, or writing into archives.",
    "Preview size limits: 10 MB for text, 32 MB for `dds`, 64 MB for `ogg`. Larger files, and binary formats " +
      "outside the list above, still show their metadata in Details.",
    "Extraction overwrites existing files at the destination without asking.",
    "Directory extraction has no progress bar and cannot be cancelled.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_UNPACKER, EApplicationId.ARCHIVES_PACKER],
};
