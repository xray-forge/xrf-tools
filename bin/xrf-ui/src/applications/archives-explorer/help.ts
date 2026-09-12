import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Browse packed game archives (`.db*` / `.xdb*`) without changing them. Open a folder of volumes as one " +
    "game file tree, or inspect one volume. Preview supported files and extract only what you need.",
  workflow: [
    "Choose `Directory` to load every archive below a folder, or `Archive` to inspect one volume.",
    "Browse or filter the file tree. Select an entry, then double-click it or press `Enter` to open it.",
    "Use `Extract file` for one file. Select a directory to extract its contents, or select the archive root to " +
      "extract the whole tree.",
  ],
  nuances: [
    "Directory mode follows the engine's mount order: later volumes replace earlier copies. Volumes in a folder named " +
      "`patches` are loaded last, so their files win.",
    "Archive paths are case-insensitive. `Textures\\A.DDS` and `textures\\a.dds` name the same file to the engine. " +
      "When that hides an entry, the `Unreachable files` panel shows it.",
    "Text is read as Windows-1251, so Cyrillic configs remain readable.",
    "The explorer previews engine text, `dds` images, `ogg` audio, and `ogf` models. Other files still have a " +
      "Details entry.",
    "An `ogg` without X-Ray playback data still plays. The game would use its built-in source defaults.",
    "Extracting `configs\\gameplay` writes that folder's contents directly into the destination. Extract the archive " +
      "root to keep the full archive layout.",
    "Each open mode remembers its last path.",
    "Compressed entries are unpacked and checked against their CRC32 before they are shown or extracted.",
    "Opening another file abandons its unfinished preview and starts the new one. An extraction keeps running until it " +
      "finishes or you cancel it.",
  ],
  limitations: [
    "The explorer cannot edit or repack an archive.",
    "Preview limits are 10 MB for text, 32 MB for `dds`, and 64 MB for `ogg`. Larger and unsupported files still " +
      "appear in Details.",
    "Extraction replaces existing destination files without asking.",
    "Directory extraction shows progress and can be cancelled between files. Files already written remain on disk.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_UNPACKER, EApplicationId.ARCHIVES_PACKER],
};
