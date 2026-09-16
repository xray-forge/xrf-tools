import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Browse game files without changing them. Open a whole game folder and see the files the engine would actually " +
    "load, open a folder of `.db*` / `.xdb*` volumes as one file tree, or inspect one volume. Preview supported " +
    "files and extract only what you need.",
  workflow: [
    "Choose `Game` to open a game folder as the engine mounts it, `Directory` to load every archive below a folder, " +
      "or `Archive` to inspect one volume.",
    "Browse or filter the file tree. Select an entry, then double-click it or press `Enter` to open it.",
    "Use `Extract file` for one file. Select a directory to extract its contents, or select the tree root to " +
      "extract everything.",
    "Open `Statistics` in the toolbar for a breakdown of what is open: by extension, by folder, by size, and where " +
      "its files come from.",
    "Open `Resolution` in the toolbar to see which sources are searched for a file and in what order, before opening " +
      "any file at all.",
  ],
  nuances: [
    "Directory mode follows the engine's mount order: later volumes replace earlier copies. Volumes in a folder named " +
      "`patches` are loaded last, so their files win.",
    "Directory and Archive modes show archives only. A loose `gamedata` tree beside them is not listed, even though " +
      "the engine would load it instead - use `Game` mode to see that arrangement.",
    "Game mode lists one row per engine path: the copy the engine would open. Its icon is tinted by where " +
      "the bytes come from - loose on disk, or inside a volume - and hovering it says which. `File details` names " +
      "where that copy sits and which copies it hides, and the status bar counts how many paths are answered more " +
      "than once.",
    "Game mode reads `fsgame.ltx` when the folder declares one, and otherwise treats the folder as a game " +
      "data tree. Previews and extraction resolve the same copy the tree shows.",
    "`Resolution` lists the sources top to bottom in the order they are searched: the first one holding an engine " +
      "path is the copy the engine loads. A game folder is its `fsgame.ltx` declarations reversed, because the engine " +
      "registers roots as declared and a later registration overwrites an earlier one - which is exactly why a loose " +
      "`gamedata` tree ends up in front of the volumes it overrides. Each row names the alias that declared it.",
    "A `Resolution` row for a volume set lists the volumes inside it, also in the order a lookup reaches them: a " +
      "set merges with the later volume winning, so a patch volume is asked before the one it patches.",
    "`Resolution` also lists sources that were declared and could not be opened. A source missing from the search " +
      "looks exactly like content that was never there, so a corrupt or unreadable volume is stated rather than " +
      "silently dropped.",
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
    "The explorer previews engine text, `dds` images, `ogg` audio, and `ogf` models. Engine text covers configs, " +
      "scripts and every shader stage, so a `gs` or an `hlsl` reads like the `vs` beside it.",
    "A binary file the explorer cannot draw is described in words instead, where a reader for its format exists: " +
      "`thm` descriptors, `omf` motion banks, `anm` object motions, `ppe` post-process effects, `particles.xr`, " +
      "`shaders.xr`, a compiled `level` and a spawn set. A description names the files it refers to, and a name the " +
      "open subject holds selects it in the tree.",
    "An `anm` is a camera or object path over six channels, and is described by what the engine would play: the " +
      "frame range counts both its ends, so a range of 0 to 59 runs for 60 frames. Where the keys reach past that " +
      "range, or stop short of it, the description says so - 101 of the shipped animations do one or the other.",
    "A `ppe` is a screen effect over eleven parameters, three of which are colours the engine assembles from a red, " +
      "a green and a blue envelope of their own. Every effect stores all of them and a shipped one keys three or " +
      "four, so the description says how many are keyed rather than leaving eleven mostly empty rows to be read. It " +
      "runs for as long as its longest parameter, which is not where its last key sits.",
    "A spawn set is summarised from its header alone: how many objects over how many levels, and what each section " +
      "of the file weighs. Its objects are not read - vanilla's set is 6,464 of them behind a game graph that is most " +
      "of a 29 MB file.",
    "A `level.cform` and a `level.ai` are summarised from their leading header: how many faces the collision mesh " +
      "holds, how many nodes the navigation grid does, and how much world each covers. Neither payload is read, " +
      "which is why a 191 MB collision mesh opens at all.",
    "A `level` is described through its shader table, which is what the level draws with. Its shader names are " +
      "looked up in the `shaders.xr` of whatever is open; with no library open they are shown unasked rather than " +
      "reported as missing.",
    "A description states what a file says and what the engine reads from it, and never whether either is wrong. " +
      "Checking that is what `gamedata verify` is for.",
    "A binary file no reader claims is still shown as the container it is: X-Ray files are trees of numbered chunks, " +
      "and that much is readable without knowing the format. Chunk ids are shown as numbers because naming one would " +
      "mean reading the format. A file that is not a container, or too large to read for its headers alone, has a " +
      "Details entry and nothing more.",
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
    "Game mode has no volume metadata: a loose file has no offset, stored size or recorded CRC, and shared " +
      "payloads are a property of one volume set. Open the archives directly for those.",
    "Preview limits are 10 MB for text, 32 MB for `dds`, and 64 MB for `ogg`. Larger and unsupported files still " +
      "appear in Details.",
    "Extraction replaces existing destination files without asking.",
    "Directory extraction shows progress and can be cancelled between files. Files already written remain on disk.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_UNPACKER, EApplicationId.ARCHIVES_PACKER],
};
