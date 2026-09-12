import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Browse game files without changing them. Open a folder of `.db*` / `.xdb*` volumes as one file tree, inspect " +
    "one volume, or open a whole game folder and see the files the engine would actually load. Preview supported " +
    "files and extract only what you need.",
  workflow: [
    "Choose `Directory` to load every archive below a folder, `Archive` to inspect one volume, or `Installation` to " +
      "open a game folder as the engine mounts it.",
    "Browse or filter the file tree. Select an entry, then double-click it or press `Enter` to open it.",
    "Use `Extract file` for one file. Select a directory to extract its contents, or select the tree root to " +
      "extract everything.",
  ],
  nuances: [
    "Directory mode follows the engine's mount order: later volumes replace earlier copies. Volumes in a folder named " +
      "`patches` are loaded last, so their files win.",
    "Directory and Archive modes show archives only. A loose `gamedata` tree beside them is not listed, even though " +
      "the engine would load it instead - use `Installation` mode to see that arrangement.",
    "Installation mode lists one row per engine path: the copy the engine would open. `File details` names where " +
      "that copy sits and which copies it hides, and the status bar counts how many paths are answered more than once.",
    "Installation mode reads `fsgame.ltx` when the folder declares one, and otherwise treats the folder as a game " +
      "data tree. Previews and extraction resolve the same copy the tree shows.",
    "Archive paths are case-insensitive. `Textures\\A.DDS` and `textures\\a.dds` name the same file to the engine. " +
      "When that hides an entry, the `Unreachable files` panel shows it.",
    "An entry another mount overrides is not the same problem: that file is exactly where it should be, and the " +
      "`Unreachable files` panel deliberately does not list it.",
    "Text is read as Windows-1251, so Cyrillic configs remain readable.",
    "The explorer previews engine text, `dds` images, `ogg` audio, and `ogf` models. Other files still have a " +
      "Details entry.",
    "An `ogg` without X-Ray playback data still plays. The game would use its built-in source defaults.",
    "Extracting `configs\\gameplay` writes that folder's contents directly into the destination. Extract the tree " +
      "root to keep the full layout.",
    "Each open mode remembers its last path.",
    "Compressed entries are unpacked and checked against their CRC32 before they are shown or extracted.",
    "Opening another file abandons its unfinished preview and starts the new one. An extraction keeps running until it " +
      "finishes or you cancel it.",
  ],
  limitations: [
    "The explorer cannot edit or repack an archive.",
    "Installation mode has no volume metadata: a loose file has no offset, stored size or recorded CRC, and shared " +
      "payloads are a property of one volume set. Open the archives directly for those.",
    "Preview limits are 10 MB for text, 32 MB for `dds`, and 64 MB for `ogg`. Larger and unsupported files still " +
      "appear in Details.",
    "Extraction replaces existing destination files without asking.",
    "Directory extraction shows progress and can be cancelled between files. Files already written remain on disk.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_UNPACKER, EApplicationId.ARCHIVES_PACKER],
};
