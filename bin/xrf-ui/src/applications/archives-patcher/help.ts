import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_PATCHER_HELP: IApplicationHelp = {
  summary:
    "Compares a released base against a new build and packs what changed into archive volumes that override the " +
    "base when the engine mounts them. It exists because `CLocatorAPI::Register` overwrites a descriptor whenever a " +
    "name it already holds is registered again, and `fsgame.ltx` declares `$arch_dir_patches$` immediately before " +
    "`$game_data$` - so an archive dropped into `db\\patches\\` wins over the content archives without touching them.",
  workflow: [
    "Pick the base: the release a player already has. It may be an installation, a directory of volumes, or a " +
      "loose gamedata tree - pointing at an installation is enough, because `fsgame.ltx` is read for every root it " +
      "declares.",
    "Pick the target: the build the patch should deliver.",
    "Pick an output directory outside both roots, and name the volumes.",
    "`Compare` reports what differs and writes nothing. `Write patch` on the result publishes exactly what was " +
      "previewed, without retyping the form.",
  ],
  nuances: [
    "One root per side is enough. An installation expands into every root it declares - `db\\`, its " +
      "subdirectories, `db\\patches\\`, and loose `gamedata\\` - already ordered the way the engine registers " +
      "them, so the layering is read from the game rather than assembled by hand.",
    "Entries are compared by size first and only then by checksum. An archive records the checksum its packer " +
      "wrote, so comparing two volume sets reads no payload at all; a loose side is read only where the sizes " +
      "already match. The report says how many payloads it had to read.",
    "The patch carries the target's bytes for everything added or modified, and nothing else. Two carried entries " +
      "holding identical payloads cost one payload and two descriptor rows.",
    "Narrowing with `Only compare` or `Ignore` applies to both sides at once, so a scope can never turn a file that " +
      "exists on both sides into an addition or a removal.",
  ],
  limitations: [
    "A patch cannot delete. The archive format has no tombstone and the engine only ever overwrites a descriptor, " +
      "so entries the base holds and the target does not are reported and left readable from the base. Removing a " +
      "file means shipping a tree rather than a patch.",
    "Every volume on both sides must mount at the gamedata root. A volume declaring another `entry_point` is " +
      "refused rather than compared against the wrong file.",
    "The output directory must sit outside both roots: a patch written into a tree it compares becomes an " +
      "input to the next run over the same pair.",
    "A side that mounts nothing is refused, because every entry of the other side would otherwise read as a " +
      "difference and the patch would carry the whole game.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_PACKER, EApplicationId.ARCHIVES_EXPLORER, EApplicationId.ARCHIVES_UNPACKER],
};
