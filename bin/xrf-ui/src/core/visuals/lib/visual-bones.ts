import { VisualBone } from "@/core/bindings/types/xrf-visual";
import { Optional } from "@/lib/types/general";

/**
 * Bones a weapon hides when its addon is absent.
 *
 * Engine names rather than a convention: `CWeapon` looks these four up by name and hides the ones whose addon status
 * says the part is not attached (`xray-16/src/xrGame/Weapon.cpp:35-38`). Two of them are the grenade launcher:
 * `wpn_launcher` is the current name and `wpn_grenade_launcher` the shadow-of-chernobyl one, and the engine checks
 * both, so a viewer that knew only one would miss half the roster.
 */
export const ADDON_BONE_NAMES: ReadonlyArray<string> = [
  "wpn_scope",
  "wpn_silencer",
  "wpn_launcher",
  "wpn_grenade_launcher",
];

/**
 * How many bones the engine's visibility mask reaches.
 *
 * `visimask` is a `Flags64` indexed by bone id (`xray-16/src/Layers/xrRender/SkeletonCustom.cpp:494`), so a skeleton
 * with more bones than this has some the engine cannot hide at all - hiding one of those is a viewer-only state.
 */
export const VISIBILITY_MASK_BONES: number = 64;

/**
 * Every bone a hidden selection covers, as indices into the bone list.
 *
 * Hiding is inherited, because the engine hides recursively: a scope bone that carries a lens bone hides both, and
 * leaving the child behind would strand it in mid air. Resolved by parent **name** rather than by `parentIndex`, which
 * is the rule the bone tree already follows - ogf stores the hierarchy as names, and a bone naming a parent the file
 * does not contain is a root.
 *
 * @param bones - Bones as the visual lists them, each naming its parent.
 * @param hidden - Bone names the viewer is hiding.
 * @returns Indices of every bone that should be posed as hidden.
 */
export function selectHiddenBoneIndices(bones: Array<VisualBone>, hidden: ReadonlySet<string>): Set<number> {
  const indices: Set<number> = new Set();

  if (!hidden.size) {
    return indices;
  }

  const byName: Map<string, number> = new Map(bones.map((bone: VisualBone, index: number) => [bone.name, index]));

  for (let index: number = 0; index < bones.length; index += 1) {
    // A malformed skeleton can name itself or close a loop, so the walk stops on a bone it has already passed rather
    // than trusting the file to be a tree.
    const walked: Set<string> = new Set();

    let cursor: Optional<VisualBone> = bones[index];

    while (cursor && !walked.has(cursor.name)) {
      if (hidden.has(cursor.name)) {
        indices.add(index);

        break;
      }

      walked.add(cursor.name);

      const parent: Optional<number> = byName.get(cursor.parent);

      cursor = parent === undefined ? undefined : bones[parent];
    }
  }

  return indices;
}

/**
 * The addon bones this visual actually carries, in engine order.
 *
 * @param bones - Bones as the visual lists them.
 * @returns Names of `ADDON_BONE_NAMES` present in this skeleton.
 */
export function selectAddonBones(bones: Array<VisualBone>): Array<string> {
  const present: Set<string> = new Set(bones.map((bone: VisualBone) => bone.name));

  return ADDON_BONE_NAMES.filter((name: string) => present.has(name));
}
