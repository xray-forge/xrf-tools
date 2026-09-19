import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_EXPLORER_HELP: IApplicationHelp = {
  summary: "Browse, preview and extract game files from a game folder, loose gamedata or `.db*` / `.xdb*` archives.",
  workflow: [
    "Choose `Game` for a game folder, `Gamedata` for loose game files, `Directory` for all archives below a " +
      "folder, or `Archive` for one volume.",
    "Browse or filter the file tree. Double-click a file or press `Enter` to preview it; open `File details` " +
      "to inspect its source and metadata.",
    "Use `Extract file` for one file, or select a directory to extract its contents. Select the tree root " +
      "to extract everything with the full folder layout.",
  ],
  nuances: [
    "`Gamedata` opens the selected directory as a loose file tree, with paths relative to that directory. " +
      "It does not follow `fsgame.ltx` or load files from packed archives.",
    "`Game` reads `fsgame.ltx` when present; otherwise it treats the folder as a gamedata tree. The tree, previews " +
      "and extraction use the copy the engine would load, including loose files that override archives.",
    "`Directory` and `Archive` show archives only, ignoring loose files. `Directory` loads volumes in path order; " +
      "later volumes replace earlier copies. Use `Game` for the installation's declared source priority.",
    "`Resolution` shows sources in search order: the first source containing a path wins. Check `Unread sources` " +
      "there if expected files are missing.",
    "`Overrides` shows duplicate paths with the winning copy first. Filter by path or source to find what a mod " +
      "replaces. `Unreachable copies` lists entries hidden by duplicate names within one source, including names " +
      "that differ only in letter case.",
    "`Statistics` breaks down file counts and sizes. Use `Bytes` / `Count` to change the ranking; source-priority " +
      "lists keep their search order. Available sections depend on what you opened.",
    "Previews support engine text (Windows-1251), `dds` images, `ogg` audio and `ogf` models. Other supported " +
      "binary formats show descriptions; linked references select matching files in the open tree. " +
      "Unknown formats may show a chunk tree or file details only.",
    "Extracting `configs\\gameplay` writes its contents directly into the destination, without the " +
      "`configs\\gameplay` parent folders.",
  ],
  limitations: [
    "Extraction overwrites existing destination files without asking. Directory extraction can be cancelled " +
      "between files; files already written remain on disk.",
    "Preview limits are 10 MB for text, 32 MB for `dds` and 64 MB for `ogg`. Larger files can still be " +
      "inspected in `File details` and extracted.",
    "`Game` and `Gamedata` modes omit volume metadata such as offsets, stored sizes and CRCs. Open archives " +
      "through `Directory` or `Archive` to inspect those details and shared payloads.",
    "The explorer does not edit or repack archives. Binary descriptions are summaries, not validation reports; " +
      "use `gamedata verify` to check game data.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_UNPACKER, EApplicationId.ARCHIVES_PACKER],
};
