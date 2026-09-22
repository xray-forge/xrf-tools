import { Nullable } from "@xrf/types";

import { ArchiveGameMtlMaterial } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";

/**
 * What a material does to what meets it.
 *
 * @param material - Material to describe.
 * @returns Its friction and bounce, which is what physics takes from it.
 */
export function describePhysics(material: ArchiveGameMtlMaterial): string {
  const friction: string = formatNumber(material.friction, 2);
  const bouncing: string = formatNumber(material.bouncing, 2);

  return `friction ${friction} · bounce ${bouncing}`;
}

/**
 * What a material does to a bullet and to sound.
 *
 * @param material - Material to describe.
 * @returns The two factors that decide whether it is shot through and heard through.
 */
export function describeFactors(material: ArchiveGameMtlMaterial): string {
  const shoot: string = formatNumber(material.shootFactor, 2);
  const occlusion: string = formatNumber(material.soundOcclusionFactor, 2);

  return `shoot ${shoot} · sound ${occlusion}`;
}

/**
 * What qualifies a material beyond its numbers.
 *
 * @param material - Material to describe.
 * @returns Its own description, the flags it sets, and whether it hurts to stand in, or null when it says nothing.
 */
export function describeMaterial(material: ArchiveGameMtlMaterial): Nullable<string> {
  const parts: Array<string> = [];

  if (material.description) {
    parts.push(material.description);
  }

  if (material.flags.length) {
    parts.push(material.flags.join(", "));
  }

  if (material.injuriousSpeed) {
    parts.push(`costs ${formatNumber(material.injuriousSpeed, 2)} health a second to stand in`);
  }

  return parts.length ? parts.join(" · ") : null;
}
