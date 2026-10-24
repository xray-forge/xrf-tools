import { Nullable } from "@xrf/types";

import { TextureCatalog } from "@/core/ipc/types/xrf-app-textures";

/**
 * What the status bar says about the open root set.
 *
 * @param catalog - The listing, or null for a single texture opened from disk.
 * @param textureCount - Textures after the fold, which is what the tree draws.
 * @param describedCount - Descriptors the sweep has read.
 * @returns The status entries.
 */
export function describeTexturesStatus(
  catalog: Nullable<TextureCatalog>,
  textureCount: number,
  describedCount: number
): Array<string> {
  if (!catalog) {
    return ["One texture"];
  }

  // A loose listing is never swept - there are no engine references to sweep by - so a descriptor count here would be
  // a zero that reads as "the sweep found nothing" rather than "no sweep applies". What is worth saying in its place
  // is how these rows are named, since a folder listed by path looks much like a tree listed by reference.
  const status: Array<string> =
    catalog.mode === "looseDirectory"
      ? [`${textureCount} textures`, "listed by path"]
      : [`${textureCount} textures`, `${describedCount} descriptors`];

  if (catalog.mode !== "looseDirectory" && catalog.outsideTexturesCount) {
    status.push(`${catalog.outsideTexturesCount} outside textures\\`);
  }

  // Left out of either listing: a name that is not Unicode could not be sent back to read the file it names.
  if (catalog.unnamedCount) {
    status.push(`${catalog.unnamedCount} unnamed files`);
  }

  return status;
}
