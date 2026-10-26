import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_UNPACKER_HELP: IApplicationHelp = {
  summary:
    "Writes everything held in a set of game archives back out as loose files. Point it at a directory of " +
    "volumes (`.db*` / `.xdb*`) and every one of them is unpacked in a single run; the archives themselves " +
    "are never modified.",
  workflow: [
    "Pick the source directory. Every volume under it is read, recursively.",
    "Pick the output directory the tree is written into.",
    "`Unpack` reports the volumes it read, the bytes it wrote, and where they went.",
  ],
  nuances: [
    "Volumes merge into one name table the way the engine mounts them: later volumes override earlier " +
      "entries, and volumes under a directory component named exactly `patches` sort last, so patch content " +
      "wins. An overridden file is written once, from whichever volume won.",
    "Each file lands under the root its own archive declares in `[header] entry_point`, with the `$fs_root$` " +
      "alias stripped, so a normal gamedata archive unpacks into `<output>\\gamedata\\...` rather than " +
      "straight into the chosen directory. An archive carrying no header has no such root, and its files land " +
      "directly in the output directory.",
    "Compressed entries are decompressed transparently, and names are decoded as Windows-1251 so Cyrillic " +
      "paths come out readable.",
    "Directories are created first, and only then are files written, up to 32 at a time. The report times " +
      "the two halves separately.",
    "Changing either path clears the previous report, which described a run these paths no longer describe.",
  ],
  limitations: [
    "All or nothing: single files and directories cannot be picked here - the archives explorer is where " +
      "extraction is selective.",
    "Existing files at the destination are overwritten without asking, and nothing already there is removed " +
      "first, so an older unpack of the same archives is left mixed in with this one.",
    "No progress and no cancellation. The first failure ends the run, and whatever reached disk before it " +
      "stays there.",
    "Every volume is read before anything is written, so a source directory holding no readable volume, or " +
      "one unreadable volume among many, fails the whole run.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_EXPLORER, EApplicationId.ARCHIVES_PACKER],
};
