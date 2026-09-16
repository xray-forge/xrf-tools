import { ArchiveLevelLightsGroup } from "@/core/ipc/types/xrf-app";

import { formatCount } from "../ArchiveDescriptionPreview.utils";

/**
 * What one chunk of the file holds.
 *
 * @param group - Chunk to describe.
 * @returns The lights it holds, or a phrase for a payload that was not a run of lights at all.
 */
export function describeGroupContents(group: ArchiveLevelLightsGroup): string {
  if (!group.isLights) {
    return "Not a run of lights";
  }

  return `${formatCount(group.lights)} ${group.lights === 1 ? "light" : "lights"}`;
}

/**
 * What becomes of a chunk at play time.
 *
 * @param group - Chunk to describe.
 * @returns Why the runtime does or does not open it, and what it takes from it when it does.
 */
export function describeGroupUse(group: ArchiveLevelLightsGroup): string {
  if (!group.isReadByEngine) {
    return "CLight_DB::LoadHemi opens no chunk but the header, so the runtime never reads this one";
  }

  return `Opened by CLight_DB::LoadHemi, which keeps the ${formatCount(group.point)} point ${group.point === 1 ? "one" : "ones"}`;
}
