import { ArchiveLevelSomDescription } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";

import { formatCount, NOTHING_TO_MEASURE } from "../ArchiveDescriptionPreview.utils";

/**
 * What the loader ends up with, which is not what the file holds wherever a triangle occludes both ways.
 *
 * @param description - Mesh to describe.
 * @returns A phrase for the faces the sound renderer gets, or for a mesh where they are the triangles themselves.
 */
export function describeTwoSided(description: ArchiveLevelSomDescription): string {
  return description.twoSided
    ? `${formatCount(description.twoSided)} occlude both ways, which the loader doubles into ${formatCount(description.faces)} faces`
    : "Each occluding one way, so the loader takes them as they are";
}

/**
 * How much sound the mesh lets past.
 *
 * @param description - Mesh to describe.
 * @returns The quietest and loudest factors, the one factor a uniform mesh carries, or a phrase for an empty mesh.
 */
export function describeOcclusionRange(description: ArchiveLevelSomDescription): string {
  const { minimumOcclusion, maximumOcclusion } = description;

  if (minimumOcclusion === null || maximumOcclusion === null) {
    return NOTHING_TO_MEASURE;
  }

  const minimum: string = formatNumber(minimumOcclusion, 2);
  const maximum: string = formatNumber(maximumOcclusion, 2);

  return minimum === maximum ? minimum : `${minimum} to ${maximum}`;
}
