import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_PATCHER_HELP: IApplicationHelp = {
  summary:
    "Packs what your loose gamedata changes about a game into archive volumes that override it when the engine " +
    "mounts them. It exists because `CLocatorAPI::Register` overwrites a descriptor whenever a name it already " +
    "holds is registered again, and `fsgame.ltx` declares `$arch_dir_patches$` immediately before `$game_data$` - " +
    "so an archive dropped into `db\\patches\\` wins over the content archives without touching them.",
  workflow: [
    "Pick the game under `Comparison`. Its archives are compared against its own loose `gamedata\\`, which is the " +
      "tree you have been editing in place.",
    "Or tick `Deliver another tree` to build the patch from a separate gamedata folder instead.",
    "Pick an output directory outside the game under `Output`, and name the volumes.",
    "Narrow the comparison under `Selection`, set what the volumes declare under `Header`, and choose how they are " +
      "written under `Options`.",
    "`Compare` and `Patch` in the toolbar are two actions rather than one verb over a mode. Both ask for " +
      "confirmation first, showing what is about to be compared and, for a patch, what the output already holds.",
    "`Import` and `Export` carry the comparison scope and the header between machines. What is compared, where it " +
      "is published and under what name stay with the run.",
  ],
  nuances: [
    "The game alone is the whole configuration. `fsgame.ltx` is read for every root it declares, and that one plan " +
      "is split by source kind: the volumes holding the release on one side, the loose tree overriding it on the " +
      "other. No pair of paths can pose that question - naming the installation twice finds every loose file equal " +
      "to itself, and naming `db\\` reaches only the volumes sitting directly in it.",
    "Files you copied out of an archive but never edited are dropped, so the patch holds what you actually changed " +
      "rather than everything in the folder.",
    "Entries are compared by size first and only then by checksum. An archive records the checksum its packer " +
      "wrote, so comparing two volume sets reads no payload at all; a loose side is read only where the sizes " +
      "already match. The report says how many payloads it had to read.",
    "Narrowing with `Only compare` or `Ignore` applies to both sides at once, so a scope can never turn a file " +
      "that exists on both sides into an addition.",
    "Header entries are merged over the defaults, so naming `creator` keeps the `auto_load` and `entry_point` the " +
      "engine reads without checking whether they are there.",
  ],
  limitations: [
    "A `.db` patch cannot override a file that exists loose in the player's `gamedata\\`. That directory is " +
      "declared last and outranks every archive, so a mod shipping as a loose folder has to keep shipping one.",
    "A patch cannot delete. The archive format has no tombstone and the engine only ever overwrites a descriptor, " +
      "so entries the game holds and the patch does not carry are simply left alone and never reported.",
    "Every volume on the compared side must mount at the gamedata root. A volume declaring another `entry_point` " +
      "is refused rather than compared against the wrong file.",
    "The output must sit outside the game: a patch written into a tree it compares becomes an input to the next " +
      "run over the same pair.",
    "An installation whose loose tree holds nothing is refused - there is nothing to publish yet - and so is a " +
      "gamedata folder named as the game, which has no archives for its files to override.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_PACKER, EApplicationId.ARCHIVES_EXPLORER, EApplicationId.ARCHIVES_UNPACKER],
};
