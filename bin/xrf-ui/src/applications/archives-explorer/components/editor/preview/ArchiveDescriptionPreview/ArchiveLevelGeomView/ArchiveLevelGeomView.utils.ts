import { ArchiveLevelGeomDescription, ArchiveLevelGeomLayout } from "@/core/ipc/types/xrf-app";

import { formatCount } from "../ArchiveDescriptionPreview.utils";

/**
 * What one vertex layout is made of.
 *
 * @param layout - Layout to describe.
 * @returns The stride with the elements it is built from.
 */
export function describeLayout(layout: ArchiveLevelGeomLayout): string {
  return `${layout.stride} bytes · ${layout.elements} ${layout.elements === 1 ? "element" : "elements"}`;
}

/**
 * How much of a level one layout accounts for.
 *
 * @param layout - Layout to describe.
 * @returns Its buffers and the vertices in them.
 */
export function describeLayoutShare(layout: ArchiveLevelGeomLayout): string {
  const buffers: string = `${layout.buffers} ${layout.buffers === 1 ? "buffer" : "buffers"}`;

  return `${buffers} · ${formatCount(layout.vertices)} vertices`;
}

/**
 * What a level's progressive meshes cost, or why there are none.
 *
 * @param description - Geometry to describe.
 * @returns A phrase for the detail levels, or for geometry that carries none.
 */
export function describeProgressiveMeshes(description: ArchiveLevelGeomDescription): string {
  if (description.isDetail) {
    return "Detail geometry carries none: the renderer reads them from level.geom alone";
  }

  return description.progressiveMeshes
    ? `Over ${formatCount(description.detailLevels)} detail levels`
    : "Nothing on this level drops detail with distance";
}
