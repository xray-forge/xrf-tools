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
    "Open `Statistics` in the toolbar for a breakdown of what is open: by extension, by folder, by size, and where " +
      "its files come from.",
  ],
  nuances: [
    "Directory mode follows the engine's mount order: later volumes replace earlier copies. Volumes in a folder named " +
      "`patches` are loaded last, so their files win.",
    "Directory and Archive modes show archives only. A loose `gamedata` tree beside them is not listed, even though " +
      "the engine would load it instead - use `Installation` mode to see that arrangement.",
    "Installation mode lists one row per engine path: the copy the engine would open. Its icon is tinted by where " +
      "the bytes come from - loose on disk, or inside a volume - and hovering it says which. `File details` names " +
      "where that copy sits and which copies it hides, and the status bar counts how many paths are answered more " +
      "than once.",
    "Installation mode reads `fsgame.ltx` when the folder declares one, and otherwise treats the folder as a game " +
      "data tree. Previews and extraction resolve the same copy the tree shows.",
    "Archive paths are case-insensitive. `Textures\\A.DDS` and `textures\\a.dds` name the same file to the engine. " +
      "When that hides an entry, the `Unreachable files` panel shows it.",
    "An entry another mount overrides is not the same problem: that file is exactly where it should be, and the " +
      "`Unreachable files` panel deliberately does not list it.",
    "`Statistics` reports both a file count and a byte total for every breakdown, because the two rank differently: " +
      "`ogg` is usually the largest group by count and a small one by size. The `Bytes` / `Count` toggle chooses " +
      "which one orders the rows and draws the bars, so the list never contradicts what it shows.",
    "`Origins` and `Overrides` keep their own order whichever measurement is chosen, because sources read by " +
      "priority: the row above is the one that wins.",
    "The `Extensions` section lists spellings exactly as they appear on disk and marks any the tools do not " +
      "recognise. That is a finding worth reporting rather than a fault - it usually means a real format the tooling " +
      "has not been taught yet.",
    "`Statistics` offers only the sections the open subject can answer. A volume set gets `Compression` and " +
      "`Volumes`; a game folder gets `Origins` and `Overrides` instead, because a merged name table cannot say what " +
      "it folded away and a loose file has no stored size.",
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
