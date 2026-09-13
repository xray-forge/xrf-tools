import { TextureCatalog } from "@/core/ipc/types/xrf-app";
import { Nullable } from "@/lib/types/general";

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
  if (catalog.mode === "looseDirectory") {
    return [`${textureCount} textures`, "listed by path"];
  }

  const status: Array<string> = [`${textureCount} textures`, `${describedCount} descriptors`];

  if (catalog.outsideTexturesCount) {
    status.push(`${catalog.outsideTexturesCount} outside textures\\`);
  }

  return status;
}
